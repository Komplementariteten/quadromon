pub(crate) mod history;
pub mod proc_values;

use crate::sensors::{ReadResult};
use crate::proc::history::History;
use crate::proc::proc_values::{Value, ValueType};

const MAX_HIST_SIZE: usize = 1024;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Processing {
    pub last: Option<Value>,
    hist: Option<History>,
}

impl Processing {
    pub(crate) fn new() -> Self {
        Processing {
            last: None,
            hist: None,
        }
    }
    
    pub(crate) fn init(&mut self, h: Option<&History>) {
        if let Some(h) = h {
            self.hist = Some(h.clone());
        }
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
        if let Some(last) = self.last.clone() {
            
            // Initialize History
            if self.hist.is_none(){
                self.hist = Some(History::new(MAX_HIST_SIZE, &last.source, &last.unit, last.value_type.clone()));
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
    #[test]
    fn reduce_is_correct() {
        let r = Processing::reduce(vec![
            Value {
                value: 1.0,
                source: "a".to_string(),
                unit: "".to_string(),
                value_type: ValueType::Default,
            },
            Value {
                value: 2.0,
                source: "a".to_string(),
                unit: "".to_string(),
                value_type: ValueType::Default,
            },
            Value {
                value: 3.0,
                source: "b".to_string(),
                unit: "".to_string(),
                value_type: ValueType::Default,
            },
            Value {
                value: 4.0,
                source: "b".to_string(),
                unit: "".to_string(),
                value_type: ValueType::Default,
            },
        ]);
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].value, 1.5);
        assert_eq!(r[1].value, 3.5);
    }
}
