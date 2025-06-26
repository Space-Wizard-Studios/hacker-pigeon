use bevy::prelude::*;

#[derive(Component, Default, Debug)]
pub struct Health {
    pub current: u8,
    pub max: u8,
}

impl Health {
    pub fn new(value: u8) -> Self {
        Self {
            current: value,
            max: value,
        }
    }
}

#[derive(Component, Default, Debug)]
pub struct Killed;

pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, _app: &mut App) {}
}
