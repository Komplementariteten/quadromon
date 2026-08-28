use std::collections::HashMap;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use crate::proc::history::History;
use crate::proc::Processing;
use crate::sensors::{sensor_read, SensorConfig};

// Multiple Sensor Modules
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub (crate) struct Module {
    pub (crate) module_name: String,
    pub (crate) sensors: Vec<SensorConfig>,
    #[serde(skip)]
    pub (crate) p: HashMap<String, Processing>,
}


impl Module {
    pub(crate) fn new(name: &str, sensors: Vec<SensorConfig>, hm: Option<HashMap<String, History>>) -> Module {
        let mut pm = HashMap::new();
        for sensor in &sensors {
            let mut p = Processing::new();
            if let Some(map) = &hm {
                p.init(map.get(&sensor.name));
            } 
            pm.insert(sensor.name.clone(), p);
        }
        
        Module {
            module_name: name.to_string(),
            sensors,
            p: pm,
        }
    }

    // Read from all Sensors in Module    
    pub fn read(&mut self) {
        info!("Reading {}", self.module_name);
        let results = sensor_read::read(self);
        debug!("Read {} results", results.len());
        for r in results {
            if let Some(p) = self.p.get_mut(&r.name) {
                p.update(r.format());
            } else {
                warn!("No processing module found for sensor {}", r.name);
            }
        }
    }

}