use bevy::prelude::*;
use rand::prelude::*;
use crate::components::{Drone, WeakPointSide, Velocity, DroneHover};
use crate::constants::*;

pub struct EnemyPlugin;

#[derive(Resource)]
pub struct DroneSpawnTimer(pub Timer);

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DroneSpawnTimer(Timer::from_seconds(DRONE_SPAWN_INTERVAL, TimerMode::Repeating)))
            .add_systems(Update, drone_spawn_system)
            .add_systems(Update, drone_hover_system);
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
