use bevy::ecs::system::ParamSet;
use bevy::prelude::*;
use crate::components::*;
use crate::constants::*;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DroneCollisionDebug::default())
            .add_systems(Update, (
                death_system,
                drone_player_collision_system,
                collision_immunity_system,
            ));
    }
}

fn death_system(mut commands: Commands, query: Query<(Entity, &Health)>) {
    for (entity, health) in query.iter() {
        if health.current <= 0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn drone_player_collision_system(
    mut commands: Commands,
    mut param_set: ParamSet<(
        Query<(Entity, &Transform, &mut Velocity, Option<&Dashing>, &mut Health, Option<&CollisionImmunity>), With<Player>>,
        Query<(Entity, &Transform, &mut Drone, &mut Velocity, Option<&CollisionImmunity>)>,
    )>,
    sprite_query: Query<&Sprite>,
    mut debug: ResMut<DroneCollisionDebug>,
) {
    // Only borrow player_query once, then drop
    let mut player_query = param_set.p0();
    let Ok((player_entity, player_transform, _, dashing_opt, _, player_immunity)) = player_query.get_single_mut() else {
        debug.last_event = String::from("[Nenhum player encontrado]");
        return;
    };
    let player_entity = player_entity;
    let player_pos = player_transform.translation.truncate();
    let is_dash = dashing_opt.is_some();
    let player_immunity = player_immunity.is_some();
    drop(player_query); // Drop borrow before drone query

    let mut player_damage = 0;
    let mut player_repel = Vec2::ZERO;
    let mut last_drone_hp = None;
    let mut last_event = String::new();
    let mut collision_happened = false;

    // Get player velocity for impact calculation
    let player_query = param_set.p0();
    drop(player_query);

    for (drone_entity, drone_transform, mut drone, mut drone_velocity, drone_immunity) in param_set.p1().iter_mut() {
        let drone_pos = drone_transform.translation.truncate();
        let distance = player_pos.distance(drone_pos);
        let collision_radius = 40.0;
        if distance < collision_radius {
            if !player_immunity && drone_immunity.is_none() {
                collision_happened = true;
                let weak_side = drone.weak_point;
                let hit_on_weak = match weak_side {
                    WeakPointSide::Left => player_pos.x < drone_pos.x,
                    WeakPointSide::Right => player_pos.x > drone_pos.x,
                    WeakPointSide::Top => player_pos.y > drone_pos.y,
                    WeakPointSide::Bottom => player_pos.y < drone_pos.y,
                };
                if is_dash && hit_on_weak {
                    last_event = format!("Dash no ponto fraco! Drone perdeu 3 HP (restante: {})", drone.hp.saturating_sub(DRONE_WEAKPOINT_DASH_DAMAGE as u8));
                    drone.hp = drone.hp.saturating_sub(DRONE_WEAKPOINT_DASH_DAMAGE as u8);
                    last_drone_hp = Some(drone.hp);
                    if drone.hp == 0 {
                        last_event.push_str(" | Drone destruído!");
                        commands.entity(drone_entity).despawn_recursive();
                    }
                    // NÃO aplica imunidade ao player nem ao drone se acertou o ponto fraco com dash
                } else {
                    last_event = format!("Colisão normal! Player perdeu 1 HP e ambos se repelem. Drone HP: {}", drone.hp);
                    player_damage += DRONE_COLLISION_DAMAGE;
                    let repel_dir = (player_pos - drone_pos).normalize_or_zero();
                    player_repel += repel_dir * DRONE_REPULSION_FORCE;
                    drone_velocity.0 -= repel_dir * DRONE_REPULSION_FORCE;
                    last_drone_hp = Some(drone.hp);
                    // Aplica imunidade e pisca para ambos, salvando a cor original
                    let player_color = sprite_query.get(player_entity).map(|s| s.color).unwrap_or(Color::rgb(0.8, 0.8, 0.8));
                    let drone_color = sprite_query.get(drone_entity).map(|s| s.color).unwrap_or(Color::rgb(1.0, 0.2, 0.2));
                    commands.entity(player_entity).insert(CollisionImmunity {
                        timer: Timer::from_seconds(COLLISION_IMMUNITY_DURATION, TimerMode::Once),
                        blink: true,
                        original_color: Some(player_color),
                    });
                    commands.entity(drone_entity).insert(CollisionImmunity {
                        timer: Timer::from_seconds(COLLISION_IMMUNITY_DURATION, TimerMode::Once),
                        blink: true,
                        original_color: Some(drone_color),
                    });
                }
                break;
            }
        }
    }
    // Re-borrow player_query to update player state
    let mut player_query = param_set.p0();
    if let Ok((_, _, mut player_velocity, _, mut player_health, _)) = player_query.get_single_mut() {
        player_health.current -= player_damage;
        player_velocity.0 += player_repel;
        debug.last_player_hp = player_health.current;
        debug.last_drone_hp = last_drone_hp;
        debug.last_event = if collision_happened { last_event } else { String::from("") };
    }
}

// Sistema para atualizar imunidade e piscar branco
fn collision_immunity_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut CollisionImmunity, &mut Sprite)>,
) {
    for (entity, mut immunity, mut sprite) in query.iter_mut() {
        immunity.timer.tick(time.delta());
        // Salva a cor original se ainda não foi salva
        if immunity.original_color.is_none() {
            immunity.original_color = Some(sprite.color);
        }
        // Pisca branco alternando entre branco e a cor original
        if immunity.blink {
            let t = immunity.timer.elapsed_secs();
            let blink_on = ((t * 12.0) as i32) % 2 == 0;
            if blink_on {
                sprite.color = COLLISION_IMMUNITY_BLINK_COLOR;
            } else if let Some(orig) = immunity.original_color {
                sprite.color = orig;
            }
        }
        if immunity.timer.finished() {
            // Restaura a cor original ao final da imunidade
            if let Some(orig) = immunity.original_color {
                sprite.color = orig;
            }
            commands.entity(entity).remove::<CollisionImmunity>();
        }
    }
}
