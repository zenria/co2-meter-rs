mod mock;
mod real;
pub use self::mock::MockMHZ19Sensor;
pub use self::real::RealMHZ19Sensor;
use crate::mqtt::MqttData;
use crossbeam::channel::{Receiver, Sender};
use std::collections::HashMap;

pub trait MHZ19Sensor {
    fn start(self) -> (Sender<MHZ19Command>, Receiver<MHZ19Response>);
}

#[derive(Debug, Clone)]
pub struct MHZ19Response {
    pub co2_concentration_ppm: u32,
}
#[derive(Debug, Clone)]
pub enum MHZ19Command {
    CalibrateZero,
    SetAutomaticBaselineCorrection { enabled: bool },
    Read,
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
