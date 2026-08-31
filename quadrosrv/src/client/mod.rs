use std::io::Read;
use log::{error, info};
use crate::client::sensor_dto::SensorDto;
use crate::shared::icp::bind_socket;

pub mod sensor_dto;

pub struct Client;

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

impl Client {
    pub fn new() -> Self {
        Client
    }
    pub fn read(&self) -> Option<SensorDto> {
        let mut s = bind_socket();
        s.set_nonblocking(true).expect("Failed to set socket to non-blocking mode");
        let mut buff = vec![];
        let mut read_bytes: usize = 0;
        match s.read_to_end(&mut buff) {
            Ok(b) => { read_bytes = b }
            Err(e) => {
                error!("Error reading from socket: {}", e);
            }
        }

        if read_bytes == 0 {
            return None;
        }

        if read_bytes != buff.len() {
            error!("Error reading from socket: Invalid number of bytes read");
            return None;
        }

        let dto: SensorDto = match bitcode::decode(&buff) {
            Ok(dto) => dto,
            Err(e) => {
                error!("Error decoding bitcode: {}", e);
                return None;
            }
        };

        info!("Received sensor data: {:?}", dto);

        if let Err(e) = s.shutdown(std::net::Shutdown::Both) {
            error!("Error shutting down socket: {}", e);
            return None;
        }
        None
    }
}
