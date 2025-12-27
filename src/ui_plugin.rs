use crate::sensors_plugin::Config;
use bevy::color::palettes::css::{LIGHT_GRAY, WHITE};
use bevy::prelude::*;

pub struct QuadroUiPlugin;

const FLOW_SENS_TITLE: &str = "Fluss Sensor";

#[derive(Component)]
struct FlowSenseText;

fn init_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    // commands.spawn((TextBundle::from_section()));
    println!("init ui");
    let default_font = asset_server.load("fonts/SpaceMono-Regular.ttf");
    commands.spawn((
        Text::new(FLOW_SENS_TITLE),
        TextFont {
            font: default_font,
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
}

impl Plugin for QuadroUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_ui);
    }
}
