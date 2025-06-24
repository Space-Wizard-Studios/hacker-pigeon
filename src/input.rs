use bevy::prelude::*;

use crate::{Input, MousePos};

const INPUT_UP: u8 = 1 << 0;
const INPUT_DOWN: u8 = 1 << 1;
const INPUT_LEFT: u8 = 1 << 2;
const INPUT_RIGHT: u8 = 1 << 3;
const INPUT_FIRE: u8 = 1 << 4;

pub fn read_mouse_position(
    mut commands: Commands,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
) {
    let window = windows.single().unwrap();
    let (camera, camera_transform) = camera_q.single().unwrap();

    if let Some(Ok(world_position)) = window
        .cursor_position()
        .map(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
    {
        commands.insert_resource(MousePos(world_position));
    }
}

pub fn read_inputs(mut commands: Commands, keys: Res<ButtonInput<KeyCode>>) {
    let mut input = 0u8;

    if keys.any_pressed([KeyCode::KeyW]) {
        input |= INPUT_UP;
    }
    if keys.any_pressed([KeyCode::KeyS]) {
        input |= INPUT_DOWN;
    }
    if keys.any_pressed([KeyCode::KeyA]) {
        input |= INPUT_LEFT
    }
    if keys.any_pressed([KeyCode::KeyD]) {
        input |= INPUT_RIGHT;
    }

    commands.insert_resource(Input(input));
}

pub fn direction(input: u8) -> Vec2 {
    let mut direction = Vec2::ZERO;
    if input & INPUT_UP != 0 {
        direction.y += 1.;
    }
    if input & INPUT_DOWN != 0 {
        direction.y -= 1.;
    }
    if input & INPUT_RIGHT != 0 {
        direction.x += 1.;
    }
    if input & INPUT_LEFT != 0 {
        direction.x -= 1.;
    }

    direction.normalize_or_zero()
}

pub fn fire(input: u8) -> bool {
    input & INPUT_FIRE != 0
}
