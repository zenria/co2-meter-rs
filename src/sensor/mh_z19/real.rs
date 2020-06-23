use crate::sensor::mh_z19::{MHZ19Command, MHZ19Response, MHZ19Sensor};
use crossbeam::channel::{Receiver, Sender};
use mh_z19::read_gas_concentration;
use serialport::prelude::*;
use serialport::{DataBits, Parity, SerialPortSettings, StopBits};
use std::io::{ErrorKind, Write};
use std::thread;
use std::time::Duration;

pub struct RealMHZ19Sensor {
    opened_port: Box<dyn SerialPort>,
}

impl RealMHZ19Sensor {
    pub fn new(serial_port: String, serial_timeout: Duration) -> Self {
        let mut settings: SerialPortSettings = Default::default();
        settings.timeout = serial_timeout;
        // MH-Z19 serial settings
        settings.baud_rate = 9600;
        settings.parity = Parity::None;
        settings.data_bits = DataBits::Eight;
        settings.stop_bits = StopBits::One;

        match serialport::open_with_settings(&serial_port, &settings) {
            Ok(opened_port) => RealMHZ19Sensor { opened_port },
            Err(e) => {
                error!("Unable to open port {}", e);
                std::process::exit(1)
            }
        }
    }
}

impl MHZ19Sensor for RealMHZ19Sensor {
    fn start(self) -> (Sender<MHZ19Command>, Receiver<MHZ19Response>) {
        // Spawn 2 threads:
        //  - write read gas command thread periodically
        //  - read serial data loop
        let (tx_data, rx_data) = crossbeam::channel::bounded(1);
        let (tx_cmd, rx_cmd) = crossbeam::channel::bounded(1);
        // ----- Write thread
        let mut serial_port_w = self
            .opened_port
            .try_clone()
            .expect("Cannot open port for write!");
        thread::Builder::new()
            .name("MHZ19 Command Write Thread".to_string())
            .spawn(move || loop {
                match rx_cmd.recv() {
                    Ok(cmd) => match cmd {
                        MHZ19Command::Read => {
                            if let Err(e) = serial_port_w
                                .write_all(&read_gas_concentration(1))
                            {
                                error!(
                                    "Unable to write 'read gas command' to the serial port: {}",
                                    e
                                );
                            }
                        }
                        MHZ19Command::CalibrateZero => {
                            if let Err(e) =
                                serial_port_w.write_all(&mh_z19::calibrate_zero_point(1))
                            {
                                error!(
                                    "Unable to write 'calibrate zero' to the serial port: {}",
                                    e
                                );
                            }
                        }
                        MHZ19Command::SetAutomaticBaselineCorrection {enabled} => {
                            if let Err(e) =
                            serial_port_w.write_all(&mh_z19::set_automatic_baseline_correction(1, enabled))
                            {
                                error!(
                                    "Unable to write 'set automatic baseline correction command' to the serial port: {}",
                                    e
                                );
                            }
                        }
                    },
                    Err(e) => return, // channel disconnected
                }
            })
            .expect("Unable to create serial write thread");

        // ----- read Thread
        let mut serial_port_r = self.opened_port;
        thread::Builder::new()
            .name("MHZ19 Read Thread".to_string())
            .spawn(move || {
                let mut serial_buf: Vec<u8> = vec![0; 9];
                loop {
                    match &serial_port_r.read_exact(serial_buf.as_mut_slice()) {
                        // general read error
                        Err(e) if e.kind() == ErrorKind::TimedOut => (),
                        // timeout: ignore & continue
                        Err(e) => error!("Unable to read serial port: {}", e),
                        // let's decode real data
                        Ok(_) => match mh_z19::parse_gas_contentration_ppm(&serial_buf) {
                            Err(e) => error!("Error decoding recieved data: {}", e),
                            Ok(co2_concentration_ppm) => {
                                if let Err(e) = tx_data.send(MHZ19Response {
                                    co2_concentration_ppm,
                                }) {
                                    error!("Error sending recieved data... {}", e);
                                }
                            }
                        },
                    }
                }
            })
            .expect("Unable to create serial read thread");

        (tx_cmd, rx_data)
    }
}
