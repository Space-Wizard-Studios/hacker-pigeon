use std::time::Duration;

use bevy::prelude::*;

use crate::{game_state::GameState, physics::Velocity};

#[derive(Default, Debug)]
pub enum AnimationDir {
    #[default]
    Forwards,
    Backwards,
}

#[derive(Component, Default, Debug)]
pub struct Animation {
    pub first: usize,
    pub last: usize,
    pub dir: AnimationDir,
    pub timer: Timer,
}

#[derive(Component, Default, Debug)]
pub struct Blink {
    pub timer: Timer,
}

impl Blink {
    pub fn new(duration_millis: u64) -> Self {
        Self {
            timer: Timer::new(Duration::from_millis(duration_millis), TimerMode::Repeating),
        }
    }
}

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (animate_player, blink_system).run_if(in_state(GameState::GameRunning)),
        );
    }
}

fn animate_player(time: Res<Time>, mut player: Query<(&mut Animation, &mut Sprite, &Velocity)>) {
    if let Ok((mut anim, mut sprite, vel)) = player.single_mut() {
        if vel.target.x.abs() != 0. {
            sprite.flip_x = vel.target.x < 0.;
        }

        anim.timer.tick(time.delta());

        if anim.timer.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                match anim.dir {
                    AnimationDir::Forwards => {
                        atlas.index += 1;
                        if atlas.index == anim.last {
                            anim.dir = AnimationDir::Backwards;
                        }
                    }
                    AnimationDir::Backwards => {
                        atlas.index -= 1;
                        if atlas.index == anim.first {
                            anim.dir = AnimationDir::Forwards;
                        }
                    }
                }
            }
        }
    }
}

fn blink_system(mut query: Query<(&mut Blink, &mut Sprite)>, time: Res<Time>) {
    for (mut blink, mut sprite) in query.iter_mut() {
        blink.timer.tick(time.delta());
        sprite.color = Color::WHITE.with_alpha(0.);

        if blink.timer.finished() {
            sprite.color = Color::WHITE;
        }
    }
}
