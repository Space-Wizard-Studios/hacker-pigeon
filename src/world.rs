use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_framepace::{FramepacePlugin, FramepaceSettings};

use crate::{game_state::GameState, player::Player};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_plugins(FramepacePlugin)
            .add_systems(OnEnter(GameState::GameSetup), (setup).chain())
            .add_systems(
                Update,
                ((camera_follow).run_if(in_state(GameState::GameRunning)),),
            );
    }
}

fn setup(mut commands: Commands, mut settings: ResMut<FramepaceSettings>) {
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
}

fn camera_follow(
    players: Query<&Transform, With<Player>>,
    mut cameras: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for player_transform in &players {
        let target_pos = player_transform.translation;

        for mut camera_transform in &mut cameras {
            let current_pos = camera_transform.translation;

            let target_x = target_pos.x.clamp(MIN_X, MAX_X);
            let target_y = target_pos.y.clamp(MIN_Y, MAX_Y);

            camera_transform.translation.x = current_pos.x.lerp(target_x, CAM_SMOOTHING * dt);
            camera_transform.translation.y = current_pos.y.lerp(target_y, CAM_SMOOTHING * dt);
        }
    }
}

const MIN_Y: f32 = -80.0;
const MAX_Y: f32 = 0.0;
const MIN_X: f32 = -400.0;
const MAX_X: f32 = 400.0;
const CAM_SMOOTHING: f32 = 4.0;
