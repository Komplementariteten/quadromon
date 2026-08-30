use crate::sensors::ReadResult;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub(crate) struct Value {
    pub value: f32,
    pub source: String,
    pub unit: String,
    pub value_type: ValueType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Encode, Decode)]
pub enum ValueType {
    FlowSpeed,
    Temperature,
    FanSpeed,
    Pwm,
    Calculated,
    Default,
}
impl Value {
    pub(crate) fn new(result: ReadResult) -> Self {
        match result {
            ReadResult::FlowSpeed(name, rpm, pulses) => {
                let dlh: f64 = (rpm * 600) as f64 / (pulses as f64);
                Value {
                    value: dlh as f32,
                    source: name,
                    unit: "dL/h".to_string(),
                    value_type: ValueType::FlowSpeed,
                }
            }
            ReadResult::Temperature(name, temp) => {
                let celsius = temp as f32 / 1000.0;
                Value {
                    value: celsius,
                    source: name,
                    unit: "Celsius".to_string(),
                    value_type: ValueType::Temperature,
                }
            }
            ReadResult::FanSpeed(name, fan_value) => {
                let percent: f32 = (fan_value as f32) / 1810.0;
                Value {
                    source: name,
                    value: percent,
                    unit: "Percent".to_string(),
                    value_type: ValueType::FanSpeed,
                }
            }
            ReadResult::Pwm(name, v1, max, _) => {
                let percent: f32 = (v1 as f32) / (max as f32);
                Value {
                    source: name.clone(),
                    value: percent,
                    unit: "PWM".to_string(),
                    value_type: ValueType::Pwm,
                }
            }
            _ => Value {
                value: 0.0,
                source: "".to_string(),
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
