use std::fs;
use crate::sensors::{ReadResult, ResultWrapper};
use dirs::{data_local_dir, home_dir};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use vello::wgpu::naga::CollectiveOperation::Reduce;

const DEFAULT_SERIALIZE_FILE: &str = "quadro.hist";
const DOT_SERIALIZE_FILE: &str = ".quadro.hist";
const APP_NAME: &str = "quadromon";

const MAX_HIST_SIZE: usize = 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Value {
    pub value: f32,
    pub source: String,
    pub display: String,
    pub unit: String,
    value_type: ValueType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
enum ValueType {
    FlowSpeed,
    Temperature,
    FanSpeed,
    Pwm,
    Calculated,
    Default,
}
impl Value {
    pub(crate) fn new(r: ResultWrapper, display_name: &str) -> Self {
        let result = r.format();
        match result {
            ReadResult::FlowSpeed(name, rpm, pulses) => {
                let dlh: f32 = (rpm * 600) as f32 / (pulses as f32);
                Value {
                    value: dlh,
                    source: name,
                    display: display_name.to_string(),
                    unit: "dL/h".to_string(),
                    value_type: ValueType::FlowSpeed,
                }
            }
            ReadResult::Temperature(name, temp) => {
                let celsius = temp as f32 / 1000.0;
                Value {
                    value: celsius,
                    source: name,
                    display: display_name.to_string(),
                    unit: "Celsius".to_string(),
                    value_type: ValueType::Temperature,
                }
            }
            ReadResult::FanSpeed(name, fan_value) => {
                let percent: f32 = (fan_value as f32) / 1810.0;
                Value {
                    source: name,
                    value: percent,
                    display: display_name.to_string(),
                    unit: "Percent".to_string(),
                    value_type: ValueType::FanSpeed,
                }
            }
            ReadResult::Pwm(name, v1, _) => Value {
                source: name.clone(),
                display: name,
                value: v1 as f32,
                unit: display_name.to_string(),
                value_type: ValueType::Pwm,
            },
            _ => Value {
                value: 0.0,
                source: "".to_string(),
                display: display_name.to_string(),
                unit: "".to_string(),
                value_type: ValueType::Default,
            },
        }
    }
}

impl Into<f64> for &Value {
    fn into(self) -> f64 {
        self.value as f64
    }
}

pub(crate) struct Processing {
    hist_file: PathBuf,
    last: Vec<Value>,
    pub hist: Vec<Value>,
    // Values to Display
    pub res: Vec<Value>,
}

impl Processing {

    pub(crate) fn new() -> Self {
        Processing {
            hist_file: Self::get_default_hist_file(),
            last: vec![],
            hist: vec![],
            res: vec![],
        }
    }
    pub(crate) fn init(results: Vec<ResultWrapper>, hist_file: Option<PathBuf>) -> Processing {
        let mut values = vec![];
        // ToDo: this crashes
        /* for result in results {
            values.push(Value::new(result, ""));
        }  */

        let eff_hist = match hist_file {
            Some(h) => h,
            _ => Self::get_default_hist_file(),
        };

        let mut p = Processing {
            last: values,
            hist_file: eff_hist,
            hist: vec![],
            res: vec![],
        };
        p.load();
        p.process();
        p
    }

    fn get_default_hist_file() -> PathBuf {
        if let Some(data_dir) = data_local_dir() {
            return data_dir.join(APP_NAME).join(DEFAULT_SERIALIZE_FILE);
        }

        if let Some(home_dir) = home_dir() {
            return home_dir.join(DOT_SERIALIZE_FILE);
        }

        PathBuf::from(DEFAULT_SERIALIZE_FILE)
    }

    fn load(&mut self) {
        if self.hist_file.exists()
            && let Ok(bytes) = std::fs::read(self.hist_file.clone())
        {
            let v: Vec<Value> = postcard::from_bytes(&bytes).expect("Failed to read form binary");
            self.hist = v;
        }
    }

    fn store(&mut self) {
        if let Ok(bytes) = postcard::to_allocvec(&self.hist.clone()) {
            fs::write(self.hist_file.clone(), bytes).expect("Failed to write binary");
        }
    }

    pub(crate) fn update(&mut self, results: Vec<ResultWrapper>) {
        let mut values = vec![];
        for result in results {
            values.push(Value::new(result, ""));
        }
        self.last = values;
        self.process();
        self.store();
    }

    pub(crate) fn process(&mut self) {
        if let Some(flow) = self
            .last
            .iter()
            .find(|v| v.value_type == ValueType::FlowSpeed)
            && let Some(rel_temp) = self.last.iter().find(|v| v.source.eq("Sensor 1"))
        {
            let temp_speed = rel_temp.value / flow.value;
            self.last.push(Value {
                value: temp_speed,
                source: "Temp / Flow".to_string(),
                display: "Temp / Flow".to_string(),
                value_type: ValueType::Calculated,
                unit: "Temp h / dL".to_string(),
            })
        }

        self.update_hist();
    }

    fn reduce(items: Vec<Value>) -> Vec<Value> {
        let mut results = vec![];
        let mut merged: bool = false;
        let mut merge_value: Value = items[0].clone();
        for item in items {
            if !merged {
                merge_value = item.clone();
                merged = true;
            } else {
                let v = merge_value.value;
                merge_value.value = (v + item.value) / 2.0;
                results.push(merge_value.clone());
                merged = false;
            }
        }
        results
    }

    fn update_hist(&mut self) {
        if self.last.is_empty() {
            return;
        }

        let latest = self.last.clone();
        if (self.hist.len() + latest.len()) < MAX_HIST_SIZE {
            self.hist.extend(latest);
        } else {
            let mut new_hist = vec![];
            for v in latest {
                let items = self.hist.iter().filter(|v| v.source.eq(&v.source)).cloned().collect::<Vec<_>>();
                new_hist.extend(Self::reduce(items));
            }
            self.hist = Processing::reduce(new_hist);
        }
        self.last.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reduce_is_correct() {
        let r = Processing::reduce(vec![Value {
            value : 1.0,
            source: "a".to_string(),
            unit: "".to_string(),
            value_type: ValueType::Default,
            display: "".to_string(),
        },Value {
            value : 2.0,
            source: "a".to_string(),
            unit: "".to_string(),
            value_type: ValueType::Default,
            display: "".to_string(),
        }, Value {
            value : 3.0,
            source: "b".to_string(),
            unit: "".to_string(),
            value_type: ValueType::Default,
            display: "".to_string(),
        },Value {
            value : 4.0,
            source: "b".to_string(),
            unit: "".to_string(),
            value_type: ValueType::Default,
            display: "".to_string(),
        }]);
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].value, 1.5);
        assert_eq!(r[1].value, 3.5);
    }
}