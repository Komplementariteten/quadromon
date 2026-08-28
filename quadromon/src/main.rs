use quadrosrv::sensors::Config;
use quadrosrv::server;
use quadrosrv::server::SensorServer;

mod consts;
pub mod ui;

fn main() {
    //     let plugins = vec![];
    //     ui::run(plugins).expect("Failed to run UI");
    let cfg = &Config::default();
    let (srv,reader) = server::SensorServer::start(cfg);
    
    
    
    SensorServer::stop(srv, reader);
}
