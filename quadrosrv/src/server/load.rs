use crate::proc::history::History;
use crate::sensors::Config;
use log::warn;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub fn load_config(path: PathBuf) -> Option<Config> {
    if path.exists() {
        if let Ok(bytes) = std::fs::read(&path) {
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

pub(crate) fn load_hist(path: PathBuf) -> Option<Vec<HashMap<String, History>>> {
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

pub(crate) fn save_config(c: &Config, path: PathBuf) -> Result<(), std::io::Error> {
    if let Ok(str) = toml::to_string(c) {
        return fs::write(path, str)
    }
    Err(std::io::Error::other("Failed to serialize as toml"))
}
