use bevy::color::palettes::tailwind;
use bevy::prelude::*;

use crate::score::Score;

#[derive(Component)]
pub struct ScoreUI;

pub fn setup_score_ui(mut commands: Commands) {
    commands.spawn((
        ScoreUI,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.),
            left: Val::Px(10.),
            ..default()
        },
        Text::new("Score: 0"),
        TextFont {
            font_size: 30.0,
            ..default()
        },
        TextColor(tailwind::GRAY_200.into()),
    ));
}

pub fn update_score_ui(score: Res<Score>, mut query_ui: Query<&mut Text, With<ScoreUI>>) {
    if score.is_changed() {
        if let Ok(mut ui) = query_ui.single_mut() {
            **ui = format!("Score: {}", score.0);
        }
    }
}
