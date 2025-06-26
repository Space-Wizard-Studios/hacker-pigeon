use std::time::Duration;

use args::Args;
use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
    render::camera::ScalingMode,
};
use bevy_asset_loader::prelude::*;
use bevy_egui::{
    egui::{self, Align2, Color32, FontId, RichText},
    EguiContextPass, EguiContexts, EguiPlugin,
};

use bevy_framepace::{FramepacePlugin, FramepaceSettings};
use clap::Parser;

use crate::input::{Input, MousePos};

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
    #[asset(path = "pigeon/flying/spritesheet.png")]
    pigeon_fly_sheet: Handle<Image>,
}

#[derive(Resource, Default, Debug)]
struct Score(u32);

#[derive(Component, Default, Debug)]
struct Player;

#[derive(Component, Default, Debug)]
struct Enemy;

#[derive(Default, Debug)]
enum WeakSpotLocation {
    North,
    #[default]
    South,
    West,
    East,
}

impl WeakSpotLocation {
    fn to_dir(&self) -> Vec3 {
        match self {
            WeakSpotLocation::North => Vec3::Y,
            WeakSpotLocation::South => Vec3::NEG_Y,
            WeakSpotLocation::West => Vec3::X,
            WeakSpotLocation::East => Vec3::NEG_X,
        }
    }
}

#[derive(Component, Default, Debug, Deref, DerefMut)]
struct WeakSpot(WeakSpotLocation);

#[derive(Default, Debug)]
enum AnimationDir {
    #[default]
    Forwards,
    Backwards,
}

#[derive(Component, Default, Debug)]
struct Animation {
    first: usize,
    last: usize,
    dir: AnimationDir,
    timer: Timer,
}

#[derive(Component, Default, Debug)]
struct Velocity {
    current: Vec2,
    target: Vec2,
}

#[derive(Component, Default, Debug, Deref, DerefMut)]
struct Radius(f32);

#[derive(Component, Default, Debug)]
struct CollisionImmunity {
    timer: Timer,
}

impl CollisionImmunity {
    fn new(duration_secs: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration_secs, TimerMode::Once),
        }
    }
}

#[derive(Component, Default, Debug)]
struct Blink {
    timer: Timer,
}

impl Blink {
    fn new(duration_millis: u64) -> Self {
        Self {
            timer: Timer::new(Duration::from_millis(duration_millis), TimerMode::Repeating),
        }
    }
}

#[derive(Component, Default, Debug)]
struct Grounded;

#[derive(Component, Default, Debug)]
struct Airborne;

#[derive(Component, Default, Debug)]
struct ChargingDash {
    dir: Vec2,
    power: f32,
}

impl ChargingDash {
    fn new(dir: Vec2) -> Self {
        Self { dir, power: 0. }
    }
}

#[derive(Component, Default, Debug)]
struct Dashing {
    power: Vec2,
    timer: Timer,
}

impl Dashing {
    fn new(power: Vec2, duration_secs: f32) -> Self {
        Self {
            power,
            timer: Timer::from_seconds(duration_secs, TimerMode::Once),
        }
    }
}

#[derive(Component, Default, Debug)]
struct Health {
    current: u8,
    max: u8,
}

impl Health {
    fn new(value: u8) -> Self {
        Self {
            current: value,
            max: value,
        }
    }
}

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
            (setup, spawn_player, spawn_enemy).chain(),
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
                drone_player_collision_system,
                collision_immunity_system,
                blink_system,
            )
                .chain()
                .run_if(in_state(GameState::GameRunning)),
        )
        .add_systems(
            Update,
            animate_player.run_if(in_state(GameState::GameRunning)),
        )
        .run();
}

fn setup(mut commands: Commands, mut settings: ResMut<FramepaceSettings>) {
    use bevy_framepace::Limiter;
    settings.limiter = Limiter::from_framerate(60.0);

    let cam = Camera2d;
    let projection = Projection::Orthographic(OrthographicProjection {
        scaling_mode: ScalingMode::FixedVertical {
            viewport_height: 480.,
        },
        ..OrthographicProjection::default_2d()
    });

    commands.spawn((cam, projection));
    commands.set_state(GameState::GameRunning);
}

