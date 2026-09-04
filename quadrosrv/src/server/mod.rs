use crate::client::sensor_dto::SensorDto;
use crate::consts::{SEPERATOR};
pub use crate::sensors::Config;
use crate::shared::icp::{bind_socket_dgram, bind_socket_listener, srv_socket_file};
use log::{debug, error, info, warn};
use std::fs::{remove_file};
use std::io::ErrorKind::NotConnected;
use std::io::Write;
use std::net::{Shutdown, TcpStream};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::{mem, thread};
use std::thread::JoinHandle;
use std::time::Duration;

const MAX_CASE_SIZE: usize = 1024;

pub struct SensorServer;

static STOP_SYNC: AtomicBool = AtomicBool::new(false);
static CLIENT_CONNECTED: AtomicBool = AtomicBool::new(false);

impl SensorServer {
    pub fn stop(a: JoinHandle<()>, b: JoinHandle<Config>) -> Config {
        STOP_SYNC.store(true, Relaxed);
        a.join().expect("failed to join socket");
        let cfg = b.join().expect("failed to join socket");
        let s_path = srv_socket_file();
        if s_path.exists() {
            remove_file(srv_socket_file()).expect("failed to remove socket file");
        }
        cfg
    }

    /// Actual Processing of the Socket Connection
    /// Using datagram socket
    fn server_th_stream(rx: Receiver<SensorDto>, verbose: bool) -> JoinHandle<()> {
        thread::spawn(move || {
            let mut cache: Vec<SensorDto> = vec![];
            let listener = bind_socket_listener();

            for stream in listener.incoming() {
                match stream {
                    Ok(mut client) => {
                        CLIENT_CONNECTED.store(true, Relaxed);
                        match handle_connection(&rx, &mut cache, &mut client, verbose) {
                            Ok(_) => info!("Client disconnected"),
                            Err(e) => error!("Failed to handle connection: {}", e),
                        }
                        CLIENT_CONNECTED.store(false, Relaxed);
                    }
                    Err(e) => {
                        error!("Failed to accept socket connection: {}", e);
                    }
                }
            }
            info!("Network middleware thread finished.");
        })
    }

    /// Triggers Modules for actual sensor reading
    fn reader_thread(tx: Sender<SensorDto>, config: &Config) -> JoinHandle<Config> {
        let mut local_cfg = config.clone();
        thread::spawn(move || {
            while !STOP_SYNC.load(Relaxed) {
                if !CLIENT_CONNECTED.load(Relaxed) {
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }
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
        let sender_h = SensorServer::server_th_stream(rx, config.verbose);
        let sensor_h = SensorServer::reader_thread(tx.clone(), config);
        log::info!("Sensor server started");
        (sender_h, sensor_h)
    }
}
fn handle_connection(
    rx: &Receiver<SensorDto>,
    cache: &mut Vec<SensorDto>,
    client: &mut TcpStream,
    verbose: bool,
) -> Result<(), std::io::Error> {
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
                return Err(std::io::Error::other(
                    "Consumer thread sender disconnected",
                ));
            }
        }

        // Wenn Pakete im Cache sind, versuche, eines an den Socket zu schreiben
        if let Some(sensor_dto) = cache.pop() {
            let mut bytes = bitcode::encode(&sensor_dto);
            let size = bytes.len();
            let ex = size % 8;
            (0..ex).for_each(|_| {
                bytes.push(0);
            });
            let mut data = vec![];
            let size_bytes = size.to_ne_bytes();
            let total = size_bytes.len() + bytes.len() + SEPERATOR.len();
            let byte_lign = total % 8;
            info!("first:{:#04X?}, last:{:#04X?}", bytes[0], bytes[bytes.len() - ex - 1]);
            data.reserve(total + byte_lign);
            data.extend(size_bytes);
            data.extend(bytes);
            data.extend_from_slice(&SEPERATOR);

            match client.write_all(&data) {
                Ok(_) => {
                    client.flush().expect("Failed to flush socket");
                    info!("package successfully written to socket");
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    cache.push(sensor_dto); // Paket zurück in den Cache, wenn Socket nicht bereit ist
                    thread::sleep(Duration::from_millis(5)); // Kurze Pause, um Busy-Waiting zu vermeiden
                }
                Err(e) => {
                    if e.kind() != NotConnected {
                        warn!("Error writing to socket in consumer thread: {:?}", e);
                        CLIENT_CONNECTED.store(false, Relaxed);
                        thread::sleep(Duration::from_millis(50));
                    }
                    return Err(e);
                }
            }
        } else {
            thread::sleep(Duration::from_millis(50)); // Keine Pakete zum Senden, kurze Pause
        }

        if cache.len() > MAX_CASE_SIZE {
            println!("Cache size exceeded: {}", cache.len());
            for _ in 0..MAX_CASE_SIZE / 3 {
                let _ = cache.pop();
            }
        }
    }
    info!("Connection server stopped");
    Ok(())
}

impl Drop for SensorServer {
    fn drop(&mut self) {
        let f = srv_socket_file();
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
