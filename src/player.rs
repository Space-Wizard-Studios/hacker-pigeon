use bevy::{audio, prelude::*, sprite::AlphaMode2d};

use crate::{
    animation::{Animation, AnimationDir},
    asset_loader::{AudioAssets, ImageAssets},
    config::GameConfig,
    game_state::GameState,
    health::Health,
    input::{Input, MousePos},
    physics::{Radius, Velocity},
    score::Score,
};

#[derive(Component, Default, Debug)]
pub struct Player;

#[derive(Component, Default, Debug)]
pub struct ChargingDash {
    pub dir: Vec2,
    pub power: f32,
    pub sound_entity: Option<Entity>,
}

impl ChargingDash {
    pub fn new(dir: Vec2) -> Self {
        Self {
            dir,
            power: 0.,
            sound_entity: None,
        }
    }

    pub fn with_sound(mut self, sound: Entity) -> Self {
        self.sound_entity = Some(sound);
        self
    }
}

#[derive(Component, Default, Debug)]
pub struct Dashing {
    pub power: Vec2,
    pub timer: Timer,
}

impl Dashing {
    pub fn new(power: Vec2, duration_secs: f32) -> Self {
        Self {
            power,
            timer: Timer::from_seconds(duration_secs, TimerMode::Once),
        }
    }
}

#[derive(Component, Default, Debug)]
pub struct DashDirectionArrow {
    pub direction: Vec2,
    pub visibility: bool,
    pub size: f32,
    pub full_charged: bool,
}

#[derive(Component, Default, Debug)]
pub struct DashEffect {
    pub dir: Vec2,
    pub power: f32,
    pub timer: Timer,
    pub combo: u32,
}

impl DashEffect {
    pub fn new(dir: Vec2, power: f32, duration_secs: f32) -> Self {
        Self {
            dir,
            power,
            timer: Timer::from_seconds(duration_secs, TimerMode::Once),
            combo: 0,
        }
    }
}

#[derive(Component, Default, Debug)]
pub struct Nuke {
    pub timer: Timer,
    pub combo: u32,
}

impl Nuke {
    pub fn new(duration_secs: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration_secs, TimerMode::Once),
            combo: 0,
        }
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GameRunning), spawn_player)
            .add_systems(
                Update,
                (
                    player_movement_system,
                    player_dash_system,
                    player_dash_effect_system,
                    player_start_charge_dash_system,
                    player_charge_dash_system,
                    dash_arrow_system,
                    player_bounds_system,
                    player_nuke_system,
                    nuke_system,
                )
                    .chain()
                    .run_if(in_state(GameState::GameRunning)),
            );
    }
}

fn spawn_player(
    mut commands: Commands,
    mut score: ResMut<Score>,
    players: Query<Entity, With<Player>>,
    nukes: Query<Entity, With<Nuke>>,
    image_assets: Res<ImageAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    log::info!("Spawning player...");

    score.0 = 0;

    for player in &players {
        commands.entity(player).despawn();
    }

    for nuke in &nukes {
        commands.entity(nuke).despawn();
    }

    let image = image_assets.pigeon_fly_sheet.clone();
    let layout = image_assets.pigeon_fly_sheet_layout.clone();

    let animation = Animation {
        first: 0,
        last: 3,
        dir: AnimationDir::Forwards,
        timer: Timer::from_seconds(0.1, TimerMode::Repeating),
    };

    let sprite = Sprite::from_atlas_image(
        image,
        TextureAtlas {
            layout,
            index: animation.first,
        },
    );

    let mesh = meshes.add(Triangle2d::new(
        Vec2::Y * 16.0,
        Vec2::new(-4.0, 0.),
        Vec2::new(4.0, 0.),
    ));

    let material = materials.add(ColorMaterial::from(Color::srgba_u8(200, 200, 10, 0)));

    commands
        .spawn((
            Player,
            Velocity::default(),
            Transform::from_translation(Vec3::ZERO),
            Radius(16.),
            Health::new(3),
            sprite,
            animation,
        ))
        .with_children(|parent| {
            parent.spawn((
                DashDirectionArrow::default(),
                Transform::from_translation(Vec3::ZERO),
                Mesh2d(mesh),
                MeshMaterial2d(material),
            ));
        });
}

