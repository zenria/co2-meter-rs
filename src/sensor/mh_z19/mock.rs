use crate::sensor::mh_z19::{MHZ19Response, MHZ19Sensor};
use crossbeam_channel::Receiver;
use std::time::Duration;

pub struct MockMHZ19Sensor;

impl MHZ19Sensor for MockMHZ19Sensor {
    fn start(self, read_interval: Duration) -> Receiver<MHZ19Response> {
        let (tx, rx) = crossbeam_channel::bounded(1);
        // spawn thread that never terminates
        std::thread::Builder::new()
            .name("Mock Sensor Read Thread".to_string())
            .spawn(move || loop {
                if let Err(e) = tx.send(MHZ19Response {
                    co2_concentration_ppm: 411,
                }) {
                    error!("Unable to send mock values {}", e);
                }
                std::thread::sleep(read_interval);
            })
            .expect("Unable to create mock sensor read thread (WTF!)");
        rx
    }
}
