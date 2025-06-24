use bevy::prelude::*;
use crate::components::*;
use crate::constants::*;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_floor)
            .add_systems(Update, (
                apply_velocity_system,
                gravity_system,
                floor_collision_system,
                friction_system,
            ).chain());
    }
}

fn spawn_floor(mut commands: Commands) {
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.5, 0.5, 0.5),
            custom_size: Some(Vec2::new(800.0, 50.0)),
            ..default()
        },
        transform: Transform::from_xyz(0.0, FLOOR_Y - 25.0, 0.0),
        ..default()
    });
}

fn gravity_system(
    mut query: Query<(&mut Velocity, Option<&Charging>), With<Player>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    if let Ok((mut velocity, charging_opt)) = query.get_single_mut() {
        let gravity_multiplier = if charging_opt.is_some() {
            CHARGING_GRAVITY_MULTIPLIER
        } else if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::KeyD) {
            LATERAL_GRAVITY_MULTIPLIER
        } else {
            1.0
        };

        velocity.y -= GRAVITY * gravity_multiplier * time.delta_seconds();
    }
}

fn floor_collision_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Velocity), With<Player>>
) {
    if let Ok((entity, mut transform, mut velocity)) = query.get_single_mut() {
        if transform.translation.y < FLOOR_Y {
            transform.translation.y = FLOOR_Y;
            velocity.y = 0.0;
            commands.entity(entity).insert(Grounded);
        }
    }
}

fn friction_system(
    mut query: Query<(&mut Velocity, Option<&Grounded>), (With<Player>, Without<Dashing>, Without<Charging>)>,
    time: Res<Time>
) {
    if let Ok((mut velocity, grounded_opt)) = query.get_single_mut() {
        let friction = if grounded_opt.is_some() {
            GROUND_FRICTION
        } else {
            FRICTION
        };

        velocity.x *= (1.0 - friction * time.delta_seconds()).max(0.0);
        if velocity.x.abs() > PLAYER_SPEED {
            velocity.x = velocity.x.signum() * PLAYER_SPEED;
        }
        if velocity.x.abs() < 1.0 {
            velocity.x = 0.0;
        }
    }
}

fn apply_velocity_system(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation.x += velocity.x * time.delta_seconds();
        transform.translation.y += velocity.y * time.delta_seconds();
    }
}
