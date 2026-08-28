use bitcode::{Decode, Encode};
use crate::proc::{Value, ValueType};

#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub(crate) struct History {
    pub values: Vec<f32>,
    pub source: String,
    pub unit: String,
    pub value_type: ValueType,
    size: usize,
    update_ptr: usize,
}

impl History {
    pub fn new(size: usize, source: &str, unit: &str, vt: ValueType) -> Self {
        History {
            values: Vec::with_capacity(size),
            source: source.to_string(),
            unit: unit.to_string(),
            value_type: vt,
            size,
            update_ptr: 0,
        }
    }

    pub(crate) fn update(&mut self, value: Value) {
        if self.values.len() >= self.size {
            let update_value = self.values.remove(self.update_ptr);
            let max = update_value.max(self.values[self.update_ptr]);
            self.values[self.update_ptr] = max;
            self.update_ptr = (self.update_ptr + 1) % self.size;
        }

        self.values.push(value.value);
    }
}