use crate::sensor::mh_z19::{MHZ19Response, MHZ19SensorError};
use crate::sensor::ReadMessage;
use actix::prelude::*;
use serialport::prelude::*;
use serialport::{DataBits, Parity, SerialPortSettings, StopBits};
use std::io::Write;
use std::time::Duration;

pub struct RealMHZ19Sensor {
    serial_port: String,
    opened_port: Box<SerialPort>,
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
            Ok(opened_port) => RealMHZ19Sensor {
                serial_port,
                opened_port,
            },
            Err(e) => {
                error!("Unable to open port {}", e);
                std::process::exit(1)
            }
        }
    }
}

impl Actor for RealMHZ19Sensor {
    type Context = SyncContext<Self>;
}
impl Handler<ReadMessage<MHZ19Response, MHZ19SensorError>> for RealMHZ19Sensor {
    type Result = Result<MHZ19Response, MHZ19SensorError>;

    fn handle(
        &mut self,
        msg: ReadMessage<MHZ19Response, MHZ19SensorError>,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        let mut serial_buf: Vec<u8> = vec![0; 9];
        self.opened_port
            .write_all(mh_z19::READ_GAS_CONCENTRATION_COMMAND_ON_DEV1_PACKET)?;

        let bytes_read = self.opened_port.read(serial_buf.as_mut_slice())?;
        Ok(MHZ19Response {
            co2_concentration_ppm: mh_z19::get_gas_contentration_ppm(&serial_buf[..bytes_read])?,
            serial_port: self.serial_port.clone(),
        })
    }
}
