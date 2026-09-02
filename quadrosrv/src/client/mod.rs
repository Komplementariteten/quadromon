use crate::client::sensor_dto::SensorDto;
use crate::consts::SEPERATOR;
use crate::shared::icp::connect_socket;
use log::{error, info};
use std::io::Read;
use std::net::TcpStream;
use std::os::unix::net::UnixDatagram;
use std::thread;
use std::time::Duration;

pub mod sensor_dto;

pub struct Client {
    ep: TcpStream,
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
        info!("Client created");
        Client { ep: s }
    }
    pub fn read(&mut self) -> Option<SensorDto> {
        let mut buff = [0; 8];
        let mut expected_bytes: usize = 0;
        let mut read_bytes: usize = 0;
        let mut seen_package: bool = false;
        let mut package_bytes = vec![];
        while !seen_package {
            match self.ep.read(&mut buff) {
                Ok(b) => {
                    read_bytes = b;
                }
                Err(e) => {
                    // error!("Error reading from socket: {}", e);
                    thread::sleep(Duration::from_millis(100));
                }
            }

            if read_bytes == 8 && expected_bytes == 0 {
                expected_bytes = u64::from_ne_bytes(buff) as usize;
                continue;
            }

            if buff[0..read_bytes] == SEPERATOR {
                seen_package = true;
                break;
            }

            if expected_bytes >= package_bytes.len() {
                package_bytes.extend_from_slice(&buff[0..read_bytes]);
            }
        }

        if expected_bytes > package_bytes.len() {
            let last = package_bytes.last_chunk::<8>()?;
            error!("Error reading from socket: Invalid package size {:#04X?}", last);
            return None;
        }
        
        if expected_bytes != package_bytes.len() {
            package_bytes = package_bytes[..expected_bytes].to_vec();
        }

        let dto: SensorDto = match bitcode::decode(&package_bytes) {
            Ok(dto) => dto,
            Err(e) => {
                error!("Error decoding bitcode: {}", e);
                return None;
            }
        };

        expected_bytes = 0;
        package_bytes.clear();
        Some(dto)
    }
}
