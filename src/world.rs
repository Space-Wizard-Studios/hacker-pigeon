use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_framepace::{FramepacePlugin, FramepaceSettings};

use crate::{game_state::GameState, player::Player};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_plugins(FramepacePlugin)
            .add_systems(OnEnter(GameState::GameRunning), (setup).chain())
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
    let x_min = -400.0;
    let x_max = 400.0;
    let y_min = -80.0;
    let y_max = 0.0;

    let smoothing_factor = 5.0;

    let dt = time.delta_secs();

    for player_transform in &players {
        let target_pos = player_transform.translation;

        for mut camera_transform in &mut cameras {
            let current_pos = camera_transform.translation;

            let target_x = target_pos.x.clamp(x_min, x_max);
            let target_y = target_pos.y.clamp(y_min, y_max);

            camera_transform.translation.x = current_pos.x.lerp(target_x, smoothing_factor * dt);
            camera_transform.translation.y = current_pos.y.lerp(target_y, smoothing_factor * dt);
        }
    }
}
