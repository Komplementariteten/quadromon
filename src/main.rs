mod app_plugin;
mod sensors_plugin;
mod ui_plugin;

use bevy::app::App;
use bevy::DefaultPlugins;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use crate::app_plugin::AppPlugin;

fn main() {
    App::new().add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            resolution: WindowResolution::new(400, 600),
            decorations: false,
            title: "Quadromon".to_string(),
            position: WindowPosition::Automatic,
            ..default()
        }),
        ..default()
    })).add_plugins(AppPlugin).run();
}
