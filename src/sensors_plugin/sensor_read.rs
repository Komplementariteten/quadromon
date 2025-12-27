use crate::sensors_plugin::{Config, Module, SensorConfig};
use regex::{Regex, RegexBuilder};
use std::fs;
use std::path::PathBuf;

const REGEX_STR: &str = r"^(?<sensor>[\w\d]{2,})\_(?<input>input+)$";
pub(crate) const HWMON_CLASS_PATH: &str = "/sys/class/hwmon/";

#[derive(Debug)]
pub(crate) struct ReadResult {
    _bytes: Vec<u8>,
    pub name: String,
}

impl ReadResult {
    pub fn new(name: &str, bytes: Vec<u8>) -> ReadResult {
        ReadResult {
            _bytes: bytes,
            name: name.to_string(),
        }
    }
    
    pub fn get_value(&self) -> String {
        if let Ok(s) = String::from_utf8(self._bytes.clone()) {
            return s.trim().to_string();
        }
        format!("{:x?}", self._bytes.clone()).trim().to_string()
    }
}
fn check_module(config: &Module, base_dir: &PathBuf) -> bevy::prelude::Result<(), String> {
    if !base_dir.exists() {
        return Err(format!("{:?} does not exist", base_dir));
    }

    let name_file = base_dir.join("name");
    if !name_file.exists() {
        return Err(format!("{:?} does not exist", name_file));
    }
    if let Ok(found_mod_name) = fs::read_to_string(&name_file) {
        return match found_mod_name.trim() == config.module_name {
            true => Ok(()),
            false => Err(format!("found wrong module name {:?}", found_mod_name)),
        };
    }

    Err(format!("{:?} could not be read", name_file))
}

fn check_sensor(
    config: &SensorConfig,
    base_path: &PathBuf,
    re: &Regex,
) -> bevy::prelude::Result<(), String> {
    if let None = re.captures(config.file.to_str().unwrap()) {
        return Err(format!(
            "{:?} sensor file does not have the expected format abc123_input",
            config.file
        ));
    }

    if !&base_path.join(&config.file).exists() {
        return Err(format!("{:?} does not exist", base_path.join(&config.file)));
    }

    let labele_file_name = re
        .replace_all(config.file.to_str().unwrap(), "${sensor}_label")
        .to_string();
    let label_file = base_path.join(labele_file_name);
    if !label_file.exists() {
        return Err(format!("{:?} does not exist", label_file));
    }
    if let Ok(found_label) = fs::read_to_string(&label_file) {
        return match found_label.trim() == config.name {
            true => Ok(()),
            false => Err(format!("found wrong sensor name {:?}", found_label)),
        };
    }

    Err(format!("{:?} could not be read", config))
}

fn read_sensor(config: &SensorConfig, base_path: &PathBuf) -> Option<ReadResult> {
    let sensor_file = base_path.join(&config.file);
    if let Ok(file_content) = fs::read(&sensor_file) {
        return Some(ReadResult::new(&config.name, file_content));
    }
    None
}

pub(crate) fn read(config: &Config) -> Vec<ReadResult> {
    let lable_re = RegexBuilder::new(REGEX_STR)
        .case_insensitive(true)
        .multi_line(false)
        .unicode(true)
        .build()
        .expect("Invalid regex");

    let mut results = vec![];
    for module in &config.modules {
        if let Ok(mod_path) = find_module(module) {
            for sensor in &module.sensors {
                if let Err(e_str) = check_sensor(sensor, &mod_path, &lable_re) {
                    panic!("{}", e_str)
                }
                if let Some(result) = read_sensor(sensor, &mod_path) {
                    results.push(result)
                }
            }
        }
    }
    results
}

fn find_module(config: &Module) -> bevy::prelude::Result<PathBuf, String> {
    let rd = fs::read_dir(HWMON_CLASS_PATH).expect("HWMON Class Dir not found");
    for entry in rd {
        let class_path = entry.unwrap().path();
        if class_path.is_dir() {
            match check_module(config, &class_path) {
                Ok(_) => return Ok(class_path),
                Err(_) => continue,
            }
        }
    }
    Err(format!("Module: {:?} not found", config.module_name))
}
