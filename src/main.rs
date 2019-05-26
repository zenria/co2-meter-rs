#![allow(dead_code)]
#![allow(unused_variables)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate error_chain;

use std::time::Duration;

use crate::mqtt::MqttConfig;
use crate::sensor::mh_z19::{MockMHZ19Sensor, RealMHZ19Sensor};
use actix::prelude::*;
use env_logger::Env;
use structopt::StructOpt;

//mod broadcast;
mod datastore;
mod http;
mod mqtt;
mod sensor;

#[derive(StructOpt, Debug)]
#[structopt(name = "Flush memcache cluster")]
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

    debug!("Starting actix");
    let sys = System::new("guide");

    debug!("Starting sensor actor");
    let read_interval = Duration::from_secs(opt.read_interval_secs);
    let data_store = datastore::DataStore::new(opt.debug_history_size).start();
    let mqtt_sender = {
        let mqtt_host = opt.mqtt_host.clone();
        let mqtt_port = opt.mqtt_port;
        let mqtt_base_topic = opt.mqtt_base_topic.clone();
        SyncArbiter::start(1, move || {
            mqtt::MqttSender::new(MqttConfig::new(
                mqtt_host.clone(),
                mqtt_port,
                mqtt_base_topic.clone(),
            ))
        })
    };
    if opt.mock_serial {
        sensor::SensorReader::new(
            SyncArbiter::start(1, || MockMHZ19Sensor),
            data_store.clone(),
            mqtt_sender.clone(),
            read_interval,
        )
        .start();
    } else {
        let serial_port = opt.serial_port.clone();
        sensor::SensorReader::new(
            SyncArbiter::start(1, move || RealMHZ19Sensor::new(serial_port.clone())),
            data_store.clone(),
            mqtt_sender.clone(),
            read_interval,
        )
        .start();
    }
    debug!("Starting http server");

    http::launch_http_server(&opt.bind_address, data_store.clone());

    info!(
        "CO2 Meter started, reading sensor every {}s.",
        opt.read_interval_secs
    );
    let _ = sys.run();
}
