use crate::client::sensor_dto::SensorDto;
use crate::consts::SEPERATOR;
use crate::shared::icp::connect_socket;
use log::{error, info, warn};
use std::io::{BufReader, Read};
use std::net::TcpStream;
use std::{mem, thread};
use std::time::Duration;

pub mod sensor_dto;

pub struct Client {
    ep: TcpStream,
    buf: Vec<u8>,
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
        Client { ep: s, buf: vec![] }
    }

    pub fn stop(&mut self) {
        info!("Client stopped");
        self.ep
            .shutdown(std::net::Shutdown::Both)
            .expect("Failed to shutdown socket");
    }

    fn handle_buff(&self) -> Option<SensorDto> {
        if self.buf.len() <= (2 * SEPERATOR.len()) {
            return None;
        }

        let buf_size = self.buf.len();
        if self.buf[buf_size - SEPERATOR.len()..buf_size] == SEPERATOR {
            let usize_bytes = mem::size_of::<usize>();
            let size_bytes = self.buf[..usize_bytes].to_vec();
            let payload_size = usize::from_ne_bytes(size_bytes.try_into().unwrap());
            let payload = self.buf[usize_bytes..(payload_size + usize_bytes)].to_vec();
            info!("first:{:#04X?}, last:{:#04X?}", payload[0], payload[payload_size - 1]);
            if let Ok(dto) = bitcode::decode(&payload) {
                return Some(dto);
            }
            warn!("Failed to decode package, end {:#04X?}", payload[payload.len() - SEPERATOR.len()..].to_vec());
        }
        None
    }

    pub fn read(&mut self) -> Option<SensorDto> {
        let mut reader = BufReader::new(&self.ep);
        let mut seen_package: bool = false;
        let mut read_buff: [u8; 8] = [0; 8];

        while !seen_package {
            match reader.read(&mut read_buff) {
                Ok(n) => {
                    self.buf.extend_from_slice(&read_buff[..n]);
                    if let Some(package) = self.handle_buff() {
                        seen_package = true;
                        return Some(package);
                    }
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::WouldBlock {
                        continue;
                    }
                    error!("Error reading from socket: {}", e);
                    thread::sleep(Duration::from_millis(20));
                }
            }
            thread::sleep(Duration::from_millis(5));
        };
        None
    }

    pub fn read_old(&mut self) -> Option<SensorDto> {
        let mut reader = BufReader::new(&self.ep);

        let mut buff = [0; 8];
        let mut expected_bytes: usize = 0;
        let mut read_bytes: usize = 0;
        let mut seen_package: bool = false;
        let mut package_bytes = vec![];
        let mut read_error = 0;
        while !seen_package {
            match reader.read_exact(&mut buff) {
                Ok(b) => {
                    read_bytes = 8;
                }
                Err(e) => {
                    // error!("Error reading from socket: {}", e);
                    read_error += 1;
                    if read_error > 10 {
                        return None;
                    }
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
            error!(
                "Error reading from socket: Invalid package size {:#04X?}",
                last
            );
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
