use bevy::prelude::*;

#[derive(States, Clone, Default, Debug, Hash, PartialEq, Eq)]
pub enum GameState {
    #[default]
    AssetLoading,
    GameRunning,
    GameOver,
}
