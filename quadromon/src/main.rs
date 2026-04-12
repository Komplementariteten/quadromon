use crate::ui::sensor_handle::SensorHandle;

mod sensors;
mod consts;
pub mod ui;
mod proc;

fn main() {
    let plugins = vec![SensorHandle::new()];
    ui::run(plugins).expect("Failed to run UI");
}
