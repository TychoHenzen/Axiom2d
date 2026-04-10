// EVOLVE-BLOCK-START
use std::collections::HashSet;

use bevy_ecs::prelude::Entity;
use engine_core::prelude::Seconds;
use glam::Vec2;

use crate::collider::Collider;
use crate::collision_event::CollisionEvent;
use crate::rigid_body::RigidBody;

#[derive(Debug, thiserror::Error)]
pub enum PhysicsError {
    #[error("entity {0:?} not found in physics world")]
    EntityNotFound(Entity),
    #[error("physics operation failed: {0}")]
    OperationFailed(String),
}

pub trait PhysicsBackend: Send + Sync {
    fn step(&mut self, dt: Seconds);
    fn add_body(&mut self, entity: Entity, body_type: &RigidBody, position: Vec2) -> bool;
    fn add_collider(&mut self, entity: Entity, collider: &Collider) -> bool;
    fn remove_body(&mut self, entity: Entity) -> Result<(), PhysicsError>;
    fn body_position(&self, entity: Entity) -> Option<Vec2>;
    fn body_rotation(&self, entity: Entity) -> Option<f32>;
    fn drain_collision_events(&mut self) -> Vec<CollisionEvent>;
    fn body_linear_velocity(&self, entity: Entity) -> Option<Vec2>;
    fn set_linear_velocity(&mut self, entity: Entity, velocity: Vec2) -> Result<(), PhysicsError>;
    fn set_angular_velocity(
        &mut self,
        entity: Entity,
        angular_velocity: f32,
    ) -> Result<(), PhysicsError>;
    fn add_force_at_point(
        &mut self,
        entity: Entity,
        force: Vec2,
        world_point: Vec2,
    ) -> Result<(), PhysicsError>;
    fn body_angular_velocity(&self, entity: Entity) -> Option<f32>;
    fn set_damping(
        &mut self,
        entity: Entity,
        linear: f32,
        angular: f32,
    ) -> Result<(), PhysicsError>;
    fn set_collision_group(
        &mut self,
        entity: Entity,
        membership: u32,
        filter: u32,
    ) -> Result<(), PhysicsError>;

    fn set_body_position(&mut self, entity: Entity, position: Vec2) -> Result<(), PhysicsError>;

    fn body_point_to_world(&self, entity: Entity, local_point: Vec2) -> Option<Vec2> {
        let pos = self.body_position(entity)?;
        let rot = self.body_rotation(entity)?;
        let (sin, cos) = rot.sin_cos();
        Some(
            pos + Vec2::new(
                local_point.x * cos - local_point.y * sin,
                local_point.x * sin + local_point.y * cos,
            ),
        )
    }
}

#[derive(Default)]
pub struct NullPhysicsBackend {
    step_count: u32,
    registered: HashSet<Entity>,
}

impl NullPhysicsBackend {
    #[must_use]
    pub fn step_count(&self) -> u32 {
        self.step_count
    }
}

impl PhysicsBackend for NullPhysicsBackend {
    fn step(&mut self, _dt: Seconds) {
        self.step_count += 1;
    }

    fn add_body(&mut self, entity: Entity, _body_type: &RigidBody, _position: Vec2) -> bool {
        self.registered.insert(entity)
    }

    fn add_collider(&mut self, entity: Entity, _collider: &Collider) -> bool {
        self.registered.contains(&entity)
    }

    fn remove_body(&mut self, entity: Entity) -> Result<(), PhysicsError> {
        self.registered.remove(&entity);
        Ok(())
    }

    fn body_position(&self, _entity: Entity) -> Option<Vec2> {
        None
    }

    fn body_rotation(&self, _entity: Entity) -> Option<f32> {
        None
    }

    fn drain_collision_events(&mut self) -> Vec<CollisionEvent> {
        Vec::new()
    }

    fn body_linear_velocity(&self, _entity: Entity) -> Option<Vec2> {
        None
    }

    fn set_linear_velocity(
        &mut self,
        _entity: Entity,
        _velocity: Vec2,
    ) -> Result<(), PhysicsError> {
        Ok(())
    }

    fn set_angular_velocity(
        &mut self,
        _entity: Entity,
        _angular_velocity: f32,
    ) -> Result<(), PhysicsError> {
        Ok(())
    }

    fn add_force_at_point(
        &mut self,
        _entity: Entity,
        _force: Vec2,
        _world_point: Vec2,
    ) -> Result<(), PhysicsError> {
        Ok(())
    }

    fn body_angular_velocity(&self, _entity: Entity) -> Option<f32> {
        None
    }

    fn set_damping(
        &mut self,
        _entity: Entity,
        _linear: f32,
        _angular: f32,
    ) -> Result<(), PhysicsError> {
        Ok(())
    }

    fn set_collision_group(
        &mut self,
        _entity: Entity,
        _membership: u32,
        _filter: u32,
    ) -> Result<(), PhysicsError> {
        Ok(())
    }

    fn set_body_position(&mut self, _entity: Entity, _position: Vec2) -> Result<(), PhysicsError> {
        Ok(())
    }
}
// EVOLVE-BLOCK-END
