use bevy::color::palettes::tailwind;
use bevy::prelude::*;

use crate::{game_state::GameState, score::Score};

#[derive(Component)]
pub struct GameOverUI;

pub fn setup_gameover_ui(mut commands: Commands, score: Res<Score>) {
    commands.spawn((
        GameOverUI,
        Node {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(16.),
            padding: UiRect::all(Val::Px(16.)),
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            ..default()
        },
        BackgroundColor(tailwind::GRAY_900.with_alpha(0.8).into()),
        children![
            (
                Text::new("GAME OVER"),
                TextFont {
                    font_size: 40.0,
                    ..default()
                },
                TextColor(tailwind::GRAY_200.into()),
            ),
            (
                Text::new(format!("Final Score: {}", score.0)),
                TextFont {
                    font_size: 40.0,
                    ..default()
                },
                TextColor(tailwind::GRAY_200.into()),
            ),
            (
                Button,
                Node {
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(8.0)),
                    margin: UiRect::top(Val::Auto),
                    ..default()
                },
                BackgroundColor(tailwind::BLUE_700.into()),
                children![(
                    Text::new("Click to Play Again"),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(tailwind::GRAY_200.into()),
                )],
            )
        ],
    ));
}

pub fn restart_on_click(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::Running);
        }
    }
}

pub fn cleanup_gameover_ui(mut commands: Commands, query: Query<Entity, With<GameOverUI>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