fn spawn_player(
    mut commands: Commands,
    mut score: ResMut<Score>,
    players: Query<Entity, With<Player>>,
    images: Res<ImageAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    log::info!("Spawning player...");

    score.0 = 0;

    for player in &players {
        commands.entity(player).despawn();
    }

    let image = images.pigeon_fly_sheet.clone();
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 4, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);

    let animation = Animation {
        first: 0,
        last: 3,
        dir: AnimationDir::Forwards,
        timer: Timer::from_seconds(0.1, TimerMode::Repeating),
    };

    let sprite = Sprite::from_atlas_image(
        image,
        TextureAtlas {
            layout: texture_atlas_layout,
            index: animation.first,
        },
    );

    commands.spawn((
        Player,
        Velocity::default(),
        Transform::from_translation(Vec3::ZERO),
        Radius(16.),
        Health::new(3),
        sprite,
        animation,
    ));
}

fn spawn_enemy(mut commands: Commands) {
    let weak_spot = WeakSpot::default();
    let weak_spot_pos = weak_spot.to_dir() * 16.;

    commands
        .spawn((
            Enemy,
            Velocity::default(),
            Transform::from_translation(Vec3::new(0., 200., 0.)),
            Radius(16.),
            Health::new(1),
            weak_spot,
            Airborne,
            Sprite {
                color: Color::srgb_u8(200, 10, 10),
                custom_size: Some(Vec2::splat(32.)),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Transform::from_translation(Vec3::new(weak_spot_pos.x, weak_spot_pos.y, 1.)),
                Sprite {
                    color: Color::srgb_u8(200, 200, 10),
                    custom_size: Some(Vec2::splat(12.)),
                    ..default()
                },
            ));
        });
}

fn animate_player(time: Res<Time>, mut player: Query<(&mut Animation, &mut Sprite, &Velocity)>) {
    if let Ok((mut anim, mut sprite, vel)) = player.single_mut() {
        if vel.target.x.abs() != 0. {
            sprite.flip_x = vel.target.x < 0.;
        }

        anim.timer.tick(time.delta());

        if anim.timer.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                match anim.dir {
                    AnimationDir::Forwards => {
                        atlas.index += 1;
                        if atlas.index == anim.last {
                            anim.dir = AnimationDir::Backwards;
                        }
                    }
                    AnimationDir::Backwards => {
                        atlas.index -= 1;
                        if atlas.index == anim.first {
                            anim.dir = AnimationDir::Forwards;
                        }
                    }
                }
            }
        }
    }
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
            &Health,
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

        let input_dir = input.dir();
        let mouse_pos = **mouse_pos;

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

        if let Ok((entity, transform, vel, health, grounded_opt, charging_opt, dashing_opt)) =
            player.single()
        {
            egui::Area::new("debug player".into())
                .anchor(Align2::LEFT_TOP, (16., 96.))
                .show(ctx, |ui| {
                    ui.set_min_width(1000.0);
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(format!("entity-idx: {}", entity.index()))
                                .color(Color32::WHITE)
                                .font(FontId::proportional(16.0)),
                        );
                        ui.label(
                            RichText::new(format!(
                                "position:\n{:.2},{:.2}",
                                transform.translation.x, transform.translation.y
                            ))
                            .color(Color32::WHITE)
                            .font(FontId::proportional(16.0)),
                        );
                        ui.label(
                            RichText::new(format!(
                                "velocity (current):\n{:.2} {:.2}",
                                vel.current.x, vel.current.y
                            ))
                            .color(Color32::WHITE)
                            .font(FontId::proportional(16.0)),
                        );
                        ui.label(
                            RichText::new(format!(
                                "velocity (target):\n{:.2} {:.2}",
                                vel.target.x, vel.target.y
                            ))
                            .color(Color32::WHITE)
                            .font(FontId::proportional(16.0)),
                        );

                        ui.label(
                            RichText::new(format!("hp: {}/{}", health.current, health.max))
                                .color(Color32::WHITE)
                                .font(FontId::proportional(16.0)),
                        );

                        ui.label(
                            RichText::new(format!("grounded: {}", grounded_opt.is_some()))
                                .color(Color32::WHITE)
                                .font(FontId::proportional(16.0)),
                        );
                        ui.label(
                            RichText::new(format!("charging: {}", charging_opt.is_some()))
                                .color(Color32::WHITE)
                                .font(FontId::proportional(16.0)),
                        );
                        ui.label(
                            RichText::new(format!("dashing: {}", dashing_opt.is_some()))
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
        let input = input.dir();
        vel.target.x += input.x * PLAYER_X_ACCELERATION * dt;
        vel.target.y += input.y * PLAYER_Y_ACCELERATION * dt;

        vel.target.x = vel.target.x.clamp(-PLAYER_MAX_X_SPEED, PLAYER_MAX_X_SPEED);
        vel.target.y = vel
            .target
            .y
            .clamp(PLAYER_MIN_FALL_SPEED, PLAYER_MAX_RISE_SPEED);

        if input.y > 0. {
            vel.target.y = vel.target.y.max(0.)
        }
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
            vel.target = Vec2::ZERO;

            let pos = transform.translation.xy();
            let dir = (**mouse_pos - pos).normalize_or_zero();
            commands.entity(entity).insert(ChargingDash::new(dir));
        }
    }
}

