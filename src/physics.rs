use bevy::prelude::*;
use std::time::Duration;

use crate::{
    config::Config,
    enemy::{Enemy, WeakSpot},
    game_state::GameState,
    health::Health,
    player::{ChargingDash, DashEffect, Dashing, Nuke, Player},
    score::Score,
};

#[derive(Component, Default, Debug, Deref, DerefMut)]
pub struct Radius(pub f32);

#[derive(Component, Default, Debug)]
pub struct Velocity {
    pub current: Vec2,
    pub target: Vec2,
}

#[derive(Component, Default, Debug)]
pub struct Grounded;

#[derive(Component, Default, Debug)]
pub struct Airborne;

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
pub struct Blink {
    pub timer: Timer,
}

impl Blink {
    pub fn new(duration_millis: u64) -> Self {
        Self {
            timer: Timer::new(Duration::from_millis(duration_millis), TimerMode::Repeating),
        }
    }
}

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                gravity_system,
                friction_system,
                apply_velocity_system,
                apply_grounding_system,
                player_damage_drone_system,
                player_drone_collision_system,
                nuke_drone_collision_system,
                collision_immunity_system,
                blink_system,
            )
                .chain()
                .run_if(in_state(GameState::GameRunning)),
        );
    }
}

fn gravity_system(
    mut query: Query<
        (&mut Velocity, Option<&ChargingDash>),
        (Without<Grounded>, Without<Dashing>, Without<Airborne>),
    >,
    time: Res<Time>,
    cfg: Res<Config>,
) {
    let dt = time.delta_secs();

    for (mut vel, charging_opt) in query.iter_mut() {
        let multiplier = if charging_opt.is_some() {
            cfg.game.charging_gravity_multiplier
        } else {
            1.0
        };

        vel.target.y += cfg.game.gravity * multiplier * dt;
    }
}

fn friction_system(
    mut query: Query<(&mut Velocity, Option<&Grounded>)>,
    time: Res<Time>,
    cfg: Res<Config>,
) {
    let dt = time.delta_secs();

    for (mut vel, grounded_opt) in query.iter_mut() {
        let friction = if grounded_opt.is_some() {
            cfg.game.ground_friction
        } else {
            cfg.game.air_friction
        };

        vel.target.x *= (1.0 - friction * dt).max(0.0);
        vel.target.y *= (1.0 - friction * dt).max(0.0);
    }
}

fn apply_velocity_system(
    mut query: Query<(&mut Transform, &mut Velocity)>,
    time: Res<Time>,
    cfg: Res<Config>,
) {
    let dt = time.delta_secs();

    for (mut transform, mut vel) in query.iter_mut() {
        vel.current = vel
            .current
            .lerp(vel.target, (cfg.game.movement_smoothing * dt).min(1.0));

        transform.translation.x += vel.current.x * dt;
        transform.translation.y += vel.current.y * dt;
    }
}

fn apply_grounding_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Velocity)>,
    cfg: Res<Config>,
) {
    for (entity, mut transform, mut vel) in query.iter_mut() {
        let is_below_or_on_floor = transform.translation.y <= cfg.game.floor_y;
        let is_above_ground_threshold = transform.translation.y > cfg.game.floor_y + 0.01;
        let is_falling = vel.current.y <= 0.0;

        if is_below_or_on_floor && is_falling {
            transform.translation.y = cfg.game.floor_y;
            vel.current.y = 0.0;
            vel.target.y = 0.0;

            commands.entity(entity).insert_if_new(Grounded);
        } else if is_above_ground_threshold {
            commands.entity(entity).remove::<Grounded>();
        }
    }
}

