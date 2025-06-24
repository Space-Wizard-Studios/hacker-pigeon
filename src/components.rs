use bevy::prelude::*;

// --- Common Components ---
#[derive(Component, Default, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

// --- Player Components ---
#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Grounded;

#[derive(Component)]
pub struct Dashing {
    pub timer: Timer,
}

#[derive(Component)]
pub struct AbilityCooldown(pub Timer);

#[derive(Component)]
pub struct AimArrow;

#[derive(Component)]
pub struct Charging {
    pub direction: Vec2,
}

#[derive(Component)]
pub struct HopDebounce {
    pub timer: Timer,
}

// --- Enemy Components ---
#[derive(Component)]
pub struct Drone {
    pub hp: u8,
    pub weak_point: WeakPointSide,
}

#[derive(Component)]
pub struct DroneHover {
    pub center: Vec2,
    pub phase: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WeakPointSide {
    Left,
    Right,
    Top,
    Bottom,
}

// --- UI Components ---
#[derive(Component)]
pub struct HpText;

#[derive(Component)]
pub struct DebugText;

// --- Resources ---
#[derive(Resource, Default)]
pub struct DroneCollisionDebug {
    pub last_event: String,
    pub last_player_hp: i32,
    pub last_drone_hp: Option<u8>,
}

#[derive(Component)]
pub struct CollisionImmunity {
    pub timer: Timer,
    pub blink: bool,
    pub original_color: Option<Color>,
}
