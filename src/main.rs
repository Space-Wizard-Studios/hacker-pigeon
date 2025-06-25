use args::Args;
use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
    render::camera::ScalingMode,
};
use bevy_asset_loader::prelude::*;
use bevy_egui::{
    egui::{self, Align2, Color32, FontId, RichText, TextStyle},
    EguiContextPass, EguiContexts, EguiPlugin,
};

use bevy_framepace::{FramepacePlugin, FramepaceSettings};
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

#[derive(Resource, Default, Debug, Deref)]
struct Input(u8);

#[derive(Resource, Default, Debug, Deref)]
struct MousePos(Vec2);

#[derive(Component, Default, Debug)]
struct Player;

#[derive(Component, Default, Debug, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component, Default, Debug)]
struct Grounded;

#[derive(Component, Default, Debug, Deref, DerefMut)]
struct ChargingDash(Vec2);

#[derive(Component, Default, Debug)]
struct Dashing {
    dir: Vec2,
    power: f32,
    timer: Timer,
}

#[derive(Component, Default, Debug, Deref, DerefMut)]
struct Health(#[deref] u8, u8);

#[derive(Component, Default, Debug)]
struct Killed;

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
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_plugins(FramepacePlugin)
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
        .add_systems(
            Update,
            (
                player_movement_system,
                player_dash_system,
                player_start_charge_dash_system,
                player_charge_dash_system,
            )
                .chain()
                .run_if(in_state(GameState::GameRunning)),
        )
        .add_systems(
            Update,
            (
                gravity_system,
                friction_system,
                apply_velocity_system,
                apply_grounding_system,
            )
                .chain()
                .run_if(in_state(GameState::GameRunning)),
        )
        .run();
}

fn setup(mut commands: Commands, mut settings: ResMut<FramepaceSettings>) {
    use bevy_framepace::Limiter;
    settings.limiter = Limiter::from_framerate(60.0);

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
        Velocity::default(),
        Transform::from_translation(Vec3::ZERO),
        Health(3, 3),
        Sprite {
            image: images.hacker_pigeon.clone(),
            custom_size: Some(Vec2::new(0.5, 0.5)),
            ..default()
        },
    ));
}

fn ui_system(
    mut contexts: EguiContexts,
    score: Res<Score>,
    input: Res<Input>,
    mouse_pos: Res<MousePos>,
    player: Query<
        (
            Entity,
            &Transform,
            &Velocity,
            Option<&Grounded>,
            Option<&ChargingDash>,
            Option<&Dashing>,
        ),
        With<Player>,
    >,
) {
    if let Some(ctx) = contexts.try_ctx_mut() {
        egui::Area::new("score".into())
            .anchor(Align2::CENTER_TOP, (0., 16.))
            .show(ctx, |ui| {
                ui.label(
                    RichText::new(format!("Score: {}", score.0))
                        .color(Color32::WHITE)
                        .font(FontId::proportional(16.0)),
                );
            });

        let input_dir = input.direction();

        egui::Area::new("debug input".into())
            .anchor(Align2::LEFT_TOP, (16., 16.))
            .show(ctx, |ui| {
                ui.set_min_width(1000.0);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(format!("Input:\n{:.2} {:.2}", input_dir.x, input_dir.y))
                            .color(Color32::WHITE)
                            .font(FontId::proportional(16.0)),
                    );
                    ui.label(
                        RichText::new(format!("Mouse:\n{:.0} {:.0}", mouse_pos.x, mouse_pos.y))
                            .color(Color32::WHITE)
                            .font(FontId::proportional(16.0)),
                    );
                });
            });

        if let Ok((entity, transform, vel, grounded_opt, charging_opt, dashing_opt)) =
            player.single()
        {
            egui::Area::new("debug player".into())
                .anchor(Align2::LEFT_TOP, (16., 96.))
                .show(ctx, |ui| {
                    ui.set_min_width(1000.0);
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(format!("EntityIdx: {}", entity.index()))
                                .color(Color32::WHITE)
                                .font(FontId::proportional(16.0)),
                        );
                        ui.label(
                            RichText::new(format!(
                                "Position:\n{:.2},{:.2}",
                                transform.translation.x, transform.translation.y
                            ))
                            .color(Color32::WHITE)
                            .font(FontId::proportional(16.0)),
                        );
                        ui.label(
                            RichText::new(format!("Velocity:\n{:.2} {:.2}", vel.x, vel.y))
                                .color(Color32::WHITE)
                                .font(FontId::proportional(16.0)),
                        );

                        ui.label(
                            RichText::new(format!("Grounded: {}", grounded_opt.is_some()))
                                .color(Color32::WHITE)
                                .font(FontId::proportional(16.0)),
                        );
                        ui.label(
                            RichText::new(format!("Charging: {}", charging_opt.is_some()))
                                .color(Color32::WHITE)
                                .font(FontId::proportional(16.0)),
                        );
                        ui.label(
                            RichText::new(format!("Dashing: {}", dashing_opt.is_some()))
                                .color(Color32::WHITE)
                                .font(FontId::proportional(16.0)),
                        );
                    });
                });
        }
    }
}