fn player_drone_collision_system(
    mut commands: Commands,
    mut player: Query<
        (Entity, &Transform, &Radius, &mut Velocity, &mut Health),
        (With<Player>, Without<CollisionImmunity>),
    >,
    enemies: Query<(&Transform, &Radius), With<Enemy>>,
) {
    if let Ok((player, player_transform, radius, mut vel, mut health)) = player.single_mut() {
        let player_pos = player_transform.translation;
        let player_size = **radius;

        for (enemy_transform, radius) in enemies.iter() {
            let enemy_pos = enemy_transform.translation;
            let enemy_size = **radius;

            let threshold = (player_size + enemy_size) * (player_size + enemy_size);

            let dist_sq = (player_pos - enemy_pos).length_squared();
            if dist_sq <= threshold {
                if health.current > 0 {
                    health.current -= 1;

                    if health.current == 0 {
                        commands.set_state(GameState::GameOver);
                    } else {
                        commands.entity(player).insert(CollisionImmunity::new(1.0));
                        commands.entity(player).insert(Blink::new(50));
                    }
                }

                let dx = (enemy_pos.x - player_pos.x).abs();
                let dy = (enemy_pos.y - player_pos.y).abs();

                if dx > dy {
                    vel.target.x *= -1.;
                    vel.current.x *= -1.;
                } else {
                    vel.target.y *= -1.;
                    vel.current.y *= -1.;
                }

                break;
            }
        }
    }
}

fn player_damage_drone_system(
    mut commands: Commands,
    mut score: ResMut<Score>,
    mut player: Query<
        (&Transform, &Radius, &mut Velocity, &mut DashEffect),
        (With<Player>, Without<CollisionImmunity>),
    >,
    enemies: Query<(Entity, &Transform, &Radius, &WeakSpot), With<Enemy>>,
) {
    if let Ok((player_transform, radius, mut vel, mut dash)) = player.single_mut() {
        let player_pos = player_transform.translation.truncate();
        let player_radius = **radius;

        for (enemy, enemy_transform, radius, weak_spot) in enemies.iter() {
            let enemy_pos = enemy_transform.translation.truncate();
            let enemy_size = **radius;

            let spot_offset = weak_spot.location.to_dir() * enemy_size;
            let spot_center = enemy_pos + spot_offset;

            let spot_half_size = weak_spot.size / 2.0;
            let rotation = weak_spot.rotation;

            let rel_pos = player_pos - spot_center;
            let unrotated = rotation
                .conjugate()
                .mul_vec3(rel_pos.extend(0.0))
                .truncate();

            let dx = unrotated.x.clamp(-spot_half_size.x, spot_half_size.x) - unrotated.x;
            let dy = unrotated.y.clamp(-spot_half_size.y, spot_half_size.y) - unrotated.y;
            let dist_sq = dx * dx + dy * dy;

            if dist_sq <= player_radius * player_radius {
                commands.entity(enemy).despawn();

                score.0 += 1 + dash.combo;
                dash.combo += 1;

                if dash.power < 1.0 {
                    let dx = (enemy_pos.x - player_pos.x).abs();
                    let dy = (enemy_pos.y - player_pos.y).abs();

                    if dx > dy {
                        vel.target.x *= -1.;
                        vel.current.x *= -1.;
                    } else {
                        vel.target.y *= -1.;
                        vel.current.y *= -1.;
                    }
                }
            }
        }
    }
}

fn nuke_drone_collision_system(
    mut commands: Commands,
    mut score: ResMut<Score>,
    mut nukes: Query<(&Transform, &Radius, &mut Nuke), (With<Nuke>, Without<Enemy>)>,
    enemies: Query<(Entity, &Transform, &Radius), (With<Enemy>, Without<Nuke>)>,
) {
    for (enemy, enemy_transform, radius) in enemies.iter() {
        let enemy_pos = enemy_transform.translation;
        let enemy_size = **radius;

        for (nuke_transform, radius, mut nuke) in nukes.iter_mut() {
            let nuke_pos = nuke_transform.translation;
            let nuke_size = **radius;

            let threshold = (nuke_size + enemy_size) * (nuke_size + enemy_size);

            let dist_sq = (nuke_pos - enemy_pos).length_squared();
            if dist_sq <= threshold {
                commands.entity(enemy).despawn();

                score.0 += 1 + nuke.combo;
                nuke.combo += 1;
                break;
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
            sprite.color.set_alpha(1.);
        }
    }
}

fn blink_system(mut query: Query<(&mut Blink, &mut Sprite)>, time: Res<Time>) {
    for (mut blink, mut sprite) in query.iter_mut() {
        blink.timer.tick(time.delta());

        let alpha = if blink.timer.finished() { 1. } else { 0. };
        sprite.color.set_alpha(alpha);
    }
}
