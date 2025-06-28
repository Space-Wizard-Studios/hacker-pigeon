use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_framepace::{FramepacePlugin, FramepaceSettings};

use crate::{asset_loader::ImageAssets, config::GameConfig, game_state::GameState, player::Player};

#[derive(Component)]
pub struct Parallax {
    pub factor: f32,
}

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .insert_resource(ClearColor(Color::srgb_u8(0, 9, 39)))
            .add_plugins(FramepacePlugin)
            .add_systems(OnEnter(GameState::GameSetup), (setup).chain())
            .add_systems(
                Update,
                ((camera_follow, parallax_system).run_if(in_state(GameState::GameRunning)),),
            );
    }
}

fn setup(
    mut commands: Commands,
    mut settings: ResMut<FramepaceSettings>,
    images: Res<ImageAssets>,
) {
    use bevy_framepace::Limiter;
    settings.limiter = Limiter::from_framerate(60.0);

    let cam = Camera2d;
    let projection = Projection::Orthographic(OrthographicProjection {
        scaling_mode: ScalingMode::FixedVertical {
            viewport_height: 480.,
        },
        ..OrthographicProjection::default_2d()
    });

    commands.spawn((cam, projection));
    commands.set_state(GameState::GameRunning);

    let sky_image = images.bg_sky.clone();
    let sky_sprite = Sprite {
        image: sky_image,
        image_mode: SpriteImageMode::Tiled {
            tile_x: true,
            tile_y: false,
            stretch_value: 0.33,
        },
        ..default()
    };

    let pos = Vec3::new(0., 80., -1000.);
    let sca = Vec3::new(6., 1., 1.);
    commands.spawn((sky_sprite, Transform::from_translation(pos).with_scale(sca)));

    let ground_image = images.bg_ground.clone();
    let ground_sprite = Sprite {
        image: ground_image,
        image_mode: SpriteImageMode::Tiled {
            tile_x: true,
            tile_y: false,
            stretch_value: 0.25,
        },
        ..default()
    };

    let pos = Vec3::new(0., -205., -500.);
    let sca = Vec3::new(8., 1., 1.);
    commands.spawn((
        ground_sprite,
        Transform::from_translation(pos).with_scale(sca),
    ));

    let buildings_image = images.bg_buildings.clone();
    let buildings_sprite = Sprite {
        image: buildings_image,
        image_mode: SpriteImageMode::Tiled {
            tile_x: true,
            tile_y: false,
            stretch_value: 0.25,
        },
        ..default()
    };

    let pos = Vec3::new(0., -45., -900.);
    let sca = Vec3::new(8., 2., 1.);
    commands.spawn((
        buildings_sprite,
        Transform::from_translation(pos).with_scale(sca),
        Parallax { factor: 0.06 },
    ));

    let buildings_image = images.bg_buildings.clone();
    let buildings_sprite = Sprite {
        image: buildings_image,
        image_mode: SpriteImageMode::Tiled {
            tile_x: true,
            tile_y: false,
            stretch_value: 0.18,
        },
        color: Color::srgb_u8(69, 176, 209),
        ..default()
    };

    let pos = Vec3::new(0., -65., -901.);
    let sca = Vec3::new(8.5, 1.8, 1.);
    commands.spawn((
        buildings_sprite,
        Transform::from_translation(pos).with_scale(sca),
        Parallax { factor: 0.03 },
    ));

    let buildings_image = images.bg_buildings.clone();
    let buildings_sprite = Sprite {
        image: buildings_image,
        image_mode: SpriteImageMode::Tiled {
            tile_x: true,
            tile_y: false,
            stretch_value: 0.12,
        },
        color: Color::srgb_u8(119, 150, 181),
        ..default()
    };

    let pos = Vec3::new(0., -85., -902.);
    let sca = Vec3::new(9., 1.6, 1.);
    commands.spawn((
        buildings_sprite,
        Transform::from_translation(pos).with_scale(sca),
        Parallax { factor: 0.01 },
    ));
}

fn camera_follow(
    players: Query<&Transform, With<Player>>,
    mut cameras: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    time: Res<Time>,
    config: Res<GameConfig>,
) {
    let dt = time.delta_secs();

    for player_transform in &players {
        let target_pos = player_transform.translation;

        for mut camera_transform in &mut cameras {
            let current_pos = camera_transform.translation;

            let target_x = target_pos.x.clamp(config.cam_min_x, config.cam_max_x);
            let target_y = target_pos.y.clamp(config.cam_min_y, config.cam_max_y);

            camera_transform.translation.x =
                current_pos.x.lerp(target_x, config.cam_smoothing * dt);
            camera_transform.translation.y =
                current_pos.y.lerp(target_y, config.cam_smoothing * dt);
        }
    }
}

fn parallax_system(
    camera_q: Query<&Transform, (With<Camera2d>, Without<Parallax>)>,
    mut parallax_q: Query<(&Parallax, &mut Transform)>,
) {
    if let Ok(camera_transform) = camera_q.single() {
        for (parallax, mut transform) in parallax_q.iter_mut() {
            transform.translation.x = -camera_transform.translation.x * parallax.factor;
        }
    }
}
