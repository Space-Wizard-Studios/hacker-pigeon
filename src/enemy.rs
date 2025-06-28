use bevy::prelude::*;
use bevy_egui::egui::emath::ease_in_ease_out;
use rand::Rng;

use crate::{
    game_state::GameState,
    health::Health,
    physics::{Airborne, Radius, Velocity},
};

#[derive(Component, Default, Debug)]
pub struct Enemy;

#[derive(Component, Default, Debug)]
pub struct EnemyMovement {
    period: f32,
    speed: f32,
}

#[derive(Component, Default, Debug)]
pub struct EnemyWobble {
    base_y: f32,
    spread: f32,
    timer: Timer,
    target_y: f32,
}

#[derive(Default, Debug)]
pub enum WeakSpotLocation {
    North,
    #[default]
    South,
    West,
    East,
}

impl WeakSpotLocation {
    pub fn random() -> Self {
        let mut rng = rand::rng();
        let dir = rng.random_range(0..4);

        match dir {
            0 => WeakSpotLocation::North,
            1 => WeakSpotLocation::South,
            2 => WeakSpotLocation::West,
            _ => WeakSpotLocation::East,
        }
    }

    pub fn to_dir(&self) -> Vec3 {
        match self {
            WeakSpotLocation::North => Vec3::Y,
            WeakSpotLocation::South => Vec3::NEG_Y,
            WeakSpotLocation::West => Vec3::X,
            WeakSpotLocation::East => Vec3::NEG_X,
        }
    }
}

#[derive(Component, Default, Debug)]
pub struct WeakSpot {
    pub location: WeakSpotLocation,
    pub size: Vec2,
}

impl WeakSpot {
    pub fn new(size: Vec2) -> Self {
        Self {
            location: WeakSpotLocation::random(),
            size,
        }
    }
}

#[derive(Resource)]
struct EnemyRespawnTimer {
    timer: Option<Timer>,
}

impl Default for EnemyRespawnTimer {
    fn default() -> Self {
        Self { timer: None }
    }
}

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EnemyRespawnTimer::default())
            .add_systems(OnEnter(GameState::GameRunning), spawn_enemies)
            .add_systems(
                Update,
                (
                    enemy_movement_system,
                    enemy_wobble_system,
                    enemy_respawn_system,
                )
                    .run_if(in_state(GameState::GameRunning)),
            );
    }
}

fn spawn_enemies(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    spawn_enemy(
        &mut commands,
        Vec3::new(0., 280., 0.),
        EnemyMovement {
            period: 6.,
            speed: 120.,
        },
        EnemyWobble {
            base_y: 160.,
            spread: 30.,
            timer: Timer::from_seconds(1.5, TimerMode::Repeating),
            target_y: 160.,
        },
        &mut meshes,
        &mut materials,
    );
    spawn_enemy(
        &mut commands,
        Vec3::new(180., 280., 0.),
        EnemyMovement {
            period: 4.,
            speed: 130.,
        },
        EnemyWobble {
            base_y: 180.,
            spread: 20.,
            timer: Timer::from_seconds(1.7, TimerMode::Repeating),
            target_y: 180.,
        },
        &mut meshes,
        &mut materials,
    );
    spawn_enemy(
        &mut commands,
        Vec3::new(-180., 280., 0.),
        EnemyMovement {
            period: 8.,
            speed: 80.,
        },
        EnemyWobble {
            base_y: 160.,
            spread: 35.,
            timer: Timer::from_seconds(1.5, TimerMode::Repeating),
            target_y: 160.,
        },
        &mut meshes,
        &mut materials,
    );
    spawn_enemy(
        &mut commands,
        Vec3::new(-120., 280., 0.),
        EnemyMovement {
            period: 7.,
            speed: 90.,
        },
        EnemyWobble {
            base_y: 150.,
            spread: 40.,
            timer: Timer::from_seconds(1.6, TimerMode::Repeating),
            target_y: 150.,
        },
        &mut meshes,
        &mut materials,
    );
}

