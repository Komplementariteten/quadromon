use crate::sensors::{Module, SensorConfig, SensorReadResultWrapper};
use glob::{MatchOptions, glob_with};
use regex::{Regex, RegexBuilder};
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

const REGEX_STR: &str = r"\S+/(?<sensor>[\w\d]{2,})\_(?<label>label+)$";
pub const HWMON_CLASS_PATH: &str = "/sys/class/hwmon/";

fn check_module(config: &Module, base_dir: &PathBuf) -> Result<(), String> {
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

fn label_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        RegexBuilder::new(REGEX_STR)
            .case_insensitive(true)
            .multi_line(false)
            .unicode(true)
            .build()
            .expect("Invalid regex")
    })
}

fn check_sensor(config: &SensorConfig, base_path: &str) -> Result<String, String> {
    let re: &Regex = label_regex();
    let glob_opt = MatchOptions {
        case_sensitive: false,
        ..Default::default()
    };

    let glob_path = format!("{}/*_label", base_path);

    for entry in glob_with(glob_path.as_str(), glob_opt).expect("Failed to read glob pattern") {
        if let Ok(path) = &entry {
            let name = fs::read_to_string(path).expect("failed to read file");
            if name.trim() == config.name {
                println!("Found match {:?}", path);
                if let Some(caps) = re.captures(&entry.unwrap().to_str().unwrap())
                    && let Some(match_name) = caps.name("sensor")
                {
                    return Ok(match_name.as_str().to_string());
                }
            }
        }
    }
    Err(format!("{:?} could not be read", config))
}

fn read_sensor(config: &SensorConfig, base_path: &PathBuf) -> Option<SensorReadResultWrapper> {
    if let Ok(s_name) = check_sensor(config, base_path.to_str().unwrap()) {
        let files = config.related_files(&PathBuf::from(base_path), s_name.as_str());
        let mut results = vec![];
        for file in files {
            if let Ok(file_content) = fs::read_to_string(&file) {
                println!(
                    "Reading related file file {:?} with {:?}",
                    file, file_content
                );
                results.push(file_content);
            }
        }

        if results.len() == 0 {
            return None;
        }
        return Some(SensorReadResultWrapper::new(
            config.name.as_str(),
            results,
            config.s_type.clone(),
        ));
    }

    None
}

pub(crate) fn read(module: &Module) -> Vec<SensorReadResultWrapper> {
    let mut results = vec![];
    if let Ok(mod_path) = find_module(module) {
        for sensor in &module.sensors {
            if let Some(result) = read_sensor(sensor, &mod_path) {
                results.push(result)
            }
        }
    }
    results
}

fn find_module(config: &Module) -> Result<PathBuf, String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sensors::{Config, SensorType};

    #[test]
    fn test_find_module() {
        let default = Config::default();
        let first_module = default.modules[0].clone();
        let r = find_module(&first_module);
        assert!(r.is_ok());
    }

    #[test]
    fn test_check_sensor() {
        let cfg = SensorConfig::new("Flow speed [dL/h]", SensorType::FlowSpeed);
        let r = check_sensor(&cfg, "/sys/class/hwmon/hwmon6");
        assert!(r.is_ok());
    }

    #[test]
    fn test_read_sensor() {
        let cfg = SensorConfig::new("Flow speed [dL/h]", SensorType::FlowSpeed);
        let r = read_sensor(&cfg, &PathBuf::from("/sys/class/hwmon/hwmon6"));
        assert!(r.is_some());
    }
}
