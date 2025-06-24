use args::Args;
use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_asset_loader::prelude::*;
use bevy_egui::{
    egui::{self, Align2, Color32, FontId, RichText},
    EguiContextPass, EguiContexts, EguiPlugin,
};
use bevy_framepace::FramepacePlugin;
use clap::Parser;

mod args;
mod input;

#[derive(States, Clone, Default, Debug, Hash, PartialEq, Eq)]
enum GameState {
    #[default]
    AssetLoading,
    GameRunning,
    GameOver,
}

#[derive(AssetCollection, Resource)]
struct ImageAssets {
    #[asset(path = "hacker_pigeon.png")]
    hacker_pigeon: Handle<Image>,
}

#[derive(Resource, Default, Debug)]
struct Score(u32);

#[derive(Component, Clone, Copy)]
struct Player;

#[derive(Resource, Default, Debug)]
struct Input(u8);

#[derive(Resource, Default, Debug)]
struct MousePos(Vec2);

fn main() {
    let args = Args::parse();
    log::info!("{args:?}");

    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: false,
                    ..default()
                }),
                ..default()
            }),
            EguiPlugin {
                enable_multipass_for_primary_context: true,
            },
            FramepacePlugin,
        ))
        .init_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .load_collection::<ImageAssets>()
                .continue_to_state(GameState::GameRunning),
        )
        .insert_resource(args)
        .insert_resource(ClearColor(Color::srgb_u8(51, 51, 51)))
        .init_resource::<Input>()
        .init_resource::<MousePos>()
        .init_resource::<Score>()
        .add_systems(EguiContextPass, ui_system)
        .add_systems(
            OnEnter(GameState::GameRunning),
            (setup, spawn_player).chain(),
        )
        .add_systems(
            Update,
            (input::read_inputs, input::read_mouse_position)
                .run_if(in_state(GameState::GameRunning)),
        )
        .run();
}

fn setup(mut commands: Commands) {
    let cam = Camera2d;
    let projection = Projection::Orthographic(OrthographicProjection {
        scaling_mode: ScalingMode::FixedVertical {
            viewport_height: 10.,
        },
        ..OrthographicProjection::default_2d()
    });

    commands.spawn((cam, projection));
    commands.set_state(GameState::GameRunning);
}

fn spawn_player(
    mut commands: Commands,
    mut score: ResMut<Score>,
    images: Res<ImageAssets>,
    players: Query<Entity, With<Player>>,
) {
    log::info!("Spawning player...");

    score.0 = 0;

    for player in &players {
        commands.entity(player).despawn();
    }

    commands.spawn((
        Player,
        Transform::from_translation(Vec3::ZERO),
        Sprite {
            color: Color::srgb_u8(0, 120, 255),
            custom_size: Some(Vec2::new(1., 1.)),
            ..default()
        },
        Sprite {
            image: images.hacker_pigeon.clone(),
            custom_size: Some(Vec2::new(0.3, 0.1)),
            ..default()
        },
    ));
}

fn ui_system(
    mut contexts: EguiContexts,
    score: Res<Score>,
    input: Res<Input>,
    mouse_pos: Res<MousePos>,
) {
    egui::Area::new("score".into())
        .anchor(Align2::CENTER_TOP, (0., 25.))
        .show(contexts.ctx_mut(), |ui| {
            ui.label(
                RichText::new(format!("Score: {}", score.0))
                    .color(Color32::BLACK)
                    .font(FontId::proportional(72.0)),
            );
        });

    let input_dir = input::direction(input.0);
    let mouse_pos = mouse_pos.0;

    egui::Area::new("debug".into())
        .anchor(Align2::LEFT_TOP, (0., 25.))
        .show(contexts.ctx_mut(), |ui| {
            ui.label(
                RichText::new(format!(
                    "Input: {:.2}, {:.2}\nMouse: {:.0} {:.0}",
                    input_dir.x, input_dir.y, mouse_pos.x, mouse_pos.y
                ))
                .color(Color32::BLACK)
                .font(FontId::proportional(72.0)),
            );
        });
}
