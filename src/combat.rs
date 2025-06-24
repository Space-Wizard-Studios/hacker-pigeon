use bevy::prelude::*;
use crate::components::*;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        // Remove sistemas antigos de colisão para não conflitar com drones
        // app.add_systems(Update, (collision_system, death_system).chain());
        app.add_systems(Update, death_system);
    }
}

fn death_system(mut commands: Commands, query: Query<(Entity, &Health)>) {
    for (entity, health) in query.iter() {
        if health.current <= 0 {
            commands.entity(entity).despawn();
        }
    }
}