fn player_movement_system(
    input: Res<Input>,
    mut player: Query<&mut Velocity, (With<Player>, Without<ChargingDash>, Without<Dashing>)>,
    time: Res<Time>,
    config: Res<GameConfig>,
) {
    let dt = time.delta_secs();

    if let Ok(mut vel) = player.single_mut() {
        let input = input.dir();

        vel.target.x += input.x * config.player_x_acceleration * dt;
        vel.target.y += input.y * config.player_y_acceleration * dt;

        vel.target.x = vel
            .target
            .x
            .clamp(-config.player_max_x_speed, config.player_max_x_speed);
        vel.target.y = vel
            .target
            .y
            .clamp(config.player_min_fall_speed, config.player_max_rise_speed);
    }
}

fn player_bounds_system(
    mut player: Query<(&mut Transform, &mut Velocity), With<Player>>,
    config: Res<GameConfig>,
) {
    if let Ok((mut transform, mut vel)) = player.single_mut() {
        let x = transform.translation.x;

        if x > config.x_limit && vel.target.x > 0. {
            vel.target.x = 0.;
            vel.current.x = 0.;
            transform.translation.x = config.x_limit;
        } else if x < -config.x_limit && vel.target.x < 0. {
            vel.target.x = 0.;
            vel.current.x = 0.;
            transform.translation.x = -config.x_limit;
        }

        let overstep = transform.translation.y - config.ceiling_y;

        if overstep > 0.0 {
            let pull = -overstep * config.spring_force;
            vel.target.y = pull.clamp(config.max_pull, 0.0);
        }
    }
}

fn player_start_charge_dash_system(
    mut commands: Commands,
    input: Res<Input>,
    mouse_pos: Res<MousePos>,
    mut player: Query<
        (Entity, &Transform, &mut Velocity, &Children),
        (With<Player>, Without<ChargingDash>, Without<Dashing>),
    >,
    audio_assets: Res<AudioAssets>,
    mut arrows: Query<&mut DashDirectionArrow>,
) {
    if let Ok((entity, transform, mut vel, children)) = player.single_mut() {
        if input.dash() {
            vel.target = Vec2::ZERO;

            let pos = transform.translation.xy();
            let dir = (**mouse_pos - pos).normalize_or_zero();

            let sound_entity = commands
                .spawn((
                    AudioPlayer(audio_assets.dash_charging.clone()),
                    PlaybackSettings::LOOP.with_volume(audio::Volume::Linear(1.)),
                ))
                .id();

            commands
                .entity(entity)
                .insert(ChargingDash::new(dir).with_sound(sound_entity));

            for &child in children.into_iter() {
                if let Ok(mut arrow) = arrows.get_mut(child) {
                    arrow.direction = dir;
                    arrow.visibility = true;
                    arrow.size = 0.;
                    arrow.full_charged = false;
                    break;
                }
            }
        }
    }
}

fn player_charge_dash_system(
    mut commands: Commands,
    input: Res<Input>,
    mouse_pos: Res<MousePos>,
    mut player: Query<
        (Entity, &Transform, &mut ChargingDash, &Children),
        (With<Player>, Without<Dashing>),
    >,
    mut arrows: Query<&mut DashDirectionArrow>,
    audio_assets: Res<AudioAssets>,
    time: Res<Time>,
    config: Res<GameConfig>,
) {
    let dt = time.delta_secs();

    if let Ok((entity, transform, mut charging, children)) = player.single_mut() {
        if input.dash() {
            let pos = transform.translation.xy();
            let dir = (**mouse_pos - pos).normalize_or_zero();
            let power = dt / config.player_charging_power_duration;

            charging.dir = dir;
            charging.power += power;

            for &child in children.into_iter() {
                if let Ok(mut arrow) = arrows.get_mut(child) {
                    arrow.direction = dir;
                    arrow.visibility = true;
                    arrow.size += power;
                    break;
                }
            }
        } else {
            commands.entity(entity).remove::<ChargingDash>();

            if let Some(sound_entity) = charging.sound_entity.take() {
                commands.entity(sound_entity).despawn();
            }

            for &child in children.into_iter() {
                if let Ok(mut arrow) = arrows.get_mut(child) {
                    arrow.visibility = false;
                    break;
                }
            }

            let dash_power = charging.power.min(1.0);

            commands.entity(entity).insert(Dashing::new(
                charging.dir * dash_power,
                config.player_dash_duration,
            ));
            commands.entity(entity).insert(DashEffect::new(
                charging.dir,
                dash_power,
                config.player_dash_immunity_duration * dash_power,
            ));

            commands.spawn((
                AudioPlayer(audio_assets.dash_release.clone()),
                PlaybackSettings::REMOVE.with_volume(audio::Volume::Linear(1.)),
            ));
        }
    }
}

