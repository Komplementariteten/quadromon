use crate::client::sensor_dto::SensorDto;
use crate::proc::Processing;
use crate::proc::history::History;
use crate::sensors::{SensorConfig, sensor_read};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Multiple Sensor Modules
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Module {
    pub(crate) module_name: String,
    pub(crate) sensors: Vec<SensorConfig>,
    #[serde(skip)]
    pub(crate) p: HashMap<String, Processing>,
}

impl Module {
    pub(crate) fn new(name: &str, sensors: Vec<SensorConfig>) -> Module {
        let pm = HashMap::new();
        Module {
            module_name: name.to_string(),
            sensors,
            p: pm,
        }
    }

    // Read from all Sensors in Module
    pub fn read(&mut self) -> Vec<SensorDto> {
        let mut dtos = vec![];
        info!("Reading {}", self.module_name);
        let results = sensor_read::read(self);
        debug!("Read {} results", results.len());
        for r in results {
            if let Some(p) = self.p.get_mut(&r.name) {
                p.update(r.format());
            } else {
                let mut p = Processing::new();
                p.update(r.format());
                self.p.insert(r.name.clone(), p);
            }
        }
        for s in &self.sensors {
            if let Some(p) = self.p.get(&s.name)
                && let Some(export) = p.export(&self.module_name)
            {
                dtos.push(export);
            }
        }
        dtos
    }

    pub fn load_processing(&mut self, h: &HashMap<String, History>) {
        self.p.clear();
        for sensor in &self.sensors {
            if let Some(h) = h.get(&sensor.name) {
                let mut p = Processing::new();
                p.init(h);
                self.p.insert(sensor.name.clone(), p);
            }
        }
    }

    pub fn export_hist(&self) -> HashMap<String, History> {
        let mut hm = HashMap::new();
        for (k, v) in &self.p {
            if let Some(hist) = &v.hist {
                hm.insert(k.clone(), hist.clone());
            }
        }
        hm
    }
}
