use args::Args;
use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
};
use clap::Parser;

mod animation;
mod args;
mod asset_loader;
mod enemy;
mod game_state;
mod health;
mod input;
mod physics;
mod player;
mod score;
mod ui;
mod world;

use crate::{
    animation::AnimationPlugin, asset_loader::AssetLoaderPlugin, enemy::EnemyPlugin,
    health::HealthPlugin, input::InputPlugin, physics::PhysicsPlugin, player::PlayerPlugin,
    score::ScorePlugin, ui::UIPlugin, world::WorldPlugin,
};

fn main() {
    let args = Args::parse();
    log::info!("{args:?}");

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(LogPlugin {
                    filter: "warn,ui=info".to_string(),
                    level: Level::INFO,
                    ..Default::default()
                })
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .insert_resource(args)
        .insert_resource(ClearColor(Color::srgb_u8(51, 51, 51)))
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
