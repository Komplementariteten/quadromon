use bevy::prelude::*;
use crate::sensors_plugin::SensorPlugin;
use crate::ui_plugin::QuadroUiPlugin;

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
     //   app.add_systems(Startup, setup);
        app.add_plugins(SensorPlugin).add_plugins(QuadroUiPlugin);
    }
}

fn setup(mut commands: Commands) {
    println!("app setup");
    commands.spawn(Camera2d);
}