pub(crate) mod history;
pub mod proc_values;

use crate::client::sensor_dto::SensorDto;
use crate::proc::history::History;
use crate::proc::proc_values::{Value, ValueType};
use crate::sensors::ReadResult;

const MAX_HIST_SIZE: usize = 1024;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Processing {
    pub(crate) last: Option<Value>,
    pub(crate) hist: Option<History>,
}

impl Processing {
    pub(crate) fn new() -> Self {
        Processing {
            last: None,
            hist: None,
        }
    }

    pub(crate) fn init(&mut self, h: &History) {
        self.hist = Some(h.clone());
    }

    pub fn export(&self, n: &str) -> Option<SensorDto> {
        if self.last.is_none() {
            return None;
        }
        let l = self.last.clone().unwrap();
        Some(SensorDto {
            name: n.to_string(),
            current: l.value,
            unit: l.unit,
            values: self.hist.clone().unwrap().values,
            module: n.to_string(),
        })
    }

    pub fn update(&mut self, result: ReadResult) {
        let v = Value::new(result);
        self.last = Some(v);
        self.process();
    }

    fn process(&mut self) {
        // handle Flow values
        if let Some(flow) = self
            .last
            .iter()
            .find(|v| v.value_type == ValueType::FlowSpeed)
            && let Some(rel_temp) = self.last.iter().find(|v| v.source.eq("Sensor 1"))
        {
            let temp_speed = rel_temp.value / flow.value;
            self.last = Some(Value {
                value: temp_speed,
                source: "Temp / Flow".to_string(),
                value_type: ValueType::Calculated,
                unit: "Temp h / dL".to_string(),
            })
        }

        self.update_hist();
    }

    fn update_hist(&mut self) {
        if let Some(last) = self.last.clone() {
            // Initialize History
            if self.hist.is_none() {
                self.hist = Some(History::new(
                    MAX_HIST_SIZE,
                    &last.source,
                    &last.unit,
                    last.value_type.clone(),
                ));
            }

            if let Some(hist) = &mut self.hist {
                hist.update(last);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
