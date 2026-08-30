use bitcode::{Decode, Encode};

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub struct SensorDto {
    pub current: f32,
    pub unit: String,
    pub values: Vec<f32>,
    pub name: String,
    pub module: String,
}
