use bevy::prelude::*;

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
    pub fn to_dir(&self) -> Vec3 {
        match self {
            WeakSpotLocation::North => Vec3::Y,
            WeakSpotLocation::South => Vec3::NEG_Y,
            WeakSpotLocation::West => Vec3::X,
            WeakSpotLocation::East => Vec3::NEG_X,
        }
    }
}

#[derive(Component, Default, Debug, Deref, DerefMut)]
pub struct WeakSpot(pub WeakSpotLocation);

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GameRunning), spawn_enemy);
    }
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
