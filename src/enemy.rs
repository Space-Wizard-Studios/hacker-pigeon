use bevy::prelude::*;
use bevy_egui::egui::emath::ease_in_ease_out;
use rand::Rng;

use crate::{
    config::Config,
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

impl EnemyMovement {
    pub fn new_random(rng: &mut impl Rng) -> Self {
        let period = rng.random_range(4.0..8.0);
        let speed = rng.random_range(80.0..100.0);
        Self { period, speed }
    }
}

#[derive(Component, Default, Debug)]
pub struct EnemyWobble {
    base_y: f32,
    spread: f32,
    timer: Timer,
    target_y: f32,
}

impl EnemyWobble {
    pub fn new_random(rng: &mut impl Rng) -> Self {
        let base_y = rng.random_range(140.0..180.0);
        let spread = rng.random_range(15.0..40.0);
        let wobble_time = rng.random_range(1.2..2.0);
        Self {
            base_y,
            spread,
            timer: Timer::from_seconds(wobble_time, TimerMode::Repeating),
            target_y: base_y,
        }
    }
}

#[derive(Default, Debug)]
pub enum WeakSpotLocation {
    North,
    #[default]
    South,
    West,
    East,
    NorthEast,
    SouthEast,
    NorthWest,
    SouthWest,
}

impl WeakSpotLocation {
    pub fn new_random(rng: &mut impl Rng) -> Self {
        let dir = rng.random_range(0..=7);

        match dir {
            0 => Self::North,
            1 => Self::South,
            2 => Self::West,
            3 => Self::East,
            4 => Self::NorthEast,
            5 => Self::SouthEast,
            6 => Self::NorthWest,
            _ => Self::SouthWest,
        }
    }

    pub fn to_dir(&self) -> Vec2 {
        match self {
            Self::North => Vec2::Y,
            Self::South => Vec2::NEG_Y,
            Self::West => Vec2::X,
            Self::East => Vec2::NEG_X,
            Self::NorthEast => Vec2::new(-1., 1.).normalize(),
            Self::SouthEast => Vec2::new(-1., -1.).normalize(),
            Self::NorthWest => Vec2::new(1., 1.).normalize(),
            Self::SouthWest => Vec2::new(-1., 1.).normalize(),
        }
    }

    pub fn to_rotation(&self) -> Quat {
        use std::f32::consts::FRAC_PI_2;

        let angle = match self {
            Self::North => 0.0,
            Self::East => -FRAC_PI_2,
            Self::South => std::f32::consts::PI,
            Self::West => FRAC_PI_2,
            Self::NorthEast => -FRAC_PI_2 * 1.5,
            Self::SouthEast => -FRAC_PI_2 / 2.0,
            Self::NorthWest => FRAC_PI_2 * 1.5,
            Self::SouthWest => FRAC_PI_2 / 2.0,
        };

        Quat::from_rotation_z(angle)
    }

    pub fn to_size(&self, side: f32) -> Vec2 {
        Vec2::new(side, side / 2.)
    }
}

#[derive(Component, Default, Debug)]
pub struct WeakSpot {
    pub location: WeakSpotLocation,
    pub size: Vec2,
    pub rotation: Quat,
}

impl WeakSpot {
    pub fn new(location: WeakSpotLocation, side: f32) -> Self {
        let size = location.to_size(side);
        let rotation = location.to_rotation();
        Self {
            location,
            size,
            rotation,
        }
    }

    pub fn new_random(rng: &mut impl Rng, side: f32) -> Self {
        let location = WeakSpotLocation::new_random(rng);
        let size = location.to_size(side);
        let rotation = location.to_rotation();
        Self {
            location,
            size,
            rotation,
        }
    }
}

#[derive(Resource, Default)]
struct EnemyRespawnTimer {
    timer: Option<Timer>,
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
    enemies: Query<Entity, With<Enemy>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    cfg: Res<Config>,
) {
    for enemy in &enemies {
        commands.entity(enemy).despawn();
    }

    for _ in 0..3 {
        spawn_enemy(&mut commands, &mut meshes, &mut materials, &cfg);
    }
}

fn enemy_respawn_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    enemies: Query<(), With<Enemy>>,
    mut respawn_state: ResMut<EnemyRespawnTimer>,
    time: Res<Time>,
    cfg: Res<Config>,
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
                spawn_enemy(&mut commands, &mut meshes, &mut materials, &cfg);
            }
        }
    }
}

fn spawn_enemy(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    cfg: &Config,
) {
    let mut rng = rand::rng();

    let is_ground_enemy = rng.random_bool(0.3);

    if is_ground_enemy {
        spawn_ground_enemy(commands, meshes, materials, &mut rng, cfg);
    } else {
        spawn_fly_enemy(commands, meshes, materials, &mut rng);
    }
}

fn spawn_fly_enemy(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    rng: &mut impl Rng,
) {
    let x = rng.random_range(-150.0..150.0);
    let y = rng.random_range(280.0..360.0);
    let position = Vec3::new(x, y, 0.);

    let weak_spot = WeakSpot::new_random(rng, 16.);
    let weak_spot_pos = weak_spot.location.to_dir() * 20.;
    let weak_spot_rot = weak_spot.rotation;
    let weak_spot_size = weak_spot.size;

    let movement = EnemyMovement::new_random(rng);
    let wobble: EnemyWobble = EnemyWobble::new_random(rng);

    let mesh = meshes.add(Circle::new(16.));
    let material = materials.add(ColorMaterial::from_color(Color::srgb_u8(200, 10, 10)));

    let mut entity = commands.spawn((
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
    ));

    entity.with_children(|parent| {
        parent.spawn((
            Transform::from_translation(weak_spot_pos.extend(0.)).with_rotation(weak_spot_rot),
            Sprite {
                color: Color::srgb_u8(200, 200, 10),
                custom_size: Some(weak_spot_size),
                ..default()
            },
        ));
    });
}

fn spawn_ground_enemy(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    rng: &mut impl Rng,
    cfg: &Config,
) {
    let x = rng.random_range(-150.0..150.0);
    let y = cfg.game.floor_y + 16.;
    let position = Vec3::new(x, y, 0.);

    let weak_spot = WeakSpot::new(WeakSpotLocation::South, 16.);
    let weak_spot_pos = weak_spot.location.to_dir() * 16.;
    let weak_spot_rot = weak_spot.rotation;
    let weak_spot_size = weak_spot.size;

    let movement = EnemyMovement::new_random(rng);

    let mesh = meshes.add(Circle::new(16.));
    let material = materials.add(ColorMaterial::from_color(Color::srgb_u8(200, 10, 10)));

    let mut entity = commands.spawn((
        Enemy,
        Velocity::default(),
        Transform::from_translation(position),
        Radius(16.),
        Health::new(1),
        movement,
        weak_spot,
        Mesh2d(mesh),
        MeshMaterial2d(material),
    ));

    entity.with_children(|parent| {
        parent.spawn((
            Transform::from_translation(weak_spot_pos.extend(0.)).with_rotation(weak_spot_rot),
            Sprite {
                color: Color::srgb_u8(200, 200, 10),
                custom_size: Some(weak_spot_size),
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
