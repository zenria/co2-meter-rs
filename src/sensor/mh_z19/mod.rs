mod mock;
mod real;
pub use self::error::Error as MHZ19SensorError;
pub use self::mock::MockMHZ19Sensor;
pub use self::real::RealMHZ19Sensor;
use crate::mqtt::MqttData;
use actix::Message;
use std::collections::HashMap;

#[derive(Debug, Clone, Message)]
pub struct MHZ19Response {
    pub co2_concentration_ppm: u32,
    pub serial_port: String,
}
pub mod error {
    use mh_z19::MHZ19Error;
    error_chain! {
        foreign_links {
            MHZ19Error(MHZ19Error);
            IOError(std::io::Error);
        }
    }
}

// needed to send data over mqtt
impl MqttData for MHZ19Response {
    fn get_data(&self) -> HashMap<String, Vec<u8>> {
        let mut ret = HashMap::new();
        ret.insert(
            "co2_concentration_ppm".to_string(),
            format!("{}", self.co2_concentration_ppm).into_bytes(),
        );
        ret
    }
}
