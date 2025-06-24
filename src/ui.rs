use bevy::prelude::*;
use crate::components::*;
use crate::constants::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui)
           .add_systems(Update, (update_hp_ui_system, update_debug_ui_system));
    }
}

fn setup_ui(mut commands: Commands) {
    // HP Text
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

    // Debug Text
    commands.spawn((
        TextBundle::from_section(
            "", // Deixado em branco, será preenchido pelo sistema de atualização
            TextStyle {
                font_size: 18.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
        DebugText,
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

fn update_debug_ui_system(
    player_query: Query<(
        &Transform,
        &Velocity,
        Option<&AbilityCooldown>,
        Option<&Charging>,
        Option<&Grounded>,
        Option<&CollisionImmunity>,
    ), With<Player>>,
    mut text_query: Query<&mut Text, With<DebugText>>,
    landing_audio_debug: Res<crate::world::LandingAudioDebug>,
    drone_debug: Res<crate::components::DroneCollisionDebug>,
    drone_query: Query<Option<&CollisionImmunity>, With<crate::components::Drone>>,
) {
    if let Ok((transform, velocity, cooldown_opt, charging_opt, grounded_opt, player_immunity)) = player_query.get_single() {
        if let Ok(mut text) = text_query.get_single_mut() {
            let cd = cooldown_opt.map_or(0.0, |cd| cd.0.remaining_secs());
            let pos = transform.translation;
            let vel = velocity.0;
            let friction = if grounded_opt.is_some() { GROUND_FRICTION } else { FRICTION };
            let charge_angle = charging_opt.map_or(0.0, |c| c.direction.angle_between(Vec2::X).to_degrees());
            let state = if grounded_opt.is_some() {
                "Grounded"
            } else if charging_opt.is_some() {
                "Charging"
            } else {
                "Airborne"
            };
            let audio_dbg = &*landing_audio_debug;
            let audio_str = if audio_dbg.last_impact_velocity > 0.0 {
                format!(
                    "\n[Landing Audio]\nvel: {:.1}  vol: {:.2}  pitch: {:.2}  hop: {}",
                    audio_dbg.last_impact_velocity,
                    audio_dbg.last_volume,
                    audio_dbg.last_playback_rate,
                    audio_dbg.last_is_hop
                )
            } else {
                String::new()
            };
            let drone_dbg = &*drone_debug;
            let drone_str = if !drone_dbg.last_event.is_empty() {
                format!(
                    "\n[Drone Debug]\n{}\nPlayer HP: {}\nDrone HP: {:?}",
                    drone_dbg.last_event,
                    drone_dbg.last_player_hp,
                    drone_dbg.last_drone_hp
                )
            } else {
                String::new()
            };
            // Imunidade do player
            let player_imm_str = if let Some(im) = player_immunity {
                format!("\nPlayer Imune: {:.2}s", im.timer.remaining_secs())
            } else {
                String::new()
            };
            // Imunidade de um drone (mostra o primeiro encontrado)
            let drone_imm_str = if let Some(Some(im)) = drone_query.iter().find(|imm| imm.is_some()) {
                format!("\nDrone Imune: {:.2}s", im.timer.remaining_secs())
            } else {
                String::new()
            };
            text.sections[0].value = format!(
                "CD: {:.2}\nPos: {:.1}, {:.1}\nSpeed: {:.1}, {:.1}\nFriction {:.1}\nDash Angle: {:.1}\nState: {}{}{}{}{}",
                cd,
                pos.x, pos.y,
                vel.x, vel.y,
                friction,
                charge_angle,
                state,
                audio_str,
                drone_str,
                player_imm_str,
                drone_imm_str
            );
        }
    }
}