fn player_charge_dash_system(
    mut commands: Commands,
    input: Res<Input>,
    mouse_pos: Res<MousePos>,
    mut player: Query<(Entity, &Transform, &mut ChargingDash), (With<Player>, Without<Dashing>)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    if let Ok((entity, transform, mut charging)) = player.single_mut() {
        if input.dash() {
            let pos = transform.translation.xy();
            let dir = (**mouse_pos - pos).normalize_or_zero();
            charging.dir = dir;
            charging.power += dt / PLAYER_CHARGING_POWER_DURATION;
        } else {
            let dash_power = charging.power.min(1.0);
            commands.entity(entity).remove::<ChargingDash>();
            commands.entity(entity).insert(Dashing::new(
                charging.dir * dash_power,
                PLAYER_DASH_DURATION,
            ));
        }
    }
}

fn player_dash_system(
    mut commands: Commands,
    mut player: Query<(Entity, &mut Velocity, &mut Dashing), With<Player>>,
    time: Res<Time>,
) {
    if let Ok((entity, mut vel, mut dash)) = player.single_mut() {
        vel.target = dash.power * PLAYER_DASH_SPEED;

        dash.timer.tick(time.delta());
        if dash.timer.finished() {
            vel.target = dash.power.normalize_or_zero() * PLAYER_MAX_X_SPEED;
            commands.entity(entity).remove::<Dashing>();
        }
    }
}

fn drone_player_collision_system(
    mut commands: Commands,
    mut player: Query<
        (Entity, &Transform, &Radius, &mut Health),
        (With<Player>, Without<CollisionImmunity>),
    >,
    enemies: Query<(&Transform, &Radius, &WeakSpot), With<Enemy>>,
) {
    if let Ok((entity, player, radius, mut health)) = player.single_mut() {
        let player_pos = player.translation;
        let player_size = **radius;

        for (enemy, radius, weak_spot) in enemies.iter() {
            let enemy_pos = enemy.translation;
            let enemy_size = **radius;

            let dist_sq = (player_pos - enemy_pos).length_squared();
            let threshold = (player_size + enemy_size) * (player_size + enemy_size);
            if dist_sq <= threshold {
                commands.entity(entity).insert(CollisionImmunity::new(1.0));
                commands.entity(entity).insert(Blink::new(50));

                if health.current > 0 {
                    health.current -= 1;
                }
            }
        }
    }
}

