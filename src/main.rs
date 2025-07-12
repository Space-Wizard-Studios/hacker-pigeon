#![allow(clippy::type_complexity)]

use args::Args;
use bevy::{
    asset::AssetMetaCheck,
    audio::{self, AudioPlugin},
    log::{Level, LogPlugin},
    prelude::*,
};
use clap::Parser;

mod animation;
mod args;
mod asset_loader;
mod config;
mod enemy;
mod game_state;
mod health;
mod input;
mod physics;
mod player;
mod score;
mod ui;
mod world;

#[cfg(target_arch = "wasm32")]
extern crate console_error_panic_hook;

use crate::{
    animation::AnimationPlugin, asset_loader::AssetLoaderPlugin, enemy::EnemyPlugin,
    health::HealthPlugin, input::InputPlugin, physics::PhysicsPlugin, player::PlayerPlugin,
    score::ScorePlugin, ui::UIPlugin, world::WorldPlugin,
};

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    let args = Args::parse();

    App::new()
        .insert_resource(args)
        .add_plugins(
            DefaultPlugins
                .set(LogPlugin {
                    filter: "warn,ui=info".to_string(),
                    level: Level::INFO,
                    ..Default::default()
                })
                .set(AssetPlugin {
                    file_path: "assets".into(),
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(AudioPlugin {
                    global_volume: audio::GlobalVolume::new(audio::Volume::SILENT),
                    ..default()
                }),
        )
        .add_plugins((
            AssetLoaderPlugin,
            WorldPlugin,
            InputPlugin,
            PlayerPlugin,
            EnemyPlugin,
            PhysicsPlugin,
            HealthPlugin,
            ScorePlugin,
            AnimationPlugin,
            UIPlugin,
        ))
        .run();
}
