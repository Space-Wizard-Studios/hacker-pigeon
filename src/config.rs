use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::Deserialize;

#[derive(Asset, TypePath, Debug, Deserialize, Clone)]
#[serde(default)]
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

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            gravity: -42.0,
            charging_gravity_multiplier: 0.2,
            movement_smoothing: 8.0,
            air_friction: 2.0,
            ground_friction: 6.0,
            floor_y: -160.0,
            x_limit: 1000.0,
            ceiling_y: 300.0,
            spring_force: 6.0,
            max_pull: -280.0,
            player_x_acceleration: 3200.0,
            player_y_acceleration: 2200.0,
            player_max_x_speed: 240.0,
            player_min_fall_speed: -280.0,
            player_max_rise_speed: 280.0,
            player_charging_power_duration: 0.75,
            player_dash_duration: 0.12,
            player_dash_immunity_duration: 1.0,
            player_dash_speed: 3000.0,
            cam_min_y: -30.0,
            cam_max_y: 60.0,
            cam_min_x: -900.0,
            cam_max_x: 900.0,
            cam_smoothing: 8.0,
        }
    }
}

#[derive(Resource, Debug)]
pub struct Config {
    pub game: GameConfig,
}
