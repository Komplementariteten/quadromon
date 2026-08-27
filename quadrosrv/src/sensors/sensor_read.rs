use glob::{glob_with, MatchOptions};
use regex::{Regex, RegexBuilder};
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::OnceLock;
use crate::sensors::{Module, ResultWrapper, SensorConfig};

const REGEX_STR: &str = r"\S+/(?<sensor>[\w\d]{2,})\_(?<label>label+)$";
pub const HWMON_CLASS_PATH: &str = "/sys/class/hwmon/";

fn check_module(config: &Module, base_dir: &Path) -> Result<(), String> {
    let name_file = base_dir.join("name");
    if !name_file.exists() {
        return Err(format!("{:?} does not exist", name_file));
    }
    match fs::read_to_string(&name_file) {
        Ok(name) if name.trim() == config.module_name => Ok(()),
        Ok(name) => Err(format!("found wrong module name {:?}", name)),
        Err(e) => Err(format!("{:?} could not be read: {}", name_file, e)),
    }
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
            match fs::read_to_string(path) {
                Ok(name) if name.trim() == config.name => {
                    println!("Found match {:?}", path);
                    if let Some(caps) = re.captures(&path.to_string_lossy())
                        && let Some(match_name) = caps.name("sensor")
                    {
                        return Ok(match_name.as_str().to_string());
                    }
                }
                Err(e) => eprintln!("failed to read file {:?}: {}", path, e),
                _ => {}
            }
        }
    }
    Err(format!("{:?} could not be read", config))
}

fn read_sensor(config: &SensorConfig, base_path: &Path) -> Option<ResultWrapper> {
    if let Ok(s_name) = check_sensor(config, base_path.to_str()?) {
        let files = config.related_files(base_path, s_name.as_str());
        let mut results = vec![];
        for file in files {
            if let Ok(file_content) = fs::read_to_string(&file) {
                results.push(file_content);
            } else {
                eprintln!("failed to read related file {:?}", file);
            }
        }

        if results.is_empty() {
            return None;
        }
        return Some(ResultWrapper::new(
            config.name.as_str(),
            results,
            config.s_type.clone(),
        ));
    }

    None
}

pub fn read(module: &Module) -> Vec<ResultWrapper> {
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
    for class_path in rd.flatten().map(|e| e.path()) {
        if class_path.is_dir() && check_module(config, &class_path).is_ok() {
            return Ok(class_path);
        }
    }
    Err(format!("Module: {:?} not found", config.module_name))
}

#[cfg(test)]
mod tests {
    use crate::sensors::{Config, SensorType};
    use super::*;

    #[test]
    fn test_read() {
        let config = Module::default();
        let r = read(&config);
        assert!(!r.is_empty());
    }

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