fn dash_arrow_system(
    mut commands: Commands,
    mut arrows: Query<(
        &mut DashDirectionArrow,
        &mut Transform,
        &MeshMaterial2d<ColorMaterial>,
    )>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    audio_assets: Res<AudioAssets>,
) {
    for (mut arrow, mut transform, mat) in arrows.iter_mut() {
        let pos = arrow.direction * 32.;
        transform.translation.x = pos.x;
        transform.translation.y = pos.y;

        let size = arrow.size.min(1.);

        transform.scale = Vec3::new(1., size, 1.);

        let rotation = Quat::from_rotation_arc_2d(Vec2::Y, arrow.direction);
        transform.rotation = rotation;

        if let Some(mat) = materials.get_mut(mat.id()) {
            let alpha = if arrow.visibility { 255 } else { 0 };

            let color = if size == 1.0 {
                Color::srgba_u8(200, 10, 10, alpha)
            } else {
                Color::srgba_u8(200, 200, 10, alpha)
            };

            mat.color = color;
        }

        if size == 1.0 && !arrow.full_charged {
            arrow.full_charged = true;

            commands.spawn((
                AudioPlayer(audio_assets.dash_full_charged.clone()),
                PlaybackSettings::ONCE.with_volume(audio::Volume::Linear(1.)),
            ));
        }
    }
}

fn player_dash_system(
    mut commands: Commands,
    mut player: Query<(Entity, &mut Velocity, &mut Dashing), With<Player>>,
    time: Res<Time>,
    config: Res<GameConfig>,
) {
    if let Ok((entity, mut vel, mut dash)) = player.single_mut() {
        vel.target = dash.power * config.player_dash_speed;

        dash.timer.tick(time.delta());
        if dash.timer.finished() {
            vel.target = dash.power.normalize_or_zero() * config.player_max_x_speed;
            commands.entity(entity).remove::<Dashing>();
        }
    }
}

fn player_dash_effect_system(
    mut commands: Commands,
    mut player: Query<(Entity, &mut DashEffect), With<Player>>,
    time: Res<Time>,
) {
    if let Ok((entity, mut dash)) = player.single_mut() {
        dash.timer.tick(time.delta());
        if dash.timer.finished() {
            commands.entity(entity).remove::<DashEffect>();
        }
    }
}

fn player_nuke_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    audio_assets: Res<AudioAssets>,
    player: Query<(Entity, &Transform, &DashEffect), With<Player>>,
    config: Res<GameConfig>,
) {
    if let Ok((entity, transform, dash)) = player.single() {
        if transform.translation.y <= config.floor_y + 0.05 && dash.dir.y < 0. && dash.power >= 1. {
            let mesh = meshes.add(Circle::new(96.));
            let material = materials.add(ColorMaterial {
                color: Color::srgb_u8(200, 10, 10),
                alpha_mode: AlphaMode2d::Blend,
                ..default()
            });

            commands.spawn((
                Nuke::new(1.),
                Radius(96.),
                Transform::from_translation(transform.translation),
                Mesh2d(mesh),
                MeshMaterial2d(material),
                AudioPlayer(audio_assets.boom.clone()),
                PlaybackSettings::REMOVE.with_volume(audio::Volume::Linear(0.5)),
            ));

            commands.entity(entity).remove::<Dashing>();
            commands.entity(entity).remove::<DashEffect>();
        }
    }
}

fn nuke_system(
    mut commands: Commands,
    mut nuke: Query<(Entity, &mut Nuke, &MeshMaterial2d<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    time: Res<Time>,
) {
    for (entity, mut nuke, mat) in nuke.iter_mut() {
        nuke.timer.tick(time.delta());

        if let Some(mat) = materials.get_mut(mat.id()) {
            let alpha = (nuke.timer.remaining_secs() * 255.) as u8;
            mat.color = Color::srgba_u8(0, 255, 0, alpha);
        }

        if nuke.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}
