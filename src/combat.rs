use bevy::prelude::*;
use crate::components::*;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (collision_system, death_system).chain());
    }
}

fn collision_system(
    mut commands: Commands,
    mut player_query: Query<(&Transform, &mut Health, &Velocity, Option<&Dashing>), With<Player>>,
    enemy_query: Query<(Entity, &Transform, &Sprite, &WeakPoint), With<Enemy>>,
) {
    if let Ok((player_transform, mut player_health, player_velocity, dashing_opt)) = player_query.get_single_mut() {
        let player_size = Vec2::new(16.0, 16.0);

        for (enemy_entity, enemy_transform, enemy_sprite, weak_point) in enemy_query.iter() {
            let enemy_size = enemy_sprite.custom_size.expect("Enemy sprite has no size");

            let player_pos = player_transform.translation;
            let enemy_pos = enemy_transform.translation;

            if (player_pos.x - enemy_pos.x).abs() < (player_size.x + enemy_size.x) / 2.0
                && (player_pos.y - enemy_pos.y).abs() < (player_size.y + enemy_size.y) / 2.0
            {
                if dashing_opt.is_some() {
                    let dash_direction = player_velocity.0.normalize_or_zero();
                    if dash_direction.dot(weak_point.direction) > 0.9 {
                        commands.entity(enemy_entity).despawn();
                    }
                } else {
                    if player_health.current > 0 {
                        player_health.current -= 1;
                        commands.entity(enemy_entity).despawn();
                    }
                }
            }
        }
    }
}

fn death_system(mut commands: Commands, query: Query<(Entity, &Health)>) {
    for (entity, health) in query.iter() {
        if health.current <= 0 {
            commands.entity(entity).despawn();
        }
    }
}
