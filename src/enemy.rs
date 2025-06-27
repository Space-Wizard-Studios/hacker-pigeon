use bevy::prelude::*;
use rand::Rng;

use crate::{
    game_state::GameState,
    health::Health,
    physics::{Airborne, Radius, Velocity},
};

#[derive(Component, Default, Debug)]
pub struct Enemy;

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
    pub size: f32,
}

impl WeakSpot {
    pub fn new(size: f32) -> Self {
        Self {
            location: WeakSpotLocation::random(),
            size,
        }
    }
}

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GameRunning), spawn_enemy);
    }
}

fn spawn_enemy(mut commands: Commands) {
    let weak_spot = WeakSpot::new(16.);
    let weak_spot_pos = weak_spot.location.to_dir() * 16.;

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

    let weak_spot = WeakSpot::new(16.);
    let weak_spot_pos = weak_spot.location.to_dir() * 16.;

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

    let weak_spot = WeakSpot::new(16.);
    let weak_spot_pos = weak_spot.location.to_dir() * 16.;

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
