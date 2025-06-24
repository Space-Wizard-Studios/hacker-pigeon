use bevy::{
    prelude::*,
    render::{
        settings::{Backends, RenderCreation, WgpuSettings},
        RenderPlugin,
    },
};

mod combat;
mod components;
mod constants;
mod enemy;
mod player;
mod ui;
mod world;

use combat::CombatPlugin;
use enemy::EnemyPlugin;
use player::PlayerPlugin;
use ui::UiPlugin;
use world::WorldPlugin;
use crate::components::DroneCollisionDebug;

fn main() {
    App::new()
        .insert_resource(DroneCollisionDebug::default())
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Pombo Hacker".into(),
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        backends: Some(Backends::VULKAN),
                        ..default()
                    }),
                    ..default()
                }),
        ))
        .add_plugins((
            PlayerPlugin,
            EnemyPlugin,
            CombatPlugin,
            UiPlugin,
            WorldPlugin,
        ))
        .run();
}
