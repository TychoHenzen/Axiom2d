use bevy_ecs::prelude::{Query, ResMut, With};

use crate::card::interaction::physics_helpers::warn_on_physics_result;
use crate::card::reader::components::CardReader;
use engine_physics::prelude::PhysicsRes;

pub fn reader_rotation_lock_system(
    query: Query<bevy_ecs::prelude::Entity, With<CardReader>>,
    mut physics: ResMut<PhysicsRes>,
) {
    for entity in &query {
        warn_on_physics_result(
            "set_angular_velocity",
            entity,
            physics.set_angular_velocity(entity, 0.0),
        );
    }
}
