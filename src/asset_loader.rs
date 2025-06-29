use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::game_state::GameState;

#[derive(AssetCollection, Resource)]
pub struct ImageAssets {
    #[asset(texture_atlas_layout(tile_size_x = 32, tile_size_y = 32, columns = 4, rows = 1))]
    pub pigeon_fly_sheet_layout: Handle<TextureAtlasLayout>,
    #[asset(path = "pigeon/flying.png")]
    pub pigeon_fly_sheet: Handle<Image>,

    #[asset(path = "world/sky.png")]
    pub bg_sky: Handle<Image>,
    #[asset(path = "world/ground.png")]
    pub bg_ground: Handle<Image>,
    #[asset(path = "world/buildings.png")]
    pub bg_buildings: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct AudioAssets {
    #[asset(path = "audio/badoink_megaman-loop.wav")]
    pub bgm: Handle<AudioSource>,

    #[asset(path = "audio/jsfxr/player/dash_charging.wav")]
    pub dash_charging: Handle<AudioSource>,
    #[asset(path = "audio/jsfxr/player/dash_full_charged.wav")]
    pub dash_full_charged: Handle<AudioSource>,
    #[asset(path = "audio/jsfxr/player/dash_release.wav")]
    pub dash_release: Handle<AudioSource>,

    #[asset(path = "audio/jsfxr/player/drop.wav")]
    pub boom: Handle<AudioSource>,
}

pub struct AssetLoaderPlugin;

impl Plugin for AssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .load_collection::<ImageAssets>()
                .load_collection::<AudioAssets>()
                .continue_to_state(GameState::GameSetup),
        );
    }
}
