use crate::consts::{CFG_FILE, HIST_FILE, QUADRO_DIR};
use crate::proc::history::History;
use crate::sensors::Config;
use log::{info, warn};
use std::collections::HashMap;
use std::path::PathBuf;
use std::{env, fs};

pub fn load_config() -> Option<Config> {
    let cfg_path = base_path().join(CFG_FILE);

    if cfg_path.exists() {
        if let Ok(bytes) = std::fs::read(&cfg_path) {
            if let Ok(c) = toml::from_slice(&bytes) {
                return Some(c);
            } else {
                warn!("Config File could not be deserialized")
            }
        } else {
            warn!("Config File could not be read")
        }
    }
    None
}

pub fn restore_cache(c: &mut Config) {
    let path = base_path().join(HIST_FILE);

    if let Some(maps) = load_hist(&path) {
        for mo in &mut c.modules {
            if let Some(mod_cache) = maps.get(mo.module_name.as_str()) {
                mo.load_processing(mod_cache);
                info!("Module {} restored", mo.module_name)
            }
        }
    }
}

fn load_hist(path: &PathBuf) -> Option<HashMap<String, HashMap<String, History>>> {
    if path.exists() {
        if let Ok(bytes) = std::fs::read(&path) {
            if let Ok(hists) = bitcode::decode(&bytes) {
                return Some(hists);
            } else {
                panic!("History File not be decoded")
            }
        } else {
            panic!("History File could not be read")
        }
    }
    warn!("History file {} does not exist", path.display());
    None
}

pub(crate) fn init_base_dir() {
    let path = base_path();
    if !path.exists() {
        std::fs::create_dir_all(&path).unwrap();
    }
}

pub(crate) fn base_path() -> PathBuf {
    let home_path = match env::home_dir() {
        Some(home) => home,
        None => panic!("Home directory not found"),
    };
    home_path.join(QUADRO_DIR)
}

pub fn save(c: &Config) -> Result<(), std::io::Error> {
    let quadro_path = base_path();

    let cfg_path = quadro_path.join(CFG_FILE);
    let hist_path = quadro_path.join(HIST_FILE);

    let mut ex = HashMap::new();
    for m in &c.modules {
        let mh = m.export_hist();
        ex.insert(m.module_name.clone(), mh);
    }

    if let Ok(str) = toml::to_string(c)
        && let Err(e) = fs::write(cfg_path, str)
    {
        print!("Failed to write config file: {}", e);
        return Err(e);
    }

    let bytes = bitcode::encode(&ex);
    if let Err(err) = std::fs::write(hist_path, bytes) {
        warn!("History File could not be written");
        return Err(err);
    }

    Err(std::io::Error::other("Failed to serialize as toml"))
}
