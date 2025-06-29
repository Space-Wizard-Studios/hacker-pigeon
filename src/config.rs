use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::Deserialize;

#[derive(Asset, TypePath, Debug, Deserialize, Clone)]
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

#[derive(Resource, Debug)]
pub struct Config {
    pub game: GameConfig,
}
