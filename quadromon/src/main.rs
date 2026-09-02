use log::error;
use quadrosrv::server::{Config, SensorServer};
use quadrosrv::shared::file;
use std::process::ExitCode;
use std::thread;

fn main() -> ExitCode {
    let mut cfg = Config::default();
    if let Some(loaded_cfg) = file::load_config() {
        cfg = loaded_cfg;
        file::restore_cache(&mut cfg);
    } else {
        cfg.init_default();
    }

    let (srv, reader) = SensorServer::start(&cfg);

    thread::sleep(std::time::Duration::from_secs(10));

    let ncf = SensorServer::stop(srv, reader);

    match file::save(&ncf) {
        Ok(_) => ExitCode::SUCCESS,
        Err(_) => {
            error!("Failed to save config");
            ExitCode::FAILURE
        }
    }
}
