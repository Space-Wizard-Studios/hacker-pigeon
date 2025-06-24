use bevy::prelude::*;
use bevy::render::{
    render_asset::RenderAssetUsages,
    settings::{Backends, RenderCreation, WgpuSettings},
    RenderPlugin,
};

// --- Constants ---
const PLAYER_ACCELERATION: f32 = 2000.0;
const PLAYER_SPEED: f32 = 250.0;
const PLAYER_DASH_SPEED: f32 = 1000.0;
const PLAYER_DASH_DURATION: f32 = 0.15;
const FRICTION: f32 = 5.0;
const CHARGING_FRICTION: f32 = 10.0;
const GRAVITY: f32 = 980.0;
const FLOOR_Y: f32 = -250.0;

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

#[derive(Component)]
struct AimArrow;

#[derive(Component)]
struct Charging {
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
            gravity_system,
            player_movement_system,
            player_charge_system,
            dashing_system,
            apply_velocity_system,
            floor_collision_system,
            friction_system, // Adicionado para movimento suave
            update_hp_ui_system,
            collision_system,
            death_system,
        ).chain())
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    // Cria uma malha de triângulo para a seta
    let mut arrow_mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    // Um triângulo isósceles apontando para a direita (ao longo do eixo X)
    arrow_mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![[12.0, 0.0, 0.0], [-6.0, 8.0, 0.0], [-6.0, -8.0, 0.0]],
    );
    arrow_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 1.0]; 3]);
    arrow_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[1.0, 0.5], [0.0, 1.0], [0.0, 0.0]]);
    let arrow_mesh_handle = meshes.add(arrow_mesh);
    let arrow_material_handle = materials.add(ColorMaterial::from(Color::YELLOW));

    // Gera o jogador com a seta como um filho
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
    )).with_children(|parent| {
        parent.spawn((
            ColorMesh2dBundle {
                mesh: arrow_mesh_handle.into(),
                material: arrow_material_handle,
                visibility: Visibility::Hidden,
                transform: Transform::from_xyz(0.0, 0.0, 2.0), // Posição relativa ao jogador
                ..default()
            },
            AimArrow,
        ));
    });

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

    // Adiciona o chão
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
    mut query: Query<&mut Velocity, (With<Player>, Without<Dashing>, Without<Charging>)>,
    time: Res<Time>
) {
    if let Ok(mut velocity) = query.get_single_mut() {
        let mut direction = Vec2::ZERO;
        if keyboard_input.pressed(KeyCode::KeyW) { direction.y += 1.0; }
        if keyboard_input.pressed(KeyCode::KeyS) { direction.y -= 1.0; }
        if keyboard_input.pressed(KeyCode::KeyA) { direction.x -= 1.0; }
        if keyboard_input.pressed(KeyCode::KeyD) { direction.x += 1.0; }
        if direction != Vec2::ZERO {
            velocity.0 += direction.normalize() * PLAYER_ACCELERATION * time.delta_seconds();
        }
    }
}

