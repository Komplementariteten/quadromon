use crate::app_config::AppConfig;
use crate::sensors::Config;
use std::fs::{remove_file, File};
use std::io::Write;
use std::net::Shutdown;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

const MAX_PACKAGE_SIZE: usize = 1024;

#[derive(Debug, Copy, Clone)]
struct Package {
    size: u32,
    c: [u8; MAX_PACKAGE_SIZE],
}

impl Package {}

const DEFAULT_SOCKET: &str = "quadro.sock";

const DEFAULT_DIR: &str = ".quadro/";

// Nur noch von den *_old Funktionen genutzt. Der neue Server kapselt das
// Stop-Signal pro Instanz, damit Neustarts und mehrere Instanzen möglich sind.
static STOP_SYNC: AtomicBool = AtomicBool::new(false);

pub struct SensorServer {
    handles: (JoinHandle<()>, JoinHandle<()>),
    stop_flag: Arc<AtomicBool>,
}

impl SensorServer {
    /// Startet den Server mit der Konfiguration aus `.config/quadromon.json`.
    /// Existiert die Datei nicht, wird sie mit einem Beispiel eines vorhandenen
    /// Sensors angelegt.
    pub fn start() -> Self {
        let cfg = AppConfig::load_or_create();
        Self::start_from_app(&cfg)
    }

    pub fn start_from_app(app: &AppConfig) -> Self {
        let sensor_cfg = Config::from_app(app);
        let (tx, rx) = mpsc::channel();
        let stop_flag = Arc::new(AtomicBool::new(false));
        let poll = Duration::from_millis(app.server.poll_interval_ms);

        let sender_h = Self::spawn_server_th(rx, Arc::clone(&stop_flag));
        let sensor_h = Self::spawn_reader_th(tx, sensor_cfg, poll, Arc::clone(&stop_flag));

        SensorServer {
            handles: (sender_h, sensor_h),
            stop_flag,
        }
    }

    #[allow(dead_code)]
    pub fn start_with_config_old(config: &Config) -> Self {
        let (tx, rx) = mpsc::channel();
        let stop_flag = Arc::new(AtomicBool::new(false));
        // altes Verhalten: festes 500ms Intervall
        let poll = Duration::from_millis(500);

        let sender_h = Self::spawn_server_th(rx, Arc::clone(&stop_flag));
        let sensor_h = Self::spawn_reader_th(tx, config.clone(), poll, Arc::clone(&stop_flag));

        SensorServer {
            handles: (sender_h, sensor_h),
            stop_flag,
        }
    }

    pub fn stop(self) {
        self.stop_flag.store(true, Release);
        let (a, b) = self.handles;
        if let Err(e) = a.join() {
            eprintln!("failed to join socket thread: {:?}", e);
        }
        if let Err(e) = b.join() {
            eprintln!("failed to join sensor thread: {:?}", e);
        }
    }

    pub fn local_socket() -> PathBuf {
        let path = match dirs::home_dir() {
            Some(path) => path,
            None => panic!("Home dir not set"),
        }
        .join(DEFAULT_DIR);
        if !path.exists() {
            std::fs::create_dir_all(&path).unwrap();
        }
        path.join(DEFAULT_SOCKET)
    }