fn enemy_respawn_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    enemies: Query<(), With<Enemy>>,
    mut respawn_state: ResMut<EnemyRespawnTimer>,
    time: Res<Time>,
) {
    if enemies.iter().count() > 3 {
        respawn_state.timer = None;
        return;
    }

    if respawn_state.timer.is_none() {
        let mut rng = rand::rng();
        let duration = rng.random_range(2.0..4.0);
        respawn_state.timer = Some(Timer::from_seconds(duration, TimerMode::Once));
    }

    if let Some(timer) = respawn_state.timer.as_mut() {
        timer.tick(time.delta());
        if timer.finished() {
            let mut rng = rand::rng();

            let n = rng.random_range(1..=3);
            for _ in 0..n {
                let x = rng.random_range(-150.0..150.0);
                let y = rng.random_range(280.0..360.0);
                let base_y = rng.random_range(140.0..180.0);
                let spread = rng.random_range(15.0..40.0);
                let period = rng.random_range(4.0..8.0);
                let speed = rng.random_range(80.0..100.0);
                let wobble_time = rng.random_range(1.2..2.0);

                spawn_enemy(
                    &mut commands,
                    Vec3::new(x, y, 0.),
                    EnemyMovement { period, speed },
                    EnemyWobble {
                        base_y,
                        spread,
                        timer: Timer::from_seconds(wobble_time, TimerMode::Repeating),
                        target_y: base_y,
                    },
                    &mut meshes,
                    &mut materials,
                );
            }
        }
    }
}

fn spawn_enemy(
    commands: &mut Commands,
    position: Vec3,
    movement: EnemyMovement,
    wobble: EnemyWobble,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let weak_spot = WeakSpot::new(Vec2::new(6., 6.));
    let weak_spot_pos = weak_spot.location.to_dir() * 16.;

    let mesh = meshes.add(Circle::new(16.));
    let material = materials.add(ColorMaterial::from_color(Color::srgb_u8(200, 10, 10)));

    commands
        .spawn((
            Enemy,
            Velocity::default(),
            Transform::from_translation(position),
            Radius(16.),
            Health::new(1),
            movement,
            wobble,
            Airborne,
            weak_spot,
            Mesh2d(mesh),
            MeshMaterial2d(material),
        ))
        .with_children(|parent| {
            parent.spawn((
                Transform::from_translation(Vec3::new(weak_spot_pos.x, weak_spot_pos.y, 0.)),
                Sprite {
                    color: Color::srgb_u8(200, 200, 10),
                    custom_size: Some(Vec2::splat(12.)),
                    ..default()
                },
            ));
        });
}

fn enemy_movement_system(
    mut enemies: Query<(&mut Velocity, &EnemyMovement), With<Enemy>>,
    time: Res<Time>,
) {
    let timer = time.elapsed_secs();

    for (mut vel, movement) in enemies.iter_mut() {
        let period = movement.period;

        let phase = (timer / period) * std::f32::consts::PI;
        let dir = phase.sin().signum();

        let t = (timer % period) / period;
        let speed = ease_in_ease_out(t) * 2. - 1.;

        vel.target.x = movement.speed * speed * dir;
    }
}

fn enemy_wobble_system(
    mut enemies: Query<(&mut Velocity, &mut EnemyWobble, &Transform, &EnemyMovement), With<Enemy>>,
    time: Res<Time>,
) {
    let dt = time.delta();
    let mut rng = rand::rng();

    for (mut vel, mut wobble, transform, movement) in enemies.iter_mut() {
        wobble.timer.tick(dt);
        if wobble.timer.finished() {
            let new_target = wobble.base_y + (rng.random_range(0f32..2f32) - 1.) * wobble.spread;
            wobble.target_y = new_target;
        }

        let dy = wobble.target_y - transform.translation.y;
        if dy.abs() < 1. {
            vel.target.y = 0.;
        } else {
            vel.target.y = movement.speed * dy.signum();
        }
    }
}