fn player_charge_system(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut player_query: Query<(Entity, &Transform, &mut Velocity, Option<&mut Charging>), With<Player>>,
    mut arrow_query: Query<(&mut Transform, &mut Visibility), (With<AimArrow>, Without<Player>)>,
    time: Res<Time>,
) {
    let Ok((entity, player_transform, mut velocity, charging_opt)) = player_query.get_single_mut() else { return };

    // Pega a posição do cursor no mundo do jogo
    let window = windows.single();
    let (camera, camera_transform) = camera_query.single();
    let cursor_pos_world = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate());

    if let Some(mut charging) = charging_opt {
        // O jogador está carregando. Aplica atrito forte e verifica a entrada.
        velocity.0 *= (1.0 - CHARGING_FRICTION * time.delta_seconds()).max(0.0);

        if keyboard_input.just_released(KeyCode::Space) {
            // Executa o dash ao soltar
            let dash_dir = charging.direction; // A direção já está normalizada
            velocity.0 = dash_dir * PLAYER_DASH_SPEED;
            commands.entity(entity).remove::<Charging>();
            commands.entity(entity).insert(Dashing {
                timer: Timer::from_seconds(PLAYER_DASH_DURATION, TimerMode::Once),
            });

            // Esconde a seta
            if let Ok((_, mut arrow_visibility)) = arrow_query.get_single_mut() {
                *arrow_visibility = Visibility::Hidden;
            }
        } else if let Some(cursor_pos) = cursor_pos_world {
            // Ainda carregando, atualiza a direção da mira com o mouse
            let player_pos = player_transform.translation.truncate();
            let new_direction = (cursor_pos - player_pos).normalize_or_zero();

            if new_direction != Vec2::ZERO {
                charging.direction = new_direction;
                // Atualiza a transformação da seta (posição e rotação)
                if let Ok((mut arrow_transform, _)) = arrow_query.get_single_mut() {
                    let offset = 25.0;
                    arrow_transform.translation = (charging.direction * offset).extend(2.0);
                    arrow_transform.rotation = Quat::from_rotation_z(charging.direction.y.atan2(charging.direction.x));
                }
            }
        }
    } else {
        // O jogador não está carregando. Verifica se deve começar a carregar.
        if keyboard_input.just_pressed(KeyCode::Space) {
            velocity.0 = Vec2::ZERO;

            // Direção inicial da mira baseada no mouse
            let mut final_direction = Vec2::Y; // Padrão
            if let Some(cursor_pos) = cursor_pos_world {
                let player_pos = player_transform.translation.truncate();
                let dir_to_cursor = (cursor_pos - player_pos).normalize_or_zero();
                if dir_to_cursor != Vec2::ZERO {
                    final_direction = dir_to_cursor;
                }
            }

            commands.entity(entity).insert(Charging { direction: final_direction });

            // Mostra e orienta a seta
            if let Ok((mut arrow_transform, mut arrow_visibility)) = arrow_query.get_single_mut() {
                *arrow_visibility = Visibility::Visible;
                let offset = 25.0;
                arrow_transform.translation = (final_direction * offset).extend(2.0);
                arrow_transform.rotation = Quat::from_rotation_z(final_direction.y.atan2(final_direction.x));
            }
        }
    }
}

fn gravity_system(mut query: Query<&mut Velocity, (With<Player>, Without<Charging>)>, time: Res<Time>) {
    if let Ok(mut velocity) = query.get_single_mut() {
        velocity.y -= GRAVITY * time.delta_seconds();
    }
}

fn floor_collision_system(mut query: Query<(&mut Transform, &mut Velocity), With<Player>>) {
    if let Ok((mut transform, mut velocity)) = query.get_single_mut() {
        if transform.translation.y < FLOOR_Y {
            transform.translation.y = FLOOR_Y;
            velocity.y = 0.0;
        }
    }
}

fn friction_system(
    mut query: Query<&mut Velocity, (With<Player>, Without<Dashing>, Without<Charging>)>,
    time: Res<Time>
) {
    if let Ok(mut velocity) = query.get_single_mut() {
        if velocity.0 == Vec2::ZERO { return; }
        velocity.0 *= (1.0 - FRICTION * time.delta_seconds()).max(0.0);
        if velocity.length() > PLAYER_SPEED {
            velocity.0 = velocity.normalize() * PLAYER_SPEED;
        }
        if velocity.length_squared() < 1.0 {
            velocity.0 = Vec2::ZERO;
        }
    }
}

fn dashing_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Dashing)>,
) {
    for (entity, mut dashing) in query.iter_mut() {
        dashing.timer.tick(time.delta());
        if dashing.timer.finished() {
            commands.entity(entity).remove::<Dashing>();
        }
    }
}

fn apply_velocity_system(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation.x += velocity.x * time.delta_seconds();
        transform.translation.y += velocity.y * time.delta_seconds();
    }
}
