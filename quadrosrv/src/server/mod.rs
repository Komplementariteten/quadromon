pub mod load;

use crate::sensors::Config;
use std::fs::{remove_file, File};
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

const MAX_PACKAGE_SIZE: usize = 1024;

pub struct SensorServer;

#[derive(Debug, Copy, Clone)]
struct Package {
    size: u32,
    c: [u8; MAX_PACKAGE_SIZE],
}

impl Package {}

const DEFAULT_SOCKET: &str = "quadro.sock";

const DEFAULT_DIR: &str = ".quadro/";

static STOP_SYNC: AtomicBool = AtomicBool::new(false);

impl SensorServer {
    fn local_socket() -> PathBuf {
        let path = match dirs::home_dir() {
            Some(path) => path,
            None => panic!("Home dir not set"),
        }
            .join(DEFAULT_DIR);
        if !path.exists() {
            std::fs::create_dir_all(&path).unwrap();
            let fs = path.join(DEFAULT_SOCKET);
            File::create(&fs).unwrap();
            return fs.join(DEFAULT_SOCKET);
        }
        path.join(DEFAULT_SOCKET)
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

    pub fn stop(a: JoinHandle<()>, b: JoinHandle<()>) {
        STOP_SYNC.store(true, Relaxed);
        a.join().expect("failed to join socket");
        b.join().expect("failed to join socket");
    }

    /// Actual Processing of the Socket Connection
    fn server_th(rx: Receiver<Package>) -> JoinHandle<()> {
        thread::spawn(move || {
            let mut cache: Vec<Package> = vec![];
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
                if let Some(l) = cache.pop() {
                    if l.size < 1 {
                        thread::sleep(Duration::from_millis(100));
                        continue;
                    }
                    match socket.write_all(&l.c[..l.size as usize]) {
                        Ok(_) => {
                            println!("package successfully written");
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            cache.push(l); // Paket zurück in den Cache, wenn Socket nicht bereit ist
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
            }
            socket.shutdown(Shutdown::Both).unwrap();
            remove_file(SensorServer::local_socket()).unwrap_or_else(|e| {
                println!("failed to remove socket: {:?}", e);
            });
            println!("Consumer thread finished.");
        })
    }

    /// Triggers Modules for actual sensor reading
    fn reader_thread(tx: Sender<Package>, config: &Config) -> JoinHandle<()> {
        let mut local_cfg = config.clone();
        thread::spawn(move || {
            while !STOP_SYNC.load(Relaxed) {
                for m in &mut local_cfg.modules {
                    m.read();
                }
                let d = [0u8; MAX_PACKAGE_SIZE];
                tx.send(Package { size: MAX_PACKAGE_SIZE as u32, c: d })
                    .expect("Failed to send package from sensor thread"); // Aussagekräftigere Fehlermeldung
                println!("Package sent from sensor thread");
                thread::sleep(Duration::from_millis(500));
            }
            println!("Sensor thread finished.");
        })
    }

    pub fn start(config: &Config) -> (JoinHandle<()>, JoinHandle<()>) {
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
