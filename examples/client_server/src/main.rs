use std::thread;
use quadrosrv::client::Client;
use quadrosrv::sensors::Config;
use quadrosrv::server::SensorServer;
use quadrosrv::shared::file;
use quadrosrv::shared::logger::init_logger;

fn main() {
    init_logger();
    let mut cfg = Config::default();
    if let Some(loaded_cfg) = file::load_config() {
        cfg = loaded_cfg;
        file::restore_cache(&mut cfg);
    } else {
        cfg.init_default();
    }

    cfg.verbose = true;
    let (srv, reader) = SensorServer::start(&cfg);

    /// thread::sleep(std::time::Duration::from_secs(10));
    let mut c = Client::new();

    for _ in 0..1000 {
        if let Some(d) = c.read() {
            println!("{:?}", d);
        }
        thread::sleep(std::time::Duration::from_millis(20));
    }

    let ncf = SensorServer::stop(srv, reader);

    match file::save(&ncf) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Failed to save config {}", e);
        }
    }
    println!("Hello, world!");
}
