use bevy::prelude::*;

#[derive(States, Clone, Default, Debug, Hash, PartialEq, Eq)]
pub enum GameState {
    #[default]
    AssetLoading,
    GameSetup,
    GameRunning,
    GameOver,
}
