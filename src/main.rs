use bevy::prelude::*;
use bevy::render::{
    settings::{Backends, RenderCreation, WgpuSettings},
    RenderPlugin,
};

// --- Constants ---
const PLAYER_ACCELERATION: f32 = 2000.0;
const PLAYER_SPEED: f32 = 250.0;
const PLAYER_DASH_SPEED: f32 = 1000.0;
const PLAYER_DASH_DURATION: f32 = 0.15;
const FRICTION: f32 = 5.0;

// --- Components ---

#[derive(Component)]
struct Player;

#[derive(Component, Default, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component)]
struct Dashing {
    timer: Timer,
}

#[derive(Component)]
struct Health {
    current: i32,
    max: i32,
}

#[derive(Component)]
struct HpText;

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct WeakPoint {
    direction: Vec2,
}


fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: (800.0, 600.0).into(),
                        title: "Pombo Hacker".into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        backends: Some(Backends::VULKAN),
                        ..default()
                    }),
                    ..default()
                }),
        ))
        .add_systems(Startup, (setup, setup_ui))
        .add_systems(Update, (
            player_movement_system,
            player_dash_system,
            apply_velocity_system,
            friction_system, // Adicionado para movimento suave
            update_hp_ui_system,
            collision_system,
            death_system,
        ))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.8, 0.8, 0.8),
                custom_size: Some(Vec2::new(16.0, 16.0)),
                ..default()
            },
            ..default()
        },
        Player,
        Velocity::default(),
        Health { current: 3, max: 3 },
    ));

    // Adiciona o inimigo com um ponto fraco visual
    commands
        .spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 0.2, 0.2), // Vermelho
                    custom_size: Some(Vec2::new(16.0, 16.0)),
                    ..default()
                },
                transform: Transform::from_xyz(100.0, 0.0, 0.0),
                ..default()
            },
            Enemy,
            WeakPoint { direction: Vec2::X }, // Ponto fraco à direita
        ))
        .with_children(|parent| {
            // Adiciona um sprite filho para indicar o ponto fraco
            parent.spawn(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(1.0, 0.8, 0.8), // Rosa claro
                    custom_size: Some(Vec2::new(4.0, 16.0)),
                    ..default()
                },
                transform: Transform::from_xyz(8.0, 0.0, 1.0),
                ..default()
            });
        });
}

// --- UI Systems ---

fn setup_ui(mut commands: Commands) {
    // Cria o texto para o HP
    commands.spawn((
        TextBundle::from_section(
            "HP: 3/3",
            TextStyle {
                font_size: 24.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        }),
        HpText,
    ));
}

fn update_hp_ui_system(
    player_query: Query<&Health, (With<Player>, Changed<Health>)>, 
    mut text_query: Query<&mut Text, With<HpText>>
) {
    if let Ok(player_health) = player_query.get_single() {
        if let Ok(mut text) = text_query.get_single_mut() {
            text.sections[0].value = format!("HP: {}/{}", player_health.current, player_health.max);
        }
    }
}

// --- Game Systems ---

fn collision_system(
    mut commands: Commands,
    mut player_query: Query<(&Transform, &mut Health, &Velocity, Option<&Dashing>), With<Player>>,
    enemy_query: Query<(Entity, &Transform, &Sprite, &WeakPoint), With<Enemy>>,
) {
    if let Ok((player_transform, mut player_health, player_velocity, dashing_opt)) = player_query.get_single_mut() {
        let player_size = Vec2::new(16.0, 16.0);

        for (enemy_entity, enemy_transform, enemy_sprite, weak_point) in enemy_query.iter() {
            let enemy_size = enemy_sprite.custom_size.expect("Enemy sprite has no size");

            let player_pos = player_transform.translation;
            let enemy_pos = enemy_transform.translation;

            if (player_pos.x - enemy_pos.x).abs() < (player_size.x + enemy_size.x) / 2.0
                && (player_pos.y - enemy_pos.y).abs() < (player_size.y + enemy_size.y) / 2.0
            {
                if dashing_opt.is_some() {
                    let dash_direction = player_velocity.0.normalize_or_zero();
                    if dash_direction.dot(weak_point.direction) > 0.9 {
                        commands.entity(enemy_entity).despawn();
                    }
                } else {
                    if player_health.current > 0 {
                        player_health.current -= 1;
                        commands.entity(enemy_entity).despawn();
                    }
                }
            }
        }
    }
}

fn death_system(mut commands: Commands, query: Query<(Entity, &Health)>) {
    for (entity, health) in query.iter() {
        if health.current <= 0 {
            commands.entity(entity).despawn();
        }
    }
}

fn player_movement_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Velocity, (With<Player>, Without<Dashing>)>, 
    time: Res<Time>
) {
    if let Ok(mut velocity) = query.get_single_mut() {
        let mut direction = Vec2::ZERO;

        if keyboard_input.pressed(KeyCode::KeyW) {
            direction.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            direction.y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }

        if direction != Vec2::ZERO {
            velocity.0 += direction.normalize() * PLAYER_ACCELERATION * time.delta_seconds();
        }
    }
}

fn friction_system(
    mut query: Query<&mut Velocity, (With<Player>, Without<Dashing>)>, 
    time: Res<Time>
) {
    if let Ok(mut velocity) = query.get_single_mut() {
        if velocity.0 == Vec2::ZERO {
            return;
        }
        velocity.0 *= (1.0 - FRICTION * time.delta_seconds()).max(0.0);
        if velocity.length() > PLAYER_SPEED {
            velocity.0 = velocity.normalize() * PLAYER_SPEED;
        }
        if velocity.length_squared() < 1.0 {
            velocity.0 = Vec2::ZERO;
        }
    }
}

fn player_dash_system(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(Entity, &mut Velocity, Option<&mut Dashing>), With<Player>>,
    time: Res<Time>,
) {
    if let Ok((entity, mut velocity, dashing_opt)) = query.get_single_mut() {
        if let Some(mut dashing) = dashing_opt {
            dashing.timer.tick(time.delta());
            if dashing.timer.finished() {
                commands.entity(entity).remove::<Dashing>();
            }
        } else {
            if keyboard_input.just_pressed(KeyCode::Space) {
                let dash_direction = if velocity.0 != Vec2::ZERO {
                    velocity.0.normalize()
                } else {
                    Vec2::Y
                };
                velocity.0 = dash_direction * PLAYER_DASH_SPEED;
                commands.entity(entity).insert(Dashing {
                    timer: Timer::from_seconds(PLAYER_DASH_DURATION, TimerMode::Once),
                });
            }
        }
    }
}

fn apply_velocity_system(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation.x += velocity.x * time.delta_seconds();
        transform.translation.y += velocity.y * time.delta_seconds();
    }
}
