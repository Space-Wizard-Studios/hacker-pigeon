use bevy::prelude::*;

const INPUT_UP: u8 = 1 << 0;
const INPUT_DOWN: u8 = 1 << 1;
const INPUT_LEFT: u8 = 1 << 2;
const INPUT_RIGHT: u8 = 1 << 3;
const INPUT_DASH: u8 = 1 << 4;

#[derive(Resource, Default, Debug, Deref)]
pub struct Input(u8);

#[derive(Resource, Default, Debug, Deref)]
pub struct MousePos(Vec2);

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

pub fn read_inputs(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
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
    if mouse.any_pressed([MouseButton::Left]) {
        input |= INPUT_DASH;
    }

    commands.insert_resource(Input(input));
}

impl Input {
    pub fn dir(&self) -> Vec2 {
        let mut dir = Vec2::ZERO;

        if self.0 & INPUT_UP != 0 {
            dir.y += 1.;
        }
        if self.0 & INPUT_DOWN != 0 {
            dir.y -= 1.;
        }
        if self.0 & INPUT_RIGHT != 0 {
            dir.x += 1.;
        }
        if self.0 & INPUT_LEFT != 0 {
            dir.x -= 1.;
        }

        dir.normalize_or_zero()
    }

    pub fn dash(&self) -> bool {
        self.0 & INPUT_DASH != 0
    }
}
