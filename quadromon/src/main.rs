use std::path::PathBuf;
use std::process::ExitCode;
use std::thread;
use std::thread::Thread;
use log::{error, info};
use quadrosrv::sensors::Config;
use quadrosrv::{consts, server};
use quadrosrv::server::SensorServer;

pub mod ui;

fn main() -> ExitCode {
    let mut cfg = Config::default();
    if let Some(loaded_cfg) = server::load::load_config() {
        cfg = loaded_cfg;
        server::load::restore_cache(&mut cfg);
    } else {
        cfg.init_default();
    }

    let (srv, reader) = SensorServer::start(&cfg);

    thread::sleep(std::time::Duration::from_secs(10));

    let ncf = SensorServer::stop(srv, reader);

    match server::load::save(&ncf) {
        Ok(_) => ExitCode::SUCCESS,
        Err(_) => {
            error!("Failed to save config");
            ExitCode::FAILURE
        }
    }
}
