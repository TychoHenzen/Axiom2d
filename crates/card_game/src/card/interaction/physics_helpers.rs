use bevy_ecs::prelude::Entity;
use engine_physics::prelude::{Collider, PhysicsError, PhysicsRes, RigidBody};
use glam::Vec2;

use crate::card::interaction::damping::{BASE_ANGULAR_DRAG, BASE_LINEAR_DRAG};

/// Report a physics operation that returned `Err` so the failure is visible in logs.
pub(crate) fn warn_on_physics_result(
    operation: &'static str,
    entity: Entity,
    result: Result<(), PhysicsError>,
) {
    if let Err(error) = result {
        tracing::warn!(?entity, operation, error = %error, "physics operation failed");
    }
}

/// Report a physics operation that returned `false` so the failure is visible in logs.
pub(crate) fn warn_on_physics_bool(operation: &'static str, entity: Entity, success: bool) {
    if !success {
        tracing::warn!(?entity, operation, "physics operation failed");
    }
}

/// Register a dynamic rigid body in the physics backend with base damping
/// and the given collision group. No-ops if the entity already has a body.
pub(crate) fn activate_physics_body(
    entity: Entity,
    position: Vec2,
    collider: &Collider,
    physics: &mut PhysicsRes,
    membership: u32,
    filter: u32,
) {
    if physics.body_position(entity).is_some() {
        return;
    }
    warn_on_physics_bool(
        "add_body",
        entity,
        physics.add_body(entity, &RigidBody::Dynamic, position),
    );
    warn_on_physics_bool(
        "add_collider",
        entity,
        physics.add_collider(entity, collider),
    );
    physics
        .set_damping(entity, BASE_LINEAR_DRAG, BASE_ANGULAR_DRAG)
        .expect("activate_physics_body: entity should have been just added");
    physics
        .set_collision_group(entity, membership, filter)
        .expect("activate_physics_body: entity should have been just added");
}
