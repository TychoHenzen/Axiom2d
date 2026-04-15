use bevy_ecs::prelude::Entity;
use engine_core::prelude::Event;
use glam::Vec2;

use crate::collider::Collider;
use crate::rigid_body::RigidBody;

#[derive(Debug)]
pub enum PhysicsCommand {
    AddBody {
        entity: Entity,
        body_type: RigidBody,
        position: Vec2,
    },
    AddCollider {
        entity: Entity,
        collider: Collider,
    },
    RemoveBody {
        entity: Entity,
    },
    SetLinearVelocity {
        entity: Entity,
        velocity: Vec2,
    },
    SetAngularVelocity {
        entity: Entity,
        angular_velocity: f32,
    },
    SetDamping {
        entity: Entity,
        linear: f32,
        angular: f32,
    },
    SetCollisionGroup {
        entity: Entity,
        membership: u32,
        filter: u32,
    },
    SetBodyPosition {
        entity: Entity,
        position: Vec2,
    },
    AddForceAtPoint {
        entity: Entity,
        force: Vec2,
        world_point: Vec2,
    },
}

impl Event for PhysicsCommand {}
