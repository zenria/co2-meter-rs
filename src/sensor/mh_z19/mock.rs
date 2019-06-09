use crate::sensor::mh_z19::{MHZ19Command, MHZ19Response, MHZ19Sensor};
use crossbeam_channel::{Receiver, Sender};

pub struct MockMHZ19Sensor;

impl MHZ19Sensor for MockMHZ19Sensor {
    fn start(self) -> (Sender<MHZ19Command>, Receiver<MHZ19Response>) {
        let (tx_data, rx_data) = crossbeam_channel::bounded(1);
        let (tx_cmd, rx_cmd) = crossbeam_channel::bounded(1);
        // spawn thread that never terminates
        std::thread::Builder::new()
            .name("Mock Sensor Read Thread".to_string())
            .spawn(move || loop {
                match rx_cmd.recv() {
                    Ok(cmd) => match cmd {
                        MHZ19Command::Read => {
                            if let Err(e) = tx_data.send(MHZ19Response {
                                co2_concentration_ppm: 411,
                            }) {
                                return; // channel disconnected
                            }
                        }
                        other => info!("Received command: {:?}", other),
                    },

                    Err(e) => return, // channel disconnected
                }
            })
            .expect("Unable to create mock sensor read thread (WTF!)");
        (tx_cmd, rx_data)
    }
}