    #[allow(dead_code)]
    pub fn local_socket_old() -> PathBuf {
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
                if p.exists() {
                    let _ = remove_file(&p);
                }
                let listener = match UnixListener::bind(&p) {
                    Ok(listener) => listener,
                    Err(e) => {
                        eprintln!("failed to create socket at {:?}: {}", p, e);
                        panic!("failed to create socket: {}", e);
                    }
                };
                let addr = listener.local_addr().unwrap();
                UnixStream::connect_addr(&addr).unwrap_or_else(|e| {
                    eprintln!("Could not connect to socket at {:?} with error: {}", addr, e);
                    panic!("Could not connect to socket with error: {}", e);
                })
            }
        }
    }

    #[allow(dead_code)]
    fn init_socket_old() -> UnixStream {
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

    fn spawn_server_th(rx: Receiver<Package>, stop_flag: Arc<AtomicBool>) -> JoinHandle<()> {
        thread::spawn(move || {
            let mut cache: Vec<Package> = vec![];

            let mut socket = SensorServer::init_socket();
            socket
                .set_nonblocking(true)
                .expect("Failed to set socket non-blocking"); // Socket auf nicht-blockierend setzen

            while !stop_flag.load(Acquire) {
                // Versuche, ein Paket mit Timeout zu empfangen, um nicht ewig zu blockieren
                match rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(package) => {
                        cache.push(package);
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        eprintln!("Consumer thread sender disconnected. Exiting.");
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
                        Ok(_) => {}
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
            socket.shutdown(Shutdown::Both).unwrap_or_else(|e| {
                eprintln!("failed to shutdown socket: {:?}", e);
            });
            remove_file(SensorServer::local_socket()).unwrap_or_else(|e| {
                eprintln!("failed to remove socket: {:?}", e);
            });
        })
    }

    fn spawn_reader_th(
        tx: Sender<Package>,
        config: Config,
        poll: Duration,
        stop_flag: Arc<AtomicBool>,
    ) -> JoinHandle<()> {
        let mut local_cfg = config;
        thread::spawn(move || {
            while !stop_flag.load(Acquire) {
                for m in &mut local_cfg.modules {
                    m.read();
                }
                let d = [0u8; MAX_PACKAGE_SIZE];
                if let Err(e) = tx.send(Package {
                    size: MAX_PACKAGE_SIZE as u32,
                    c: d,
                }) {
                    eprintln!("receiver disconnected, stopping sensor thread: {}", e);
                    break;
                }
                thread::sleep(poll);
            }
        })
    }

    // ---------- alte Implementierung: globales Stop-Flag, Tupel-API ----------

    #[allow(dead_code)]
    pub fn start_old(config: &Config) -> (JoinHandle<()>, JoinHandle<()>) {
        let (tx, rx) = mpsc::channel();
        let sender_h = SensorServer::server_th_old(rx);
        let sensor_h = SensorServer::reader_thread_old(tx, config);
        (sender_h, sensor_h)
    }

    #[allow(dead_code)]
    pub fn stop_old(a: JoinHandle<()>, b: JoinHandle<()>) {
        STOP_SYNC.store(true, Relaxed);
        a.join().expect("failed to join socket");
        b.join().expect("failed to join socket");
    }

    #[allow(dead_code)]
    fn server_th_old(rx: Receiver<Package>) -> JoinHandle<()> {
        thread::spawn(move || {
            let mut cache: Vec<Package> = vec![];

            let mut socket = SensorServer::init_socket();
            socket
                .set_nonblocking(true)
                .expect("Failed to set socket non-blocking");

            while !STOP_SYNC.load(Relaxed) {
                match rx.recv_timeout(Duration::from_millis(100)) {
                    Ok(package) => {
                        cache.push(package);
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        eprintln!("Consumer thread sender disconnected. Exiting.");
                        break;
                    }
                }

                if let Some(l) = cache.pop() {
                    if l.size < 1 {
                        thread::sleep(Duration::from_millis(100));
                        continue;
                    }
                    match socket.write_all(&l.c[..l.size as usize]) {
                        Ok(_) => {}
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            cache.push(l);
                            thread::sleep(Duration::from_millis(5));
                        }
                        Err(e) => {
                            eprintln!("Error writing to socket in consumer thread: {:?}", e);
                            thread::sleep(Duration::from_millis(5));
                        }
                    }
                } else {
                    thread::sleep(Duration::from_millis(5));
                }
            }
            socket.shutdown(Shutdown::Both).unwrap_or_else(|e| {
                eprintln!("failed to shutdown socket: {:?}", e);
            });
            remove_file(SensorServer::local_socket()).unwrap_or_else(|e| {
                eprintln!("failed to remove socket: {:?}", e);
            });
        })
    }

    #[allow(dead_code)]
    fn reader_thread_old(tx: Sender<Package>, config: &Config) -> JoinHandle<()> {
        let mut local_cfg = config.clone();
        thread::spawn(move || {
            while !STOP_SYNC.load(Relaxed) {
                for m in &mut local_cfg.modules {
                    m.read();
                }
                let d = [0u8; MAX_PACKAGE_SIZE];
                tx.send(Package { size: 1024, c: d })
                    .expect("Failed to send package from sensor thread");
                thread::sleep(Duration::from_millis(500));
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::app_config::{AppConfig, ModuleCfg, SensorCfg};
    use crate::sensors::SensorType;
    use crate::server::{SensorServer, DEFAULT_DIR, DEFAULT_SOCKET};
    use std::thread;
    use std::time::Duration;

    fn test_app() -> AppConfig {
        AppConfig {
            server: Default::default(),
            history: Default::default(),
            modules: vec![ModuleCfg {
                module_name: "quadro".to_string(),
                sensors: vec![SensorCfg {
                    name: "Sensor 1".to_string(),
                    s_type: SensorType::Temperature,
                }],
            }],
        }
    }

    #[test]
    fn test_init_and_restart() {
        // Erster Lebenszyklus
        let server = SensorServer::start_from_app(&test_app());
        thread::sleep(Duration::from_millis(1000));
        server.stop();

        // Mit globalem Stop-Flag bliebe ein zweiter Start sofort stehen.
        let server = SensorServer::start_from_app(&test_app());
        thread::sleep(Duration::from_millis(300));
        server.stop();
    }

    #[test]
    fn test_local_socket_path() {
        let path = SensorServer::local_socket();
        assert_eq!(path.file_name().unwrap(), DEFAULT_SOCKET);
        let expected = dirs::home_dir()
            .expect("Home dir not set")
            .join(DEFAULT_DIR);
        assert_eq!(path.parent().unwrap(), expected);
    }
}