fn player_movement_system(
    input: Res<Input>,
    mut player: Query<&mut Velocity, (With<Player>, Without<ChargingDash>, Without<Dashing>)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    if let Ok(mut vel) = player.single_mut() {
        let dir = input.direction();
        let acc = dir * PLAYER_ACCELERATION * dt;
        vel.0 = vel.0 + acc;

        vel.x = vel.x.clamp(-PLAYER_MAX_X_SPEED, PLAYER_MAX_X_SPEED);
        vel.y = vel.y.clamp(PLAYER_MIN_FALL_SPEED, PLAYER_MAX_RISE_SPEED);
    }
}

fn player_start_charge_dash_system(
    mut commands: Commands,
    input: Res<Input>,
    mouse_pos: Res<MousePos>,
    mut player: Query<(Entity, &Transform, &mut Velocity), (With<Player>, Without<ChargingDash>)>,
) {
    if let Ok((entity, transform, mut vel)) = player.single_mut() {
        if input.dash() {
            vel.0 = Vec2::ZERO;

            let pos = transform.translation.xy();
            let dir = (mouse_pos.0 - pos).normalize_or_zero();
            commands.entity(entity).insert(ChargingDash(dir));
        }
    }
}

fn player_charge_dash_system(
    mut commands: Commands,
    input: Res<Input>,
    mouse_pos: Res<MousePos>,
    mut player: Query<(Entity, &Transform, &mut ChargingDash), (With<Player>, Without<Dashing>)>,
) {
    if let Ok((entity, transform, mut charging)) = player.single_mut() {
        if input.dash() {
            let pos = transform.translation.xy();
            let dir = (mouse_pos.0 - pos).normalize_or_zero();
            charging.0 = dir;
        } else {
            commands.entity(entity).remove::<ChargingDash>();
            commands.entity(entity).insert(Dashing {
                dir: charging.0,
                power: PLAYER_DASH_ACCELERATION,
                timer: Timer::from_seconds(PLAYER_DASH_DURATION, TimerMode::Once),
            });
        }
    }
}

fn player_dash_system(
    mut commands: Commands,
    time: Res<Time>,
    mut player: Query<(Entity, &mut Velocity, &mut Dashing), With<Player>>,
) {
    let dt = time.delta_secs();

    if let Ok((entity, mut vel, mut dash)) = player.single_mut() {
        vel.x += dash.dir.x * dash.power * dt;
        vel.y += dash.dir.y * dash.power * dt;
        vel.0 = vel.clamp_length_max(dash.power);

        dash.timer.tick(time.delta());
        if dash.timer.finished() {
            commands.entity(entity).remove::<Dashing>();
        }
    }
}

fn gravity_system(
    mut query: Query<(&mut Velocity, Option<&ChargingDash>), (Without<Grounded>, Without<Dashing>)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (mut vel, charging_opt) in query.iter_mut() {
        let multiplier = if charging_opt.is_some() {
            CHARGING_GRAVITY_MULTIPLIER
        } else {
            1.0
        };

        vel.y += GRAVITY * multiplier * dt;
    }
}

fn friction_system(
    mut query: Query<(&mut Velocity, Option<&Grounded>, Option<&Dashing>)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (mut vel, grounded_opt, dashing_opt) in query.iter_mut() {
        let friction_x = if grounded_opt.is_some() {
            GROUND_FRICTION
        } else if dashing_opt.is_some() {
            DASHING_FRICTION
        } else {
            AIR_FRICTION
        };

        let friction_y = if dashing_opt.is_some() {
            DASHING_FRICTION
        } else if vel.y < 0.0 {
            FALL_FRICTION
        } else {
            AIR_FRICTION
        };

        vel.x *= (1.0 - friction_x * dt).max(0.0);
        vel.y *= (1.0 - friction_y * dt).max(0.0);
    }
}

fn apply_velocity_system(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    let dt = time.delta_secs();

    for (mut transform, vel) in query.iter_mut() {
        transform.translation.x += vel.x * dt;
        transform.translation.y += vel.y * dt;
    }
}

fn apply_grounding_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Velocity)>,
) {
    for (entity, mut transform, mut vel) in query.iter_mut() {
        let is_below_or_on_floor = transform.translation.y <= FLOOR_Y;
        let is_above_ground_threshold = transform.translation.y > FLOOR_Y + 0.01;
        let is_falling = vel.y <= 0.0;

        if is_below_or_on_floor && is_falling {
            transform.translation.y = FLOOR_Y;
            vel.y = 0.0;

            commands.entity(entity).insert_if_new(Grounded);
        } else if is_above_ground_threshold {
            commands.entity(entity).remove::<Grounded>();
        }
    }
}

const GRAVITY: f32 = -9.8;
const CHARGING_GRAVITY_MULTIPLIER: f32 = 0.1;

const AIR_FRICTION: f32 = 4.0;
const FALL_FRICTION: f32 = 0.0;
const GROUND_FRICTION: f32 = 12.0;
const DASHING_FRICTION: f32 = 12.0;

const FLOOR_Y: f32 = 0.0;

const PLAYER_ACCELERATION: f32 = 80.0;
const PLAYER_MAX_X_SPEED: f32 = 6.0;
const PLAYER_MIN_FALL_SPEED: f32 = -12.0;
const PLAYER_MAX_RISE_SPEED: f32 = 6.0;
const PLAYER_DASH_DURATION: f32 = 0.35;
const PLAYER_DASH_ACCELERATION: f32 = 260.0;
