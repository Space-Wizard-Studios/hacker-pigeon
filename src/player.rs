use bevy::prelude::*;
use bevy::render::{render_asset::RenderAssetUsages, render_resource::PrimitiveTopology};

use crate::components::*;
use crate::constants::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(Update, (
                player_movement_system,
                player_charge_system,
                dashing_system,
                ability_cooldown_system,
            ));
    }
}

fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Create a triangle mesh for the aim arrow
    let mut arrow_mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    arrow_mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![[12.0, 0.0, 0.0], [-6.0, 8.0, 0.0], [-6.0, -8.0, 0.0]],
    );
    arrow_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 1.0]; 3]);
    arrow_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, vec![[1.0, 0.5], [0.0, 1.0], [0.0, 0.0]]);
    let arrow_mesh_handle = meshes.add(arrow_mesh);
    let arrow_material_handle = materials.add(ColorMaterial::from(Color::YELLOW));

    // Spawn the player with the arrow as a child
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("hacker_pigeon.png"),
            sprite: Sprite {
                custom_size: Some(Vec2::new(32.0, 32.0)),
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
                transform: Transform::from_xyz(0.0, 0.0, 2.0),
                ..default()
            },
            AimArrow,
        ));
    });
}

fn player_movement_system(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(Entity, &mut Velocity, Option<&Grounded>, Option<&AbilityCooldown>), (With<Player>, Without<Dashing>, Without<Charging>)>,
    time: Res<Time>
) {
    if let Ok((player_entity, mut velocity, grounded_opt, cooldown_opt)) = query.get_single_mut() {
        let mut direction_x = 0.0;
        if keyboard_input.pressed(KeyCode::KeyA) { direction_x -= 1.0; }
        if keyboard_input.pressed(KeyCode::KeyD) { direction_x += 1.0; }

        let is_grounded = grounded_opt.is_some();

        // --- Vertical Movement ---

        // 1. Grounded-Specific Actions
        if is_grounded {
            // Hopping movement for A/D.
            if direction_x != 0.0 {
                velocity.y = GROUND_HOP_FORCE.max(IMPACT_AUDIO_MIN_VELOCITY + 1.0); // sempre força suficiente para "descolar"
                commands.entity(player_entity).remove::<Grounded>();
            }
        }

        // 2. Burst (Spacebar) - Can be used on ground or in air.
        // This is an impulse and will override the ground hop if used on the same frame.
        if keyboard_input.just_pressed(KeyCode::Space) && cooldown_opt.is_none() {
            velocity.y = BURST_FORCE;
            commands.entity(player_entity).insert(AbilityCooldown(Timer::from_seconds(ABILITY_COOLDOWN * 0.75, TimerMode::Once)));
            if is_grounded {
                commands.entity(player_entity).remove::<Grounded>();
            }
        }

        // 3. Continuous Thrust (W) and Dive (S)

        // Continuous thrust with W (flapping)
        if keyboard_input.pressed(KeyCode::KeyW) {
            // Apply upward thrust, but cap the vertical speed from this source.
            if velocity.y < MAX_AERIAL_VERTICAL_SPEED {
                 velocity.y += AERIAL_VERTICAL_THRUST * time.delta_seconds();
            }
            if is_grounded {
                commands.entity(player_entity).remove::<Grounded>();
            }
        }
        
        // Aerial-only dive with S
        if !is_grounded {
            if keyboard_input.pressed(KeyCode::KeyS) {
                velocity.y -= AERIAL_DOWNWARD_FORCE * time.delta_seconds();
            }
        }

        // --- Horizontal Movement ---
        if direction_x != 0.0 {
            velocity.x += direction_x * PLAYER_ACCELERATION * time.delta_seconds();
        }
    }
}

fn player_charge_system(
    mut commands: Commands,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut player_query: Query<(Entity, &Transform, &mut Velocity, Option<&mut Charging>, Option<&AbilityCooldown>, Option<&Grounded>), With<Player>>,
    mut arrow_query: Query<(&mut Transform, &mut Visibility), (With<AimArrow>, Without<Player>)>,
    time: Res<Time>,
) {
    let Ok((entity, player_transform, mut velocity, charging_opt, cooldown_opt, grounded_opt)) = player_query.get_single_mut() else { return };

    let window = windows.single();
    let (camera, camera_transform) = camera_query.single();
    let cursor_pos_world = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate());

    if let Some(mut charging) = charging_opt {
        velocity.0 *= (1.0 - CHARGING_FRICTION * time.delta_seconds()).max(0.0);

        if mouse_button_input.just_released(MouseButton::Left) {
            let dash_dir = charging.direction;
            velocity.0 = dash_dir * PLAYER_DASH_SPEED;
            // Se dash tem componente vertical relevante, remove Grounded
            if grounded_opt.is_some() && dash_dir.y.abs() > 0.2 {
                commands.entity(entity).remove::<Grounded>();
            }
            commands.entity(entity).remove::<Charging>();
            commands.entity(entity).insert(Dashing {
                timer: Timer::from_seconds(PLAYER_DASH_DURATION, TimerMode::Once),
            });
            commands.entity(entity).insert(AbilityCooldown(Timer::from_seconds(ABILITY_COOLDOWN, TimerMode::Once)));

            if let Ok((_, mut arrow_visibility)) = arrow_query.get_single_mut() {
                *arrow_visibility = Visibility::Hidden;
            }
        } else if let Some(cursor_pos) = cursor_pos_world {
            let player_pos = player_transform.translation.truncate();
            let new_direction = (cursor_pos - player_pos).normalize_or_zero();

            if new_direction != Vec2::ZERO {
                charging.direction = new_direction;
                if let Ok((mut arrow_transform, _)) = arrow_query.get_single_mut() {
                    let offset = 25.0;
                    arrow_transform.translation = (charging.direction * offset).extend(2.0);
                    arrow_transform.rotation = Quat::from_rotation_z(charging.direction.y.atan2(charging.direction.x));
                }
            }
        }
    } else {
        if mouse_button_input.just_pressed(MouseButton::Left) && cooldown_opt.is_none() {
            velocity.0 = Vec2::ZERO;

            if let Some(cursor_pos) = cursor_pos_world {
                let player_pos = player_transform.translation.truncate();
                let dir_to_cursor = (cursor_pos - player_pos).normalize_or_zero();
                if dir_to_cursor != Vec2::ZERO {
                    commands.entity(entity).insert(Charging { direction: dir_to_cursor });
                }
            }

            if let Ok((mut arrow_transform, mut arrow_visibility)) = arrow_query.get_single_mut() {
                *arrow_visibility = Visibility::Visible;
                let offset = 25.0;
                let initial_direction = (cursor_pos_world.unwrap_or_default() - player_transform.translation.truncate()).normalize_or_zero();
                arrow_transform.translation = (initial_direction * offset).extend(2.0);
                arrow_transform.rotation = Quat::from_rotation_z(initial_direction.y.atan2(initial_direction.x));
            }
        }
    }
}

fn ability_cooldown_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Sprite, Option<&mut AbilityCooldown>), With<Player>>,
) {
    if let Ok((entity, mut sprite, cooldown_opt)) = query.get_single_mut() {
        if let Some(mut cooldown) = cooldown_opt {
            sprite.color = Color::BLUE;
            cooldown.0.tick(time.delta());
            if cooldown.0.finished() {
                commands.entity(entity).remove::<AbilityCooldown>();
            }
        } else {
            sprite.color = Color::rgb(0.8, 0.8, 0.8);
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
