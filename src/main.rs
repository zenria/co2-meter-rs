#![allow(dead_code)]
#![allow(unused_variables)]

#[macro_use]
extern crate log;

use std::time::Duration;

use crate::datastore::DataStore;
use crate::mqtt::MqttConfig;
use crate::sensor::mh_z19::{
    MHZ19Command, MHZ19Response, MHZ19Sensor, MockMHZ19Sensor, RealMHZ19Sensor,
};
use crossbeam::channel::Sender;
use env_logger::Env;
use rouille::Response;
use std::sync::{Arc, RwLock};
use structopt::StructOpt;

mod datastore;
mod mqtt;
mod sensor;

#[derive(StructOpt, Debug)]
#[structopt(name = "Reads MH-Z19 CO2 meter from serial and publish it to a MQTT topic")]
struct Opt {
    /// print out some debugging information
    #[structopt(short = "d", long = "debug")]
    debug: bool,
    #[structopt(short = "b", long = "bind-address", default_value = "0.0.0.0:9090")]
    bind_address: String,
    #[structopt(short = "m", long = "mockserial")]
    mock_serial: bool,
    /// Sensor read interval in second
    #[structopt(short = "i", long = "read-interval", default_value = "60")]
    read_interval_secs: u64,
    /// History size kept for /debug endpoint
    #[structopt(short = "h", long = "history-size", default_value = "300")]
    debug_history_size: usize,
    /// The serial port name or device path (eg: /dev/ttyS0 or COM3)
    #[structopt(name = "serial_port")]
    serial_port: String,
    #[structopt(long = "mqtt-port", default_value = "1883")]
    mqtt_port: u16,
    #[structopt(long = "mqtt-host", default_value = "mosquitto")]
    mqtt_host: String,
    #[structopt(long = "mqtt-base-topic", default_value = "co2-meter/dev")]
    mqtt_base_topic: String,
}

fn main() {
    let opt: Opt = Opt::from_args();
    env_logger::from_env(Env::default().default_filter_or(if opt.debug {
        "debug"
    } else {
        "info"
    }))
    .init();

    debug!("Config {:?}", opt);

    let read_interval = Duration::from_secs(opt.read_interval_secs);

    let data_store = Arc::new(RwLock::new(datastore::DataStore::new(
        opt.debug_history_size,
    )));
    let mut mqtt_sender = {
        let mqtt_host = opt.mqtt_host.clone();
        let mqtt_port = opt.mqtt_port;
        let mqtt_base_topic = opt.mqtt_base_topic.clone();

        mqtt::MqttSender::new(MqttConfig::new(
            mqtt_host.clone(),
            mqtt_port,
            mqtt_base_topic.clone(),
        ))
    };

    let (cmd_sender, data_receiver) = if opt.mock_serial {
        MockMHZ19Sensor.start()
    } else {
        let serial_port = opt.serial_port.clone();
        let serial_timeout_secs = opt.read_interval_secs;
        RealMHZ19Sensor::new(
            serial_port.clone(),
            Duration::from_secs(serial_timeout_secs),
        )
        .start()
    };
    launch_read_gas_timer_thread(
        cmd_sender.clone(),
        Duration::from_secs(opt.read_interval_secs),
    );
    info!(
        "CO2 Meter started, reading sensor every {}s.",
        opt.read_interval_secs
    );

    launch_http_server(&opt, data_store.clone(), cmd_sender);

    loop {
        // if no data received during 10 read cycles, then let's throw an error and quit
        match data_receiver.recv_timeout(Duration::from_secs(opt.read_interval_secs * 10)) {
            Err(timeout_error) => {
                error!("No data received during 10 read cycle, please read the logs to find out the bug - {}!!", timeout_error);
                std::process::exit(1);
            }
            Ok(data) => {
                debug!("Read data {:?}", data);
                // store the data
                data_store.write().unwrap().insert(data.clone());
                mqtt_sender.send_data(data);
            }
        }
    }
}

fn launch_http_server(
    opt: &Opt,
    data_store: Arc<RwLock<DataStore<MHZ19Response>>>,
    cmd_sender: Sender<MHZ19Command>,
) {
    std::thread::Builder::new()
        .name("HTTP Server Launcher".to_string())
        .spawn({
            let bind_address = opt.bind_address.clone();
            info!("Starting HTTP server on {}", bind_address);
            move || {
                rouille::start_server(bind_address, move |request| {
                    info!("{} {}", request.method(), request.url());
                    match request.url().as_str() {
                        "/debug" => Response::text(format!("{}", data_store.read().unwrap())),
                        "/cmd/zero" => send_command(MHZ19Command::CalibrateZero, &cmd_sender),
                        "/cmd/abc/enable" => send_command(
                            MHZ19Command::SetAutomaticBaselineCorrection { enabled: true },
                            &cmd_sender,
                        ),
                        "/cmd/abc/disable" => send_command(
                            MHZ19Command::SetAutomaticBaselineCorrection { enabled: false },
                            &cmd_sender,
                        ),
                        "/cmd/read" => send_command(MHZ19Command::Read, &cmd_sender),
                        _ => Response::empty_404(),
                    }
                })
            }
        })
        .expect("Cannot start http server thread");
}

fn send_command(command: MHZ19Command, cmd_sender: &Sender<MHZ19Command>) -> Response {
    match cmd_sender.send(command) {
        Ok(_) => Response::text(format!("{:?} sent!", command)),
        Err(e) => {
            error!("Unable to send command - {}", e);
            Response::text("An error occured!").with_status_code(500)
        }
    }
}

fn launch_read_gas_timer_thread(cmd_sender: Sender<MHZ19Command>, read_interval: Duration) {
    std::thread::Builder::new()
        .name("gas timer thread".to_string())
        .spawn(move || loop {
            if let Err(_) = cmd_sender.send(MHZ19Command::Read) {
                return; // Closed channel
            }
            std::thread::sleep(read_interval);
        })
        .unwrap();
}
