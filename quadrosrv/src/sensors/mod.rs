pub mod sensor_read;

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::proc::Processing;
use crate::app_config::{AppConfig, ModuleCfg, SensorCfg};

pub const FLOW_SPEED_NAME: &str = "Flow speed [dL/h]";
pub const TEMP_SENSOR_NAME: &str = "Sensor 1";
pub const PUMP_SPEED_NAME: &str = "Pump Fan";
const QUADRO_MODULE: &str = "quadro";

const MAINBOARD_MODULE: &str = "nct6687";

#[derive(Debug, Clone)]
pub struct Config {
    pub modules: Vec<Module>,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub module_name: String,
    pub sensors: Vec<SensorConfig>,
    pub(crate) p: Option<Processing>,
    hist_max: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SensorType {
    Temperature,
    InputVoltage,
    FanSpeed,
    Pwm,
    FlowSpeed,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct SensorConfig {
    pub name: String,
    pub s_type: SensorType,
}

#[derive(Debug, Clone)]
pub struct ResultWrapper {
    _values: Vec<String>,
    pub name: String,
    _t: SensorType,
}

impl ResultWrapper {
    pub fn new(name: &str, bytes: Vec<String>, t: SensorType) -> ResultWrapper {
        ResultWrapper {
            _values: bytes,
            name: name.to_string(),
            _t: t,
        }
    }
    pub fn format(&self) -> ReadResult {
        match self._t {
            SensorType::Temperature => {
                let int_value = self._values[0].trim().parse::<i32>().expect("not a number");
                ReadResult::Temperature(self.name.clone(), int_value)
            },
            SensorType::FlowSpeed => {
                let in_value = self._values[0].trim().parse::<i32>().expect("not a number");
                let pulse_value = self._values[1].trim().parse::<i32>().expect("not a number");
                ReadResult::FlowSpeed(self.name.clone(), in_value, pulse_value)
            }
            SensorType::FanSpeed => {
                let fan_value = self._values[0].trim().parse::<i32>().expect("not a number");
                ReadResult::FanSpeed(self.name.clone(), fan_value)
            }
            SensorType::Pwm => {
                let v1 = self._values[0].trim().parse::<i32>().expect("not a number");
                ReadResult::Pwm(self.name.clone(), v1, 0)
            }
            _ => ReadResult::None,
        }
    }
}

pub enum ReadResult {
    Temperature(String, i32),
    InputVoltage(String, i32),
    FanSpeed(String, i32),
    Pwm(String, i32, i32),
    FlowSpeed(String, i32, i32),
    None,
}

impl SensorConfig {
    fn new(name: &str, t: SensorType) -> SensorConfig {
        SensorConfig {
            name: name.to_string(),
            s_type: t,
        }
    }

    fn related_files(&self, base_path: &Path, mod_file: &str) -> Vec<PathBuf> {
        let mut related_files = vec![base_path.join(format!("{}_input", mod_file))];
        match self.s_type {
            SensorType::FlowSpeed => {
                let pulses = base_path.join(format!("{}_pulses", mod_file));
                related_files.push(pulses);
            }
            SensorType::Temperature => {
                let offset = base_path.join(format!("{}_offset", mod_file));
                related_files.push(offset);
            }
            _ => {}
        };
        related_files
    }
}

impl Module {
    pub(crate) fn new(name: &str, sensors: Vec<SensorConfig>) -> Module {
        Module {
            module_name: name.to_string(),
            p: None,
            hist_max: crate::proc::MAX_HIST_SIZE,
            sensors,
        }
    }

    pub(crate) fn from_cfg(cfg: &ModuleCfg, hist_max: usize) -> Module {
        let mut m = Module::new(
            cfg.module_name.as_str(),
            cfg.sensors.iter().map(SensorConfig::from_cfg).collect(),
        );
        m.hist_max = hist_max;
        m
    }

    pub fn read(&mut self) {
        let r = sensor_read::read(self);
        if self.p.is_none() {
            let mut p = Processing::init(r.clone(), None);
            p.set_max_hist(self.hist_max);
            self.p = Some(p)
        } else if let Some(p) = self.p.as_mut() {
            p.set_max_hist(self.hist_max);
            p.update(r);
        }
    }
}

impl Config {
    pub fn from_app(app: &AppConfig) -> Config {
        Config {
            modules: app
                .modules
                .iter()
                .map(|m| Module::from_cfg(m, app.history.max_size))
                .collect(),
        }
    }
}

impl SensorConfig {
    pub(crate) fn from_cfg(cfg: &SensorCfg) -> SensorConfig {
        SensorConfig {
            name: cfg.name.clone(),
            s_type: cfg.s_type.clone(),
        }
    }
}

impl Default for Module {
    fn default() -> Self {
        Module::new("default", Vec::new())
    }
}


impl Default for Config {
    fn default() -> Self {
        Self {
            modules: vec![
                Module::new(
                    QUADRO_MODULE,
                    vec![
                        SensorConfig::new(FLOW_SPEED_NAME, SensorType::FlowSpeed),
                        SensorConfig::new(TEMP_SENSOR_NAME, SensorType::Temperature),
                    ],
                ),
                Module::new(
                    MAINBOARD_MODULE,
                    vec![SensorConfig::new(PUMP_SPEED_NAME, SensorType::FanSpeed)],
                ),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_config::{AppConfig, SensorCfg};

    #[test]
    fn from_app_maps_modules_and_sensors() {
        let app = AppConfig {
            history: crate::app_config::HistoryCfg { max_size: 77 },
            ..AppConfig::default()
        };
        // Beispiel erzeugen, damit Module vorhanden sind
        let mut app = app;
        app.modules = vec![crate::app_config::ModuleCfg {
            module_name: "testmod".to_string(),
            sensors: vec![SensorCfg {
                name: "Sensor 1".to_string(),
                s_type: SensorType::Temperature,
            }],
        }];

        let mut cfg = Config::from_app(&app);
        assert_eq!(cfg.modules.len(), 1);
        assert_eq!(cfg.modules[0].module_name, "testmod");
        assert_eq!(cfg.modules[0].sensors[0].name, "Sensor 1");
        assert_eq!(cfg.modules[0].sensors.len(), 1);

        // hist_max muss aus der Config uebernommen werden
        cfg.modules[0].read();
        if let Some(p) = &cfg.modules[0].p {
            assert_eq!(p.max_hist, 77);
        } else {
            panic!("Processing should be initialized after read");
        }
    }

    #[test]
    fn default_config_uses_default_hist_size() {
        let m = Module::default();
        assert_eq!(m.hist_max, crate::proc::MAX_HIST_SIZE);
    }
}
