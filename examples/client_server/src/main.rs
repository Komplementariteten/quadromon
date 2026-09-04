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

    // cfg.verbose = true;
    let (srv, reader) = SensorServer::start(&cfg);

    /// thread::sleep(std::time::Duration::from_secs(10));
    let mut c = Client::new();

    let mut read_count = 0;

    while read_count < 100 {
        if let Some(_) = c.read() {
            read_count += 1;
            println!("Data read {read_count}");
        };
        
        thread::sleep(std::time::Duration::from_millis(10));
    }
    c.stop();

    println!("Trying to stop server");
    let _ = SensorServer::stop(srv, reader);
    println!("Server stopped");
}
