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

// --- Enemy Components ---
#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct WeakPoint {
    pub direction: Vec2,
}

// --- UI Components ---
#[derive(Component)]
pub struct HpText;

#[derive(Component)]
pub struct DebugText;
