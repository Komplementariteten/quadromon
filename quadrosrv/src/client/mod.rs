use std::io::Read;
use std::os::unix::net::UnixDatagram;
use std::time::Duration;
use log::{error, info};
use crate::client::sensor_dto::SensorDto;
use crate::shared::icp::{bind_socket_sream, connect_socket};

pub mod sensor_dto;

pub struct Client {
    ep: UnixDatagram   
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        if let Err(e) = self.ep.shutdown(std::net::Shutdown::Both) {
            error!("Error shutting down socket: {}", e);
        }
    }
}

impl Client {
    pub fn new() -> Self {
        let s = connect_socket(Some(Duration::from_millis(1000)));
        Client { ep: s }
    }
    pub fn read(&mut self) -> Option<SensorDto> {
        let mut buff = vec![];
        let mut read_bytes: usize = 0;
        match self.ep.recv(&mut buff) {
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
        None
    }
}
