use std::collections::BTreeMap;
use crate::sensors_plugin::{Config, SensorEvent, FLOW_SPEED_NAME};
use bevy::color::palettes::css::{LIGHT_GRAY, WHITE};
use bevy::prelude::*;

pub struct QuadroUiPlugin;

const FLOW_SENS_TITLE: &str = "Fluss Sensor";

const FLOW_VALUE_POS: u32 = 50;

#[derive(Resource, Default)]
struct DataHistory {
    pub max: BTreeMap<String, String>,
    pub v: BTreeMap<String, Vec<String>>
}

#[derive(Component)]
struct FlowSenseText;

#[derive(Component)]
struct FlowSenseValues;

fn draw_graph(mut gizmos: Gizmos, data: Res<DataHistory>) {
    // --- Konfiguration des Graphen ---
    let window_width = 200.0;
    let window_height = 400.0;
    let padding = 20.0;

    // Defines the bounds of the drawing area (Centered origin at 0,0)
    let left = -window_width / 2.0 + padding;
    let right = window_width / 2.0 - padding;
    let bottom = -window_height / 2.0 + padding;
    let top = window_height / 2.0 - padding;

    // --- 1. Achsen zeichnen (Weiß) ---
    // Y-Achse (Links)
    gizmos.line_2d(Vec2::new(left, bottom), Vec2::new(left, top), Color::WHITE);
    // X-Achse (Unten)
    gizmos.line_2d(Vec2::new(left, bottom), Vec2::new(right, bottom), Color::WHITE);

    // --- 2. Daten zeichnen (Grün) ---
    if data.v.is_empty() { return; }

    let count = data.v.len();
    let step_x = (right - left) / (count as f32 - 1.0);

    // Wir iterieren durch die Punkte und zeichnen eine Linie zum jeweils nächsten Punkt
    for i in 0..count - 1 {
        let val_current = data.v[i];
        let val_next = data.v[i + 1];

        // Mapping: Wert (-1.0 bis 1.0) auf Pixelhöhe (bottom bis top) umrechnen
        // Formel: pos = bottom + (wert - min) / (max - min) * hoehe
        // Hier vereinfacht für Sinus (-1 bis 1):
        let y_current = bottom + (val_current + 1.0) / 2.0 * (top - bottom);
        let y_next = bottom + (val_next + 1.0) / 2.0 * (top - bottom);

        let x_current = left + (i as f32 * step_x);
        let x_next = left + ((i + 1) as f32 * step_x);

        gizmos.line_2d(
            Vec2::new(x_current, y_current),
            Vec2::new(x_next, y_next),
            Color::srgb(0.0, 1.0, 0.0), // Grün
        );
    }
}

fn update_values(mut commands: Commands, mut hist: ResMut<DataHistory>, mut msg_receiver: MessageReader<SensorEvent>, mut query: Query<&mut Text, With<FlowSenseValues>>) {
    for sensor_event in msg_receiver.read() {
        if sensor_event.sensor_name.eq(FLOW_SPEED_NAME) {
            return;
        } 
        let v = sensor_event.sensor_value.clone();
        println!("update values: {}", &v);
        for mut text in &mut query {
            text.0 = v.clone();
            hist.max.entry(sensor_event.sensor_name.clone()).and_modify(|v| *v = v.trim().to_string()).or_insert(v.clone());
        }
    }
}

fn init_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    // commands.spawn((TextBundle::from_section()));
    println!("init ui");
    let default_font = asset_server.load("fonts/SpaceMono-Regular.ttf");
    commands.spawn((
        Text::new(FLOW_SENS_TITLE),
        TextFont {
            font: default_font.clone(),
            font_size: 26.,
            ..default()
        },
        TextLayout::new_with_justify(Justify::Left),
        TextColor(LIGHT_GRAY.into()),
        Node {
            position_type: PositionType::Absolute,
            top: px(8),
            left: px(8),
            ..default()
        },
        FlowSenseText,
    ));

    commands.spawn((
        Text::new(format!("{:2}", 0.00)),
        TextFont {
            font: default_font,
            font_size: 26.,
            ..default()
        },
        TextLayout::new_with_justify(Justify::Left),
        TextColor(LIGHT_GRAY.into()),
        Node {
            position_type: PositionType::Relative,
            top: px(FLOW_VALUE_POS),
            left: px(8),
            ..default()
        },
        FlowSenseValues,
    ));
}

impl Plugin for QuadroUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DataHistory>().add_systems(Startup, init_ui).add_systems(Update, update_values);
    }
}
