use crate::sensors::SensorHandle;
use crate::ui::EventDrivenPlugin;

mod sensors;
mod consts;
mod ui;

fn main() {
    let plugins = vec![SensorHandle { }];
    ui::run(plugins).expect("Failed to run UI");
}
