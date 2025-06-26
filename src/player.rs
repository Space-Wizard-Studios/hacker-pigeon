use bevy::prelude::*;

use crate::{
    animation::{Animation, AnimationDir},
    asset_loader::ImageAssets,
    game_state::GameState,
    health::Health,
    input::{Input, MousePos},
    physics::{Radius, Velocity},
    score::Score,
};

#[derive(Component, Default, Debug)]
pub struct Player;

#[derive(Component, Default, Debug)]
pub struct ChargingDash {
    pub dir: Vec2,
    pub power: f32,
}

impl ChargingDash {
    pub fn new(dir: Vec2) -> Self {
        Self { dir, power: 0. }
    }
}

#[derive(Component, Default, Debug)]
pub struct Dashing {
    pub power: Vec2,
    pub timer: Timer,
}

impl Dashing {
    pub fn new(power: Vec2, duration_secs: f32) -> Self {
        Self {
            power,
            timer: Timer::from_seconds(duration_secs, TimerMode::Once),
        }
    }
}

#[derive(Component, Default, Debug)]
pub struct DashDirectionArrow {
    pub direction: Vec2,
    pub visibility: bool,
    pub size: f32,
}

#[derive(Component, Default, Debug)]
pub struct DashImmunity {
    pub timer: Timer,
}

impl DashImmunity {
    pub fn new(duration_secs: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration_secs, TimerMode::Once),
        }
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GameRunning), spawn_player)
            .add_systems(
                Update,
                (
                    player_movement_system,
                    player_dash_system,
                    player_dash_immunity_system,
                    player_bounds_system,
                    player_start_charge_dash_system,
                    player_charge_dash_system,
                    dash_arrow_system,
                )
                    .chain()
                    .run_if(in_state(GameState::GameRunning)),
            );
    }
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

    commands
        .spawn((
            Player,
            Velocity::default(),
            Transform::from_translation(Vec3::ZERO),
            Radius(16.),
            Health::new(3),
            sprite,
            animation,
        ))
        .with_children(|parent| {
            parent.spawn((
                DashDirectionArrow::default(),
                Transform::from_translation(Vec3::ZERO),
                Sprite {
                    color: Color::srgba_u8(200, 200, 10, 0),
                    custom_size: Some(Vec2::splat(6.)),
                    ..default()
                },
            ));
        });
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
    }
}

fn player_bounds_system(mut player: Query<(&Transform, &mut Velocity), With<Player>>) {
    if let Ok((transform, mut vel)) = player.single_mut() {
        let overstep = transform.translation.y - CEILING_Y;

        if overstep > 0.0 {
            let pull = -overstep * SPRING_FORCE;
            vel.target.y = pull.clamp(MAX_PULL, 0.0);
        }
    }
}

fn player_start_charge_dash_system(
    mut commands: Commands,
    input: Res<Input>,
    mouse_pos: Res<MousePos>,
    mut player: Query<
        (Entity, &Transform, &mut Velocity, &Children),
        (With<Player>, Without<ChargingDash>),
    >,
    mut arrows: Query<&mut DashDirectionArrow>,
) {
    if let Ok((entity, transform, mut vel, children)) = player.single_mut() {
        if input.dash() {
            vel.target = Vec2::ZERO;

            let pos = transform.translation.xy();
            let dir = (**mouse_pos - pos).normalize_or_zero();
            commands.entity(entity).insert(ChargingDash::new(dir));

            for &child in children.into_iter() {
                if let Ok(mut arrow) = arrows.get_mut(child) {
                    arrow.direction = dir;
                    arrow.visibility = true;
                    arrow.size = 0.;
                    break;
                }
            }
        }
    }
}

fn player_charge_dash_system(
    mut commands: Commands,
    input: Res<Input>,
    mouse_pos: Res<MousePos>,
    mut player: Query<
        (Entity, &Transform, &mut ChargingDash, &Children),
        (With<Player>, Without<Dashing>),
    >,
    mut arrows: Query<&mut DashDirectionArrow>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    if let Ok((entity, transform, mut charging, children)) = player.single_mut() {
        if input.dash() {
            let pos = transform.translation.xy();
            let dir = (**mouse_pos - pos).normalize_or_zero();
            let power = dt / PLAYER_CHARGING_POWER_DURATION;

            charging.dir = dir;
            charging.power += power;

            for &child in children.into_iter() {
                if let Ok(mut arrow) = arrows.get_mut(child) {
                    arrow.direction = dir;
                    arrow.visibility = true;
                    arrow.size += power;
                    break;
                }
            }
        } else {
            let dash_power = charging.power.min(1.0);
            commands.entity(entity).remove::<ChargingDash>();
            commands.entity(entity).insert(Dashing::new(
                charging.dir * dash_power,
                PLAYER_DASH_DURATION,
            ));
            commands
                .entity(entity)
                .insert(DashImmunity::new(PLAYER_DASH_IMMUNITY_DURATION));

            for &child in children.into_iter() {
                if let Ok(mut arrow) = arrows.get_mut(child) {
                    arrow.visibility = false;
                    break;
                }
            }
        }
    }
}

fn dash_arrow_system(mut arrows: Query<(&DashDirectionArrow, &mut Transform, &mut Sprite)>) {
    for (arrow, mut transform, mut sprite) in arrows.iter_mut() {
        let pos = arrow.direction * 32.;
        transform.translation.x = pos.x;
        transform.translation.y = pos.y;

        let alpha = if arrow.visibility { 1. } else { 0. };
        sprite.color.set_alpha(alpha);

        let size = arrow.size.min(1.);
        transform.scale = Vec3::splat(size);
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

fn player_dash_immunity_system(
    mut commands: Commands,
    mut player: Query<(Entity, &mut DashImmunity), With<Player>>,
    time: Res<Time>,
) {
    if let Ok((entity, mut dash)) = player.single_mut() {
        dash.timer.tick(time.delta());
        if dash.timer.finished() {
            commands.entity(entity).remove::<DashImmunity>();
        }
    }
}

const CEILING_Y: f32 = 160.0;
const SPRING_FORCE: f32 = 6.0;
const MAX_PULL: f32 = -280.0;

const PLAYER_X_ACCELERATION: f32 = 2200.0;
const PLAYER_Y_ACCELERATION: f32 = 340.0;
const PLAYER_MAX_X_SPEED: f32 = 200.0;
const PLAYER_MIN_FALL_SPEED: f32 = -680.0;
const PLAYER_MAX_RISE_SPEED: f32 = 480.0;
const PLAYER_CHARGING_POWER_DURATION: f32 = 0.5;
const PLAYER_DASH_DURATION: f32 = 0.12;
const PLAYER_DASH_IMMUNITY_DURATION: f32 = 1.;
const PLAYER_DASH_SPEED: f32 = 3000.0;
