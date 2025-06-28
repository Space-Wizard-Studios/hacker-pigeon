use bevy::prelude::*;
use serde::Deserialize;
use std::fs;

#[derive(Resource, Debug, Deserialize)]
pub struct GameConfig {
    pub gravity: f32,
    pub charging_gravity_multiplier: f32,
    pub movement_smoothing: f32,
    pub air_friction: f32,
    pub ground_friction: f32,
    pub floor_y: f32,
    pub x_limit: f32,
    pub ceiling_y: f32,
    pub spring_force: f32,
    pub max_pull: f32,

    pub player_x_acceleration: f32,
    pub player_y_acceleration: f32,
    pub player_max_x_speed: f32,
    pub player_min_fall_speed: f32,
    pub player_max_rise_speed: f32,
    pub player_charging_power_duration: f32,
    pub player_dash_duration: f32,
    pub player_dash_immunity_duration: f32,
    pub player_dash_speed: f32,

    pub cam_min_y: f32,
    pub cam_max_y: f32,
    pub cam_min_x: f32,
    pub cam_max_x: f32,
    pub cam_smoothing: f32,
}

impl GameConfig {
    pub fn load(path: &str) -> GameConfig {
        let content = fs::read_to_string(path).expect("Failed to read config file");
        ron::de::from_str(&content).expect("Failed to parse config file")
    }
}
