mod config;
pub mod sensor_read;
mod sensor_module;

use std::path::PathBuf;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use crate::sensors::sensor_module::Module;

pub const FLOW_SPEED_NAME: &str = "Flow speed [dL/h]";
pub const TEMP_SENSOR_NAME: &str = "Sensor 1";
pub const PUMP_SPEED_NAME: &str = "Pump Fan";
const QUADRO_MODULE: &str = "quadro";

const MAINBOARD_MODULE: &str = "nct6687";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub(crate) modules: Vec<Module>,
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SensorType {
    Temperature,
    InputVoltage,
    FanSpeed,
    Pwm,
    FlowSpeed,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SensorConfig {
    pub name: String,
    pub s_type: SensorType,
}

/// Single Read Result directly form a sensor
#[derive(Debug, Clone)]
pub struct SensorReadResultWrapper {
    _values: Vec<String>,
    pub name: String,
    _t: SensorType,
}

impl SensorReadResultWrapper {
    pub fn new(name: &str, bytes: Vec<String>, t: SensorType) -> SensorReadResultWrapper {
        SensorReadResultWrapper {
            _values: bytes,
            name: name.to_string(),
            _t: t,
        }
    }
    pub fn format(&self) -> ReadResult {
        match self._t {
            SensorType::Temperature => {
                // let bytes: [u8; 4] = self._values[0][0..4].try_into().expect("Failed to read bytes");
                let int_value = self._values[0].trim().parse::<i32>().expect("not a number");
                ReadResult::Temperature(self.name.clone(), int_value)
            }
            SensorType::FlowSpeed => {
                let in_value = self._values[0].trim().parse::<i32>().expect("not a number");
                let pulse_value = self._values[1].trim().parse::<i32>().expect("not a number");
                // let in_bytes: [u8; 4] = self._values[0][0..4].try_into().expect("Failed to read bytes");
                // let pulse_bytes: [u8; 4] = self._values[1][0..4].try_into().expect("Failed to read bytes");
                ReadResult::FlowSpeed(self.name.clone(), in_value, pulse_value)
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

    fn related_files(&self, base_path: &PathBuf, mod_file: &str) -> Vec<PathBuf> {
        let mut related_files = vec![];
        match self.s_type {
            SensorType::FlowSpeed => {
                let input = base_path.join(format!("{}_input", mod_file));
                let pulses = base_path.join(format!("{}_pulses", mod_file));
                related_files.push(input);
                related_files.push(pulses);
            }
            SensorType::FanSpeed => {
                let input = base_path.join(format!("{}_input", mod_file));
                related_files.push(input);
            }
            SensorType::Pwm => {
                let input = base_path.join(format!("{}_input", mod_file));
                related_files.push(input);
            }
            SensorType::Temperature => {
                let input = base_path.join(format!("{}_input", mod_file));
                related_files.push(input);
                let offset = base_path.join(format!("{}_offset", mod_file));
                related_files.push(offset);
            }
            _ => {}
        };
        related_files
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
                    ], None
                ),
                Module::new(
                    MAINBOARD_MODULE,
                    vec![SensorConfig::new(PUMP_SPEED_NAME, SensorType::FanSpeed)], None
                ),
            ],
        }
    }
}
