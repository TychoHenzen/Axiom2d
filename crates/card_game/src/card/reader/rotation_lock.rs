use bevy_ecs::prelude::{Query, ResMut, With};
use engine_core::prelude::EventBus;
use engine_physics::prelude::PhysicsCommand;

use crate::card::reader::components::CardReader;

pub fn reader_rotation_lock_system(
    query: Query<bevy_ecs::prelude::Entity, With<CardReader>>,
    mut physics_commands: ResMut<EventBus<PhysicsCommand>>,
) {
    for entity in &query {
        physics_commands.push(PhysicsCommand::SetAngularVelocity {
            entity,
            angular_velocity: 0.0,
        });
    }
}
