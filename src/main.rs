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

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: (800.0, 600.0).into(),
                        title: "Pombo Hacker".into(),
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
        .add_systems(Startup, setup_camera)
        .add_plugins((
            PlayerPlugin,
            EnemyPlugin,
            CombatPlugin,
            UiPlugin,
            WorldPlugin,
        ))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
