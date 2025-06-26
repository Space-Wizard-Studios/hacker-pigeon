use bevy::prelude::*;

#[derive(Resource, Default, Debug)]
pub struct Score(pub u32);

pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Score>();
    }
}
