// EVOLVE-BLOCK-START
use bevy_ecs::prelude::Entity;
use engine_core::prelude::EventBus;
use engine_physics::prelude::{Collider, PhysicsCommand, RigidBody};
use glam::Vec2;

use crate::card::interaction::damping::{BASE_ANGULAR_DRAG, BASE_LINEAR_DRAG};

/// Queue commands to register a dynamic rigid body with base damping and the
/// given collision group.
pub(crate) fn activate_physics_body(
    entity: Entity,
    position: Vec2,
    collider: &Collider,
    commands: &mut EventBus<PhysicsCommand>,
    membership: u32,
    filter: u32,
) {
    commands.push(PhysicsCommand::AddBody {
        entity,
        body_type: RigidBody::Dynamic,
        position,
    });
    commands.push(PhysicsCommand::AddCollider {
        entity,
        collider: collider.clone(),
    });
    commands.push(PhysicsCommand::SetDamping {
        entity,
        linear: BASE_LINEAR_DRAG,
        angular: BASE_ANGULAR_DRAG,
    });
    commands.push(PhysicsCommand::SetCollisionGroup {
        entity,
        membership,
        filter,
    });
}
// EVOLVE-BLOCK-END
