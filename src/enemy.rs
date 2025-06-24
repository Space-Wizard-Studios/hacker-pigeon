use bevy::prelude::*;
use bevy::ecs::system::ParamSet;
use rand::prelude::*;
use crate::components::{Drone, Player, WeakPointSide, Velocity, Dashing, Health, DroneHover, CollisionImmunity};
use crate::constants::*;

pub struct EnemyPlugin;

#[derive(Resource)]
pub struct DroneSpawnTimer(pub Timer);

#[derive(Default, Resource)]
pub struct DroneCollisionDebug {
    pub last_event: String,
    pub last_drone_hp: Option<u8>,
    pub last_player_hp: i32,
}

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DroneSpawnTimer(Timer::from_seconds(DRONE_SPAWN_INTERVAL, TimerMode::Repeating)))
            .insert_resource(DroneCollisionDebug::default())
            .add_systems(Update, drone_spawn_system)
            .add_systems(Update, drone_hover_system)
            .add_systems(Update, drone_player_collision_system)
            .add_systems(Update, collision_immunity_system);
    }
}

fn drone_spawn_system(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<DroneSpawnTimer>,
) {
    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        let mut rng = thread_rng();
        // Randomly select one of four sides
        let weak_point = match rng.gen_range(0..4) {
            0 => WeakPointSide::Left,
            1 => WeakPointSide::Right,
            2 => WeakPointSide::Top,
            _ => WeakPointSide::Bottom,
        };
        let x = rng.gen_range(-1800.0..1800.0);
        let y = FLOOR_Y + 120.0 + rng.gen_range(0.0..200.0);
        // Offset for weak point sprite
        let (weak_offset_x, weak_offset_y) = match weak_point {
            WeakPointSide::Left => (-16.0, 0.0),
            WeakPointSide::Right => (16.0, 0.0),
            WeakPointSide::Top => (0.0, 12.0),
            WeakPointSide::Bottom => (0.0, -12.0),
        };
        let phase = rng.gen_range(0.0..std::f32::consts::TAU);
        let hover_center = Vec2::new(x, y);
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 0.2, 0.2),
                    custom_size: Some(Vec2::new(32.0, 24.0)),
                    ..default()
                },
                transform: Transform::from_xyz(x, y, 0.0),
                ..default()
            },
            Drone { hp: DRONE_HP, weak_point },
            DroneHover { center: hover_center, phase },
            Velocity::default(),
        )).with_children(|parent| {
            parent.spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 1.0, 0.2),
                    custom_size: Some(Vec2::new(8.0, 20.0)),
                    ..default()
                },
                transform: Transform::from_xyz(weak_offset_x, weak_offset_y, 1.0),
                ..default()
            });
        });
    }
}

fn drone_hover_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &DroneHover)>, // Modificado para usar DroneHover
) {
    let t = time.elapsed_seconds();
    for (mut transform, hover) in query.iter_mut() {
        // Hover horizontal lento e com ruído
        let freq = DRONE_HOVER_FREQUENCY * 0.5 + (hover.phase.sin() * 0.1);
        let amp = DRONE_HOVER_AMPLITUDE * (0.8 + (hover.phase.cos() * 0.2));
        let noise = (t * 0.7 + hover.phase).sin() * 2.0;
        let hover_x = (t * freq + hover.phase).sin() * amp + noise;
        let hover_y = (t * (freq * 0.3) + hover.phase * 0.5).sin() * (DRONE_HOVER_AMPLITUDE * 0.15);
        transform.translation.x = hover.center.x + hover_x;
        transform.translation.y = hover.center.y + hover_y;
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
    let mut player_query = param_set.p0();
    let player_velocity_vec = if let Ok((_, _, player_velocity, _, _, _)) = player_query.get_single_mut() {
        player_velocity.0
    } else {
        Vec2::ZERO
    };
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
            commands.entity(entity).remove::<CollisionImmunity>();
            if let Some(orig) = immunity.original_color {
                sprite.color = orig;
            }
        }
    }
}
