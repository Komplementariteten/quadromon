use std::fs;
use std::path::Path;
use bevy::camera::RenderTarget;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use serde::{Deserialize, Serialize};
use crate::sensors_plugin::SensorPlugin;
use crate::ui_plugin::QuadroUiPlugin;

const WINDOW_JSON: &str = ".cfg.json";

pub struct AppPlugin;

#[derive(Resource, Debug)]
enum LeftClickAction {
    Nothing,
    Move,
}

#[derive(Serialize, Deserialize, Debug)]
struct WindowState {
    x: i32,
    y: i32,
    width: f32,
    height: f32,
}


impl Default for WindowState {
    fn default() -> Self {
        Self {x: 100, y: 100, width: 200.0, height: 400.0}
    }
}

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LeftClickAction::Nothing);

        app.add_systems(Startup, setup).add_systems(Update, (move_window, handle_input));
        app.add_plugins(SensorPlugin).add_plugins(QuadroUiPlugin);
    }
}

// Hilfsfunktion: Laden
fn load_window_config() -> WindowState {
    if Path::new(WINDOW_JSON).exists() {
        if let Ok(content) = fs::read_to_string(WINDOW_JSON) {
            if let Ok(state) = serde_json::from_str(&content) {
                println!("Konfiguration geladen: {:?}", state);
                return state;
            }
        }
    }
    println!("Keine Config gefunden, nutze Standards.");
    WindowState::default()
}


fn move_window(mut windows: Query<&mut Window>, action: Res<LeftClickAction>, input: Res<ButtonInput<MouseButton>>) {

    if input.just_pressed(MouseButton::Left) {
        for mut w in windows.iter_mut() {
            match *action {
                LeftClickAction::Nothing => (),
                LeftClickAction::Move => {
                    let p = w.position.clone();
                    println!("{:?}", p);
                    w.start_drag_move();
                }
            }
        }
    }
}

fn handle_input(input: Res<ButtonInput<KeyCode>>, mut action: ResMut<LeftClickAction>) -> Result {

    if input.pressed(KeyCode::KeyM) {
        *action = LeftClickAction::Move;
    } else {
        *action = LeftClickAction::Nothing;
    }

    Ok(())
}

fn setup(mut commands: Commands) {
    let backend = std::env::var("WINIT_UNIX_BACKEND").unwrap_or("Nicht gesetzt".to_string());
    println!("WINIT_UNIX_BACKEND ist: {}", backend);
    println!("app setup");
    commands.spawn(Camera2d);
}