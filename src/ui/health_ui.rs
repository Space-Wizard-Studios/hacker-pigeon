use bevy::color::palettes::tailwind;
use bevy::prelude::*;

use crate::health::Health;
use crate::player::Player;

#[derive(Component)]
pub struct HealthUI;

pub fn setup_health_ui(mut commands: Commands) {
    commands.spawn((
        HealthUI,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(44.),
            left: Val::Px(10.),
            ..default()
        },
        Text::new("Health: 0/0"),
        TextFont {
            font_size: 30.0,
            ..default()
        },
        TextColor(tailwind::GRAY_200.into()),
    ));
}

pub fn update_health_ui(
    mut query_ui: Query<&mut Text, With<HealthUI>>,
    query_health: Query<&Health, With<Player>>,
) {
    if let Ok(mut ui) = query_ui.single_mut() {
        if let Ok(health) = query_health.single() {
            **ui = format!("Health: {}/{}", health.current, health.max);
        }
    }
}
