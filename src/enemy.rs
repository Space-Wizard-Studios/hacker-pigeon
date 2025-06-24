use bevy::prelude::*;
use crate::components::*;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_enemy);
    }
}

fn spawn_enemy(mut commands: Commands) {
    commands
        .spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 0.2, 0.2),
                    custom_size: Some(Vec2::new(16.0, 16.0)),
                    ..default()
                },
                transform: Transform::from_xyz(100.0, 0.0, 0.0),
                ..default()
            },
            Enemy,
            WeakPoint { direction: Vec2::X },
        ))
        .with_children(|parent| {
            parent.spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 0.8, 0.8),
                    custom_size: Some(Vec2::new(4.0, 16.0)),
                    ..default()
                },
                transform: Transform::from_xyz(8.0, 0.0, 1.0),
                ..default()
            });
        });
}
