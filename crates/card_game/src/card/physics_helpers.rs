use bevy_ecs::prelude::Entity;
use engine_physics::prelude::{Collider, PhysicsRes, RigidBody};
use glam::Vec2;

use crate::card::damping::{BASE_ANGULAR_DRAG, BASE_LINEAR_DRAG};

/// Register a dynamic rigid body in the physics backend with base damping
/// and the given collision group.
pub(crate) fn activate_physics_body(
    entity: Entity,
    position: Vec2,
    collider: &Collider,
    physics: &mut PhysicsRes,
    membership: u32,
    filter: u32,
) {
    physics.add_body(entity, &RigidBody::Dynamic, position);
    physics.add_collider(entity, collider);
    physics.set_damping(entity, BASE_LINEAR_DRAG, BASE_ANGULAR_DRAG);
    physics.set_collision_group(entity, membership, filter);
}