fn collision_immunity_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut CollisionImmunity, &mut Sprite)>,
    time: Res<Time>,
) {
    for (entity, mut immunity, mut sprite) in query.iter_mut() {
        immunity.timer.tick(time.delta());
        if immunity.timer.finished() {
            commands.entity(entity).remove::<CollisionImmunity>();
            commands.entity(entity).remove::<Blink>();
            sprite.color = Color::WHITE;
        }
    }
}

fn blink_system(mut query: Query<(&mut Blink, &mut Sprite)>, time: Res<Time>) {
    for (mut blink, mut sprite) in query.iter_mut() {
        blink.timer.tick(time.delta());
        sprite.color = COLLISION_IMMUNITY_BLINK_COLOR;

        if blink.timer.finished() {
            sprite.color = Color::WHITE;
        }
    }
}

fn gravity_system(
    mut query: Query<
        (&mut Velocity, Option<&ChargingDash>),
        (Without<Grounded>, Without<Dashing>, Without<Airborne>),
    >,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (mut vel, charging_opt) in query.iter_mut() {
        let multiplier = if charging_opt.is_some() {
            PLAYER_CHARGING_GRAVITY_MULTIPLIER
        } else {
            1.0
        };

        vel.target.y += GRAVITY * multiplier * dt;
    }
}

fn friction_system(mut query: Query<(&mut Velocity, Option<&Grounded>)>, time: Res<Time>) {
    let dt = time.delta_secs();

    for (mut vel, grounded_opt) in query.iter_mut() {
        let friction = if grounded_opt.is_some() {
            GROUND_FRICTION
        } else {
            AIR_FRICTION
        };

        vel.target.x *= (1.0 - friction * dt).max(0.0);
        vel.target.y *= (1.0 - friction * dt).max(0.0);
    }
}

fn apply_velocity_system(mut query: Query<(&mut Transform, &mut Velocity)>, time: Res<Time>) {
    let dt = time.delta_secs();

    for (mut transform, mut vel) in query.iter_mut() {
        vel.current = vel
            .current
            .lerp(vel.target, (MOVEMENT_SMOOTHING * dt).min(1.0));

        transform.translation.x += vel.current.x * dt;
        transform.translation.y += vel.current.y * dt;
    }
}

fn apply_grounding_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Velocity)>,
) {
    for (entity, mut transform, mut vel) in query.iter_mut() {
        let is_below_or_on_floor = transform.translation.y <= FLOOR_Y;
        let is_above_ground_threshold = transform.translation.y > FLOOR_Y + 0.01;
        let is_falling = vel.current.y <= 0.0;

        if is_below_or_on_floor && is_falling {
            transform.translation.y = FLOOR_Y;
            vel.current.y = 0.0;
            vel.target.y = 0.0;

            commands.entity(entity).insert_if_new(Grounded);
        } else if is_above_ground_threshold {
            commands.entity(entity).remove::<Grounded>();
        }
    }
}

const GRAVITY: f32 = -9.8;
const MOVEMENT_SMOOTHING: f32 = 8.0;

const AIR_FRICTION: f32 = 0.4;
const GROUND_FRICTION: f32 = 6.0;

const FLOOR_Y: f32 = -160.0;

const PLAYER_X_ACCELERATION: f32 = 2400.0;
const PLAYER_Y_ACCELERATION: f32 = 360.0;
const PLAYER_MAX_X_SPEED: f32 = 240.0;
const PLAYER_MIN_FALL_SPEED: f32 = -680.0;
const PLAYER_MAX_RISE_SPEED: f32 = 480.0;
const PLAYER_CHARGING_GRAVITY_MULTIPLIER: f32 = 0.05;
const PLAYER_CHARGING_POWER_DURATION: f32 = 0.5;
const PLAYER_DASH_DURATION: f32 = 0.1;
const PLAYER_DASH_SPEED: f32 = 3200.0;

const COLLISION_IMMUNITY_BLINK_COLOR: Color = Color::srgba_u8(255, 0, 0, 0);
