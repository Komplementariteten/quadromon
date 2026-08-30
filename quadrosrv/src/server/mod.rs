pub mod load;

use crate::client::sensor_dto::SensorDto;
use crate::consts::{DEFAULT_SOCKET, SEPERATOR};
use crate::sensors::Config;
use log::{error, info};
use std::fs::{File, remove_file};
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

const MAX_CASE_SIZE: usize = 1024;

pub struct SensorServer;

static STOP_SYNC: AtomicBool = AtomicBool::new(false);

impl SensorServer {
    fn local_socket() -> PathBuf {
        let path = load::base_path();
        let fs = path.join(DEFAULT_SOCKET);
        if !path.exists() {
            File::create(&fs).unwrap();
        }
        fs
    }

    fn init_socket() -> UnixStream {
        let p = Self::local_socket();

        match UnixStream::connect(&p) {
            Ok(s) => s,
            Err(_) => {
                let listener = match UnixListener::bind(&p) {
                    Ok(listener) => listener,
                    Err(e) => panic!("failed to create socket: {}", e),
                };
                let addr = listener.local_addr().unwrap();
                UnixStream::connect_addr(&addr)
                    .unwrap_or_else(|e| panic!("Could not connect to socket with error: {}", e))
            }
        }
    }

    pub fn stop(a: JoinHandle<()>, b: JoinHandle<Config>) -> Config {
        STOP_SYNC.store(true, Relaxed);
        a.join().expect("failed to join socket");
        let cfg = b.join().expect("failed to join socket");
        let s_path = Self::local_socket();
        if s_path.exists() {
            remove_file(Self::local_socket()).expect("failed to remove socket file");
        }
        cfg
    }

    /// Actual Processing of the Socket Connection
    fn server_th(rx: Receiver<SensorDto>) -> JoinHandle<()> {
        thread::spawn(move || {
            let mut cache: Vec<SensorDto> = vec![];
            // let mut read_buff = vec![];

            let mut socket = SensorServer::init_socket();
            socket
                .set_nonblocking(true)
                .expect("Failed to set socket non-blocking"); // Socket auf nicht-blockierend setzen

            while !STOP_SYNC.load(Relaxed) {
                // Versuche, ein Paket mit Timeout zu empfangen, um nicht ewig zu blockieren
                match rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(package) => {
                        cache.push(package);
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        // Kein Paket empfangen, weiter zum Socket-Check
                        println!("timeout waiting for package...");
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        // Sender wurde geschlossen, Haupt-Thread möchte uns beenden
                        println!("Consumer thread sender disconnected. Exiting.");
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
                    match socket.write_all(&data) {
                        Ok(_) => {
                            println!("package successfully written");
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            cache.push(sensor_dto); // Paket zurück in den Cache, wenn Socket nicht bereit ist
                            thread::sleep(Duration::from_millis(5)); // Kurze Pause, um Busy-Waiting zu vermeiden
                        }
                        Err(e) => {
                            eprintln!("Error writing to socket in consumer thread: {:?}", e);
                            thread::sleep(Duration::from_millis(5));
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
            remove_file(SensorServer::local_socket()).unwrap_or_else(|e| {
                println!("failed to remove socket: {:?}", e);
            });
            println!("Consumer thread finished.");
        })
    }

    /// Triggers Modules for actual sensor reading
    fn reader_thread(tx: Sender<SensorDto>, config: &Config) -> JoinHandle<Config> {
        let mut local_cfg = config.clone();
        thread::spawn(move || {
            while !STOP_SYNC.load(Relaxed) {
                for m in &mut local_cfg.modules {
                    let exports = m.read();

                    for dto in exports {
                        match tx.send(dto) {
                            Ok(_) => (),
                            Err(e) => info!("failed to send sensor data: {:?}", e),
                        }
                    }
                }
                thread::sleep(Duration::from_secs(2));
            }
            println!("Sensor thread finished.");
            local_cfg
        })
    }

    pub fn start(config: &Config) -> (JoinHandle<()>, JoinHandle<Config>) {
        let (tx, rx) = mpsc::channel();
        let sender_h = SensorServer::server_th(rx);
        let sensor_h = SensorServer::reader_thread(tx.clone(), config);
        (sender_h, sensor_h)
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
