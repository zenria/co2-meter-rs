mod mock;
mod real;
pub use self::error::Error as MHZ19SensorError;
pub use self::mock::MockMHZ19Sensor;
pub use self::real::RealMHZ19Sensor;

#[derive(Debug, Clone)]
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
