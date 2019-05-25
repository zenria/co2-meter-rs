use crate::sensor::mh_z19::{MHZ19Response, MHZ19SensorError};
use crate::sensor::ReadMessage;
use actix::prelude::*;

pub struct MockMHZ19Sensor;

impl Actor for MockMHZ19Sensor {
    type Context = SyncContext<MockMHZ19Sensor>;
}

impl Handler<ReadMessage<MHZ19Response, MHZ19SensorError>> for MockMHZ19Sensor {
    type Result = Result<MHZ19Response, MHZ19SensorError>;

    fn handle(
        &mut self,
        msg: ReadMessage<MHZ19Response, MHZ19SensorError>,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        Ok(MHZ19Response {
            co2_concentration_ppm: 411,
            serial_port: "__mock__".to_string(),
        })
    }
}
