use crate::sensors::{ReadResult, ResultWrapper};
use dirs::{data_local_dir, home_dir};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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
                
                println!("rpm:{rpm}, pulses:{pulses}");
                
                let dlh: f64 = (rpm * 600) as f64 / (pulses as f64);
                Value {
                    value: dlh as f32,
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
                display: display_name.to_string(),
                value: v1 as f32,
                unit: "PWM".to_string(),
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

    #[allow(dead_code)]
    fn new_old(r: ResultWrapper, display_name: &str) -> Self {
        let result = r.format();
        match result {
            ReadResult::FlowSpeed(name, rpm, pulses) => {
                println!("rpm:{rpm}, pulses:{pulses}");

                let dlh: f64 = (rpm * 600) as f64 / (pulses as f64);
                Value {
                    value: dlh as f32,
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

#[derive(Debug, Clone)]
pub(crate) struct Processing {
    hist_file: PathBuf,
    last: Vec<Value>,
    pub hist: Vec<Value>,
}

impl Processing {
    pub(crate) fn init(results: Vec<ResultWrapper>, hist_file: Option<PathBuf>) -> Processing {
        let values: Vec<Value> = results
            .iter()
            .map(|r| Value::new(r.clone(), r.name.as_str()))
            .collect();

        let eff_hist = hist_file.unwrap_or_else(Self::get_default_hist_file);

        let mut p = Processing {
            last: values,
            hist_file: eff_hist,
            hist: vec![],
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
            && let Ok(bytes) = fs::read(&self.hist_file)
        {
            let v: Vec<Value> = postcard::from_bytes(&bytes).expect("Failed to read form binary");
            self.hist = v;
        }
    }

    fn store(&mut self) {
        if let Ok(bytes) = postcard::to_stdvec(&self.hist) {
            fs::write(&self.hist_file, bytes).expect("Failed to write binary");
        } else {
            eprintln!("failed to serialize history to {:?}", self.hist_file);
        }
    }

    pub fn update(&mut self, results: Vec<ResultWrapper>) {
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
                let items = self
                    .hist
                    .iter()
                    .filter(|h| h.source.eq(&v.source))
                    .cloned()
                    .collect::<Vec<_>>();
                new_hist.extend(Self::reduce(items));
            }
            self.hist = Processing::reduce(new_hist);
        }
        self.last.clear();
    }

    #[allow(dead_code)]
    fn update_hist_old(&mut self) {
        if self.last.is_empty() {
            return;
        }

        let latest = self.last.clone();
        if (self.hist.len() + latest.len()) < MAX_HIST_SIZE {
            self.hist.extend(latest);
        } else {
            let mut new_hist = vec![];
            for _v in latest {
                let items = self
                    .hist
                    .iter()
                    .filter(|v| v.source.eq(&v.source))
                    .cloned()
                    .collect::<Vec<_>>();
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
    use crate::sensors::{ResultWrapper, SensorType};

    fn make_value(value: f32, source: &str) -> Value {
        Value {
            value,
            source: source.to_string(),
            unit: "".to_string(),
            value_type: ValueType::Default,
            display: "".to_string(),
        }
    }

    #[test]
    fn reduce_is_correct() {
        let r = Processing::reduce(vec![
            Value {
                value: 1.0,
                source: "a".to_string(),
                unit: "".to_string(),
                value_type: ValueType::Default,
                display: "".to_string(),
            },
            Value {
                value: 2.0,
                source: "a".to_string(),
                unit: "".to_string(),
                value_type: ValueType::Default,
                display: "".to_string(),
            },
            Value {
                value: 3.0,
                source: "b".to_string(),
                unit: "".to_string(),
                value_type: ValueType::Default,
                display: "".to_string(),
            },
            Value {
                value: 4.0,
                source: "b".to_string(),
                unit: "".to_string(),
                value_type: ValueType::Default,
                display: "".to_string(),
            },
        ]);
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].value, 1.5);
        assert_eq!(r[1].value, 3.5);
    }

    #[test]
    fn pwm_value_maps_display_and_unit() {
        let wrapper = ResultWrapper::new("Pump PWM", vec!["128".to_string()], SensorType::Pwm);
        let v = Value::new(wrapper, "Pump Fan");
        assert_eq!(v.value_type, ValueType::Pwm);
        assert_eq!(v.source, "Pump PWM");
        assert_eq!(v.display, "Pump Fan");
        assert_eq!(v.unit, "PWM");
        assert_eq!(v.value, 128.0);
    }

    #[test]
    fn pwm_value_old_swapped_display_and_unit() {
        // Dokumentiert das alte Verhalten: display und unit waren vertauscht.
        let wrapper = ResultWrapper::new("Pump PWM", vec!["128".to_string()], SensorType::Pwm);
        let v = Value::new_old(wrapper, "Pump Fan");
        assert_eq!(v.display, "Pump PWM");
        assert_eq!(v.unit, "Pump Fan");
    }

    #[test]
    fn update_hist_appends_below_limit() {
        let mut p = Processing {
            hist_file: PathBuf::from("/tmp/opencode/quadro-test.hist"),
            last: vec![make_value(1.0, "a")],
            hist: vec![make_value(2.0, "a")],
        };
        p.update_hist();
        assert_eq!(p.hist.len(), 2);
        assert!(p.last.is_empty());
    }

    #[test]
    fn update_hist_downsamples_per_source() {
        let mut hist = vec![];
        for _ in 0..512 {
            hist.push(make_value(2.0, "a"));
        }
        for _ in 0..512 {
            hist.push(make_value(4.0, "b"));
        }
        let mut p = Processing {
            hist_file: PathBuf::from("/tmp/opencode/quadro-test.hist"),
            last: vec![make_value(6.0, "a")],
            hist,
        };
        p.update_hist();

        // 512 Einträge je Quelle -> reduce halbiert zweimal: 256 (per Quelle) -> 128 (final)
        assert_eq!(p.hist.len(), 128);
        assert!(p.hist.iter().all(|v| v.source == "a"));
        assert!(p.hist.iter().all(|v| v.value == 2.0));
    }

    #[test]
    fn update_hist_old_keeps_broken_filter_behavior() {
        // Dokumentiert das alte Verhalten: der Filter verglich v.source mit sich selbst,
        // daher wurden alle Quellen gemeinsam reduziert (1024 -> 512 -> 256).
        let mut hist = vec![];
        for _ in 0..512 {
            hist.push(make_value(2.0, "a"));
        }
        for _ in 0..512 {
            hist.push(make_value(4.0, "b"));
        }
        let mut p = Processing {
            hist_file: PathBuf::from("/tmp/opencode/quadro-test.hist"),
            last: vec![make_value(6.0, "a")],
            hist,
        };
        p.update_hist_old();

        assert_eq!(p.hist.len(), 256);
        assert!(p.hist.iter().any(|v| v.source == "b"));
    }
}
