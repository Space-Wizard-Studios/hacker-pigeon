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
                        resolution: (1920.0, 1080.0).into(),
                        title: "Pombo Hacker".into(),
                        resizable: true,
                        resize_constraints: WindowResizeConstraints {
                            min_width: 640.0,
                            min_height: 360.0,
                            max_width: 3840.0,
                            max_height: 2160.0,
                        },
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
