use bevy::color::palettes::tailwind;
use bevy::prelude::*;

use crate::game_state::GameState;

#[derive(Component)]
pub struct MainMenuUI;

pub fn setup_main_menu(mut commands: Commands) {
    commands.spawn((
        MainMenuUI,
        Node {
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            width: Val::Percent(100.),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(tailwind::GRAY_900.into()),
        children![start_button()],
    ));
}

fn start_button() -> impl Bundle + use<> {
    (
        Button,
        Node {
            width: Val::Px(300.0),
            height: Val::Px(65.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(tailwind::BLUE_700.into()),
        children![(
            Text::new("Start Game"),
            TextFont {
                font_size: 40.0,
                ..default()
            },
            TextColor(tailwind::GRAY_200.into()),
        )],
    )
}

pub fn start_game_on_click(
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::Running);
        }
    }
}

pub fn cleanup_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenuUI>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
