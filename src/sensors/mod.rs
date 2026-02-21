mod config;
mod sensor_read;

use std::path::PathBuf;
use crate::sensors::sensor_read::ReadResult;
use crate::ui::EventDrivenPlugin;

pub const FLOW_SPEED_NAME: &str = "Flow speed [dL/h]";
pub const TEMP_SENSOR_NAME: &str = "Sensor 1";
pub const PUMP_SPEED_NAME: &str = "Pump Fan";
const QUADRO_MODULE: &str = "quadro";

const MAINBOARD_MODULE: &str = "nct6687";

pub(crate) struct SensorHandle {
}

impl EventDrivenPlugin for SensorHandle {
    fn event_tick(&mut self) -> anyhow::Result<()>
    {
        todo!()
    }
}

#[derive(Debug)]
pub struct SensorEvent{
    pub sensor_name: String,
    pub sensor_value: String
}

impl From<ReadResult> for SensorEvent{
    fn from(r: ReadResult) -> Self {
        let value = r.get_value();
        println!("Sensor value: {}:{}", &r.name, &value);
        SensorEvent{
            sensor_name: r.name,
            sensor_value: value
        }
    }
}

#[derive(Debug)]
pub struct Config {
    pub modules: Vec<Module>,
}

#[derive(Debug)]
pub struct Module {
    pub module_name: String,
    pub sensors: Vec<SensorConfig>,
}

#[derive(Debug)]
pub struct SensorConfig {
    pub name: String,
    pub file: PathBuf,
}

impl SensorConfig {
    fn new(name: &str, file: &str) -> SensorConfig {
        SensorConfig {
            name: name.to_string(),
            file: PathBuf::from(file),
        }
    }
}

impl Module {
    fn new(name: &str, sensors: Vec<SensorConfig>) -> Module {
        Module {
            module_name: name.to_string(),
            sensors,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            modules: vec![
                Module::new(
                    QUADRO_MODULE,
                    vec![
                        SensorConfig::new(FLOW_SPEED_NAME, "fan5_input"),
                        SensorConfig::new(TEMP_SENSOR_NAME, "temp1_input"),
                    ],
                ),
                Module::new(
                    MAINBOARD_MODULE,
                    vec![SensorConfig::new(PUMP_SPEED_NAME, "fan2_input")],
                ),
            ],
        }
    }
}

/* fn sensor_event(
    time: Res<Time>,
    mut state: ResMut<SensorTimer>,
    config: Res<Config>,
    mut sensor_ev: MessageWriter<SensorEvent>,
) {
    if state.tick(time.delta()).is_finished() {
        println!("Read Sensor");
        let rs = sensor_read::read(&config);
        for result in rs {
            sensor_ev.write(SensorEvent::from(result));
        }
    }
}

impl Plugin for SensorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SensorTimer>()
            .init_resource::<Config>()
            .add_message::<SensorEvent>()
            .add_systems(Update, sensor_event);
    }
}
*/