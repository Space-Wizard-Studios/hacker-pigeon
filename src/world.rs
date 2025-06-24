use bevy::prelude::*;
use bevy::audio::{PlaybackMode, PlaybackSettings, Volume};
use rand::Rng;

use crate::components::*;
use crate::constants::*;

pub struct WorldPlugin;

#[derive(Resource)]
pub struct ScreenShake {
    pub intensity: f32,
    pub timer: Timer,
}

impl Default for ScreenShake {
    fn default() -> Self {
        Self {
            intensity: 0.0,
            timer: Timer::from_seconds(0.2, TimerMode::Once),
        }
    }
}

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScreenShake>()
            .add_systems(Startup, spawn_floor)
            .add_systems(
                Update,
                (
                    apply_velocity_system,
                    gravity_system,
                    floor_collision_system,
                    friction_system,
                    screen_shake_system,
                    audio_cut_system, // registra o sistema de corte de áudio
                )
                    .chain(),
            );
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
    mut query: Query<(Entity, &mut Transform, &mut Velocity, Option<&Grounded>), With<Player>>,
    mut screen_shake: ResMut<ScreenShake>,
    asset_server: Res<AssetServer>,
) {
    if let Ok((entity, mut transform, mut velocity, grounded_opt)) = query.get_single_mut() {
        if transform.translation.y < FLOOR_Y {
            let impact_velocity = velocity.y.abs();

            // Só dispara som/tremor se NÃO estava grounded
            if grounded_opt.is_none() && impact_velocity > 10.0 {
                // --- Screen Shake ---
                let shake_intensity =
                    (impact_velocity / VELOCITY_FOR_MAX_SHAKE) * MAX_SHAKE_INTENSITY;
                screen_shake.intensity = shake_intensity.clamp(0.0, MAX_SHAKE_INTENSITY);
                screen_shake.timer.reset();

                // --- Sound Effect ---
                let sound = asset_server.load("lilmati_retro-explosion-04.wav");

                // HOP: impacto leve
                if impact_velocity < HOP_VELOCITY * 1.2 {
                    // Pitch bem alto, volume bem baixo, áudio cortado
                    let hop_playback_rate = MAX_PLAYBACK_RATE * 1.5; // ainda mais agudo
                    let hop_volume = 0.01;
                    let hop_duration = 0.06; // mais rápido/seco
                    let audio_entity = commands.spawn(AudioBundle {
                        source: sound.clone(),
                        settings: PlaybackSettings {
                            mode: PlaybackMode::Once,
                            volume: Volume::new(hop_volume),
                            speed: hop_playback_rate,
                            ..default()
                        },
                    }).id();
                    // Despawning do áudio após o tempo curto
                    commands.spawn((
                        AudioCutTimer {
                            timer: Timer::from_seconds(hop_duration, TimerMode::Once),
                            audio_entity,
                        },
                    ));
                } else {
                    // Queda normal/forte: áudio padrão
                    let volume_scale = (impact_velocity / VELOCITY_FOR_MAX_VOLUME).clamp(0.05, 1.0);
                    let t = ((impact_velocity - HOP_VELOCITY)
                        / (MAX_VELOCITY_FOR_SPEED_SCALING - HOP_VELOCITY))
                        .clamp(0.0, 1.0);
                    let playback_rate = MAX_PLAYBACK_RATE + t * (MIN_PLAYBACK_RATE - MAX_PLAYBACK_RATE);
                    commands.spawn(AudioBundle {
                        source: sound,
                        settings: PlaybackSettings {
                            mode: PlaybackMode::Once,
                            volume: Volume::new(volume_scale),
                            speed: playback_rate,
                            ..default()
                        },
                    });
                }
            }

            // Sempre corrige a posição e velocidade
            transform.translation.y = FLOOR_Y;
            velocity.y = 0.0;
            // Sempre garante que está grounded
            commands.entity(entity).insert(Grounded);
        }
    }
}

#[derive(Component)]
struct AudioCutTimer {
    timer: Timer,
    audio_entity: Entity,
}

fn audio_cut_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut AudioCutTimer)>,
) {
    for (entity, mut cut_timer) in query.iter_mut() {
        cut_timer.timer.tick(time.delta());
        if cut_timer.timer.finished() {
            commands.entity(cut_timer.audio_entity).despawn_recursive();
            commands.entity(entity).despawn_recursive();
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

fn screen_shake_system(
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    mut shake: ResMut<ScreenShake>,
    time: Res<Time>,
) {
    shake.timer.tick(time.delta());
    if shake.intensity > 0.0 {
        if shake.timer.finished() {
            shake.intensity = 0.0;
        }

        // Corrigido: .percent() para .fraction_remaining()
        let shake_amount = shake.intensity * shake.timer.fraction_remaining();
        if let Ok(mut camera_transform) = camera_query.get_single_mut() {
            let mut rng = rand::thread_rng();
            // Corrigido: .gen_range() está ok na versão 0.8, mas é bom estar ciente das mudanças
            camera_transform.translation.x = rng.gen_range(-1.0..1.0) * shake_amount;
            camera_transform.translation.y = rng.gen_range(-1.0..1.0) * shake_amount;
        }
    } else {
        // Garante que a câmera volte ao centro quando o tremor acabar
        if let Ok(mut camera_transform) = camera_query.get_single_mut() {
            if camera_transform.translation.x != 0.0 || camera_transform.translation.y != 0.0 {
                camera_transform.translation.x = 0.0;
                camera_transform.translation.y = 0.0;
            }
        }
    }
}