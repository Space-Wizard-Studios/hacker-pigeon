use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::game_state::GameState;

#[derive(AssetCollection, Resource)]
pub struct ImageAssets {
    #[asset(path = "pigeon/flying/spritesheet.png")]
    pub pigeon_fly_sheet: Handle<Image>,
}

pub struct AssetLoaderPlugin;

impl Plugin for AssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .load_collection::<ImageAssets>()
                .continue_to_state(GameState::GameSetup),
        );
    }
}
