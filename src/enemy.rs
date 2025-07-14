use bevy::prelude::*;
use bevy_egui::egui::emath::ease_in_ease_out;
use rand::Rng;

use crate::{
    animation::{Animation, AnimationDir},
    asset_loader::ImageAssets,
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
    _NorthEast,
    _SouthEast,
    _NorthWest,
    _SouthWest,
}

impl WeakSpotLocation {
    pub fn new_ortho(rng: &mut impl Rng) -> Self {
        let dir = rng.random_range(0..=3);

        match dir {
            0 => Self::North,
            1 => Self::South,
            2 => Self::West,
            _ => Self::East,
        }
    }

    pub fn _new_random(rng: &mut impl Rng) -> Self {
        let dir = rng.random_range(0..=7);

        match dir {
            0 => Self::North,
            1 => Self::South,
            2 => Self::West,
            3 => Self::East,
            4 => Self::_NorthEast,
            5 => Self::_SouthEast,
            6 => Self::_NorthWest,
            _ => Self::_SouthWest,
        }
    }

    pub fn to_dir(&self) -> Vec2 {
        match self {
            Self::North => Vec2::Y,
            Self::South => Vec2::NEG_Y,
            Self::West => Vec2::NEG_X,
            Self::East => Vec2::X,
            Self::_NorthEast => Vec2::new(1., 1.).normalize(),
            Self::_SouthEast => Vec2::new(1., -1.).normalize(),
            Self::_NorthWest => Vec2::new(-1., 1.).normalize(),
            Self::_SouthWest => Vec2::new(-1., -1.).normalize(),
        }
    }

    pub fn to_rotation(&self) -> Quat {
        use std::f32::consts::FRAC_PI_2;

        let angle = match self {
            Self::North => 0.0,
            Self::East => -FRAC_PI_2,
            Self::South => std::f32::consts::PI,
            Self::West => FRAC_PI_2,
            Self::_NorthEast => -FRAC_PI_2 * 1.5,
            Self::_SouthEast => -FRAC_PI_2 / 2.0,
            Self::_NorthWest => FRAC_PI_2 * 1.5,
            Self::_SouthWest => FRAC_PI_2 / 2.0,
        };

        Quat::from_rotation_z(angle)
    }

    pub fn to_size(&self, side: f32) -> Vec2 {
        Vec2::new(side, side / 2.)
    }

    pub fn to_atlas_index(&self) -> usize {
        match self {
            Self::North => 2,
            Self::South => 4,
            Self::West => 1,
            Self::East => 3,
            Self::_NorthEast => 4,
            Self::_SouthEast => 4,
            Self::_NorthWest => 4,
            Self::_SouthWest => 4,
        }
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

    pub fn new_ortho(rng: &mut impl Rng, side: f32) -> Self {
        let location = WeakSpotLocation::new_ortho(rng);
        let size = location.to_size(side);
        let rotation = location.to_rotation();
        Self {
            location,
            size,
            rotation,
        }
    }

    pub fn _new_random(rng: &mut impl Rng, side: f32) -> Self {
        let location = WeakSpotLocation::_new_random(rng);
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
            .add_systems(OnEnter(GameState::Running), spawn_enemies)
            .add_systems(
                Update,
                (
                    enemy_movement_system,
                    enemy_wobble_system,
                    enemy_respawn_system,
                )
                    .run_if(in_state(GameState::Running)),
            );
    }
}

fn spawn_enemies(
    mut commands: Commands,
    enemies: Query<Entity, With<Enemy>>,
    image_assets: Res<ImageAssets>,
    cfg: Res<Config>,
) {
    for enemy in &enemies {
        commands.entity(enemy).despawn();
    }

    for _ in 0..3 {
        spawn_enemy(&mut commands, &image_assets, &cfg);
    }
}

fn enemy_respawn_system(
    mut commands: Commands,
    image_assets: Res<ImageAssets>,
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
                spawn_enemy(&mut commands, &image_assets, &cfg);
            }
        }
    }
}

fn spawn_enemy(commands: &mut Commands, image_assets: &Res<ImageAssets>, cfg: &Config) {
    let mut rng = rand::rng();

    let is_ground_enemy = rng.random_bool(0.3);

    if is_ground_enemy {
        spawn_ground_enemy(commands, image_assets, &mut rng, cfg);
    } else {
        spawn_fly_enemy(commands, image_assets, &mut rng);
    }
}

fn spawn_fly_enemy(commands: &mut Commands, image_assets: &Res<ImageAssets>, rng: &mut impl Rng) {
    let x = rng.random_range(-150.0..150.0);
    let y = rng.random_range(280.0..360.0);
    let position = Vec3::new(x, y, 0.);

    let weak_spot = WeakSpot::new_ortho(rng, 16.);
    let movement = EnemyMovement::new_random(rng);
    let wobble = EnemyWobble::new_random(rng);

    let layout = image_assets.enemy_drone_layout.clone();
    let image = image_assets.enemy_drone.clone();

    let animation = Animation {
        first: 0,
        last: 0,
        dir: AnimationDir::Forwards,
        timer: Timer::from_seconds(0.1, TimerMode::Repeating),
    };

    let sprite = Sprite::from_atlas_image(
        image,
        TextureAtlas {
            layout,
            index: animation.first,
        },
    );

    let weak_layout = image_assets.enemy_drone_layout.clone();
    let weak_image = image_assets.enemy_drone.clone();

    let weak_sprite = Sprite {
        image: weak_image,
        color: Color::srgba_u8(255, 0, 0, 127),
        texture_atlas: Some(TextureAtlas {
            layout: weak_layout,
            index: weak_spot.location.to_atlas_index(),
        }),
        ..default()
    };

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
        sprite,
    ));

    entity.with_children(|parent| {
        parent.spawn((
            Transform::from_translation(Vec3::new(0., 0., 1.)),
            weak_sprite,
        ));
    });
}

fn spawn_ground_enemy(
    commands: &mut Commands,
    image_assets: &Res<ImageAssets>,
    rng: &mut impl Rng,
    cfg: &Config,
) {
    let x = rng.random_range(-150.0..150.0);
    let y = cfg.game.floor_y + 16.;
    let position = Vec3::new(x, y, 0.);

    let weak_spot = WeakSpot::new(WeakSpotLocation::South, 16.);
    let movement = EnemyMovement::new_random(rng);

    let layout = image_assets.enemy_drone_layout.clone();
    let image = image_assets.enemy_drone.clone();

    let animation = Animation {
        first: 0,
        last: 0,
        dir: AnimationDir::Forwards,
        timer: Timer::from_seconds(0.1, TimerMode::Repeating),
    };

    let sprite = Sprite::from_atlas_image(
        image,
        TextureAtlas {
            layout,
            index: animation.first,
        },
    );

    let weak_layout = image_assets.enemy_drone_layout.clone();
    let weak_image = image_assets.enemy_drone.clone();

    let weak_sprite = Sprite {
        image: weak_image,
        color: Color::srgba_u8(255, 0, 0, 127),
        texture_atlas: Some(TextureAtlas {
            layout: weak_layout,
            index: weak_spot.location.to_atlas_index(),
        }),
        ..default()
    };

    let mut entity = commands.spawn((
        Enemy,
        Velocity::default(),
        Transform::from_translation(position),
        Radius(16.),
        Health::new(1),
        movement,
        weak_spot,
        sprite,
    ));

    entity.with_children(|parent| {
        parent.spawn((
            Transform::from_translation(Vec3::new(0., 0., 1.)),
            weak_sprite,
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
