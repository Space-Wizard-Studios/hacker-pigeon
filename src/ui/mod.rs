use bevy::prelude::*;

mod game_over;
mod health_ui;
mod main_menu;
mod score_ui;

use crate::game_state::GameState;
use game_over::{cleanup_gameover_ui, restart_on_click, setup_gameover_ui};
use health_ui::{setup_health_ui, update_health_ui};
use main_menu::{cleanup_main_menu, setup_main_menu, start_game_on_click};
use score_ui::{setup_score_ui, update_score_ui};

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), setup_main_menu)
            .add_systems(
                Update,
                start_game_on_click.run_if(in_state(GameState::MainMenu)),
            )
            .add_systems(OnExit(GameState::MainMenu), cleanup_main_menu)
            .add_systems(
                OnEnter(GameState::Running),
                (setup_score_ui, setup_health_ui),
            )
            .add_systems(
                Update,
                (update_score_ui, update_health_ui).run_if(in_state(GameState::Running)),
            )
            .add_systems(OnEnter(GameState::GameOver), setup_gameover_ui)
            .add_systems(
                Update,
                restart_on_click.run_if(in_state(GameState::GameOver)),
            )
            .add_systems(OnExit(GameState::GameOver), cleanup_gameover_ui);
    }
}
