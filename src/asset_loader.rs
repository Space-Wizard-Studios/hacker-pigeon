use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_common_assets::json::JsonAssetPlugin;

use crate::{
    config::{Config, GameConfig},
    game_state::GameState,
};

#[derive(AssetCollection, Resource)]
pub struct ImageAssets {
    #[asset(path = "world/sky.png")]
    pub bg_sky: Handle<Image>,
    #[asset(path = "world/ground.png")]
    pub bg_ground: Handle<Image>,
    #[asset(path = "world/buildings.png")]
    pub bg_buildings: Handle<Image>,

    #[asset(texture_atlas_layout(tile_size_x = 32, tile_size_y = 32, columns = 4, rows = 1))]
    pub pigeon_fly_sheet_layout: Handle<TextureAtlasLayout>,
    #[asset(path = "pigeon/flying.png")]
    pub pigeon_fly_sheet: Handle<Image>,
    #[asset(texture_atlas_layout(tile_size_x = 32, tile_size_y = 32, columns = 4, rows = 1))]
    pub pigeon_drop_sheet_layout: Handle<TextureAtlasLayout>,
    #[asset(path = "pigeon/drop.png")]
    pub pigeon_drop_sheet: Handle<Image>,

    #[asset(texture_atlas_layout(tile_size_x = 32, tile_size_y = 32, columns = 1, rows = 5))]
    pub enemy_drone_layout: Handle<TextureAtlasLayout>,
    #[asset(path = "enemies/drone_ball.png")]
    pub enemy_drone: Handle<Image>,
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

    #[asset(path = "audio/jsfxr/player/hit.wav")]
    pub player_hit: Handle<AudioSource>,
    #[asset(path = "audio/jsfxr/player/death.wav")]
    pub player_death: Handle<AudioSource>,

    #[asset(path = "audio/jsfxr/enemy/hit.wav")]
    pub enemy_hit: Handle<AudioSource>,
    #[asset(path = "audio/jsfxr/enemy/death.wav")]
    pub enemy_death: Handle<AudioSource>,
}

pub struct AssetLoaderPlugin;

impl Plugin for AssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(JsonAssetPlugin::<GameConfig>::new(&["config.json"]))
            .add_loading_state(
                LoadingState::new(GameState::AssetLoading)
                    .load_collection::<ImageAssets>()
                    .load_collection::<AudioAssets>()
                    .continue_to_state(GameState::GameSetup),
            )
            .add_systems(OnExit(GameState::GameSetup), load_config_resource);
    }
}

fn load_config_resource(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    game_configs: Res<Assets<GameConfig>>,
) {
    let game_config = GameConfigHandle(asset_server.load("config/game.config.json"));

    let game_config = if let Some(game) = game_configs.get(game_config.0.id()) {
        game.clone()
    } else {
        GameConfig::default()
    };

    commands.insert_resource(Config { game: game_config });
}

#[derive(Resource)]
struct GameConfigHandle(Handle<GameConfig>);
