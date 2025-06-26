use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_framepace::{FramepacePlugin, FramepaceSettings};

use crate::game_state::GameState;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_plugins(FramepacePlugin)
            .add_systems(OnEnter(GameState::GameRunning), (setup).chain());
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
