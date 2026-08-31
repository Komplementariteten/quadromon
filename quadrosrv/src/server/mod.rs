use crate::client::sensor_dto::SensorDto;
use crate::consts::{DEFAULT_SOCKET, SEPERATOR};
pub use crate::sensors::Config;
use log::{debug, error, info, warn};
use std::fs::{File, remove_file};
use std::io::ErrorKind::NotConnected;
use std::io::Write;
use std::net::Shutdown;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use crate::shared::icp::{bind_socket, local_socket};

const MAX_CASE_SIZE: usize = 1024;

pub struct SensorServer;

static STOP_SYNC: AtomicBool = AtomicBool::new(false);

impl SensorServer {
    pub fn stop(a: JoinHandle<()>, b: JoinHandle<Config>) -> Config {
        STOP_SYNC.store(true, Relaxed);
        a.join().expect("failed to join socket");
        let cfg = b.join().expect("failed to join socket");
        let s_path = local_socket();
        if s_path.exists() {
            remove_file(local_socket()).expect("failed to remove socket file");
        }
        cfg
    }

    /// Actual Processing of the Socket Connection
    /// Using datagram socket
    fn server_th(rx: Receiver<SensorDto>, verbose: bool) -> JoinHandle<()> {
        thread::spawn(move || {
            let mut cache: Vec<SensorDto> = vec![];
            // let mut read_buff = vec![];

            let mut socket = bind_socket();

            while !STOP_SYNC.load(Relaxed) {
                // Versuche, ein Paket mit Timeout zu empfangen, um nicht ewig zu blockieren
                match rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(package) => {
                        cache.push(package);
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        // Kein Paket empfangen, weiter zum Socket-Check
                        if verbose {
                            debug!("timeout waiting for package...");
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        // Sender wurde geschlossen, Haupt-Thread möchte uns beenden
                        error!("Consumer thread sender disconnected. Exiting.");
                        break;
                    }
                }

                // Wenn Pakete im Cache sind, versuche, eines an den Socket zu schreiben
                if let Some(sensor_dto) = cache.pop() {
                    let bytes = bitcode::encode(&sensor_dto);
                    let size = bytes.len();
                    let mut data = SEPERATOR.to_vec();
                    let size_bytes = bitcode::encode(&size);
                    data.reserve(size_bytes.len() + bytes.len());
                    data.extend(size_bytes);
                    data.extend(bytes);
                    match socket.send(&data) {
                        Ok(_) => {
                            info!("package successfully written");
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            cache.push(sensor_dto); // Paket zurück in den Cache, wenn Socket nicht bereit ist
                            thread::sleep(Duration::from_millis(5)); // Kurze Pause, um Busy-Waiting zu vermeiden
                        }
                        Err(e) => {
                            if e.kind() != NotConnected {
                                warn!("Error writing to socket in consumer thread: {:?}", e);
                                thread::sleep(Duration::from_millis(5));
                            }
                        }
                    }
                } else {
                    thread::sleep(Duration::from_millis(5)); // Keine Pakete zum Senden, kurze Pause
                }

                if cache.len() > MAX_CASE_SIZE {
                    error!("Cache size exceeded: {}", cache.len());
                    break;
                }
            }
            socket.shutdown(Shutdown::Both).unwrap();
            remove_file(local_socket()).unwrap_or_else(|e| {
                error!("failed to remove socket: {:?}", e);
            });
            info!("Network middleware thread finished.");
        })
    }

    /// Triggers Modules for actual sensor reading
    fn reader_thread(tx: Sender<SensorDto>, config: &Config) -> JoinHandle<Config> {
        let mut local_cfg = config.clone();
        thread::spawn(move || {
            while !STOP_SYNC.load(Relaxed) {
                for m in &mut local_cfg.modules {
                    let exports = m.read(&local_cfg.verbose);

                    for dto in exports {
                        match tx.send(dto) {
                            Ok(_) => (),
                            Err(e) => info!("failed to send sensor data: {:?}", e),
                        }
                    }
                }
                thread::sleep(Duration::from_secs(2));
            }
            info!("Reader thread finished.");
            local_cfg
        })
    }

    pub fn start(config: &Config) -> (JoinHandle<()>, JoinHandle<Config>) {
        let (tx, rx) = mpsc::channel();
        let sender_h = SensorServer::server_th(rx, config.verbose);
        let sensor_h = SensorServer::reader_thread(tx.clone(), config);
        (sender_h, sensor_h)
    }
}

impl Drop for SensorServer {
    fn drop(&mut self) {
        let f = local_socket();
        if f.exists() {
            remove_file(f).expect("Failed to remove local socket file");
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sensors::Config;
    use crate::server::SensorServer;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_init() {
        let (server, reader) = SensorServer::start(&Config::default());
        thread::sleep(Duration::from_millis(2000));
        SensorServer::stop(server, reader);
        assert!(true);
    }
}
