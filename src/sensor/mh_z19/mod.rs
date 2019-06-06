mod mock;
mod real;
pub use self::mock::MockMHZ19Sensor;
pub use self::real::RealMHZ19Sensor;
use crate::mqtt::MqttData;
use std::collections::HashMap;
use std::time::Duration;

pub trait MHZ19Sensor {
    fn start(self, read_interval: Duration) -> crossbeam_channel::Receiver<MHZ19Response>;
}

#[derive(Debug, Clone)]
pub struct MHZ19Response {
    pub co2_concentration_ppm: u32,
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
