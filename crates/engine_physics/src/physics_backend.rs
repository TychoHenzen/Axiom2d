use std::collections::HashSet;

use bevy_ecs::prelude::Entity;
use engine_core::prelude::Seconds;
use glam::Vec2;

use crate::collider::Collider;
use crate::collision_event::CollisionEvent;
use crate::rigid_body::RigidBody;

pub trait PhysicsBackend: Send + Sync {
    fn step(&mut self, dt: Seconds);
    fn add_body(&mut self, entity: Entity, body_type: &RigidBody, position: Vec2) -> bool;
    fn add_collider(&mut self, entity: Entity, collider: &Collider) -> bool;
    fn remove_body(&mut self, entity: Entity);
    fn body_position(&self, entity: Entity) -> Option<Vec2>;
    fn body_rotation(&self, entity: Entity) -> Option<f32>;
    fn drain_collision_events(&mut self) -> Vec<CollisionEvent>;
}

#[derive(Default)]
pub struct NullPhysicsBackend {
    step_count: u32,
    registered: HashSet<Entity>,
}

impl NullPhysicsBackend {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

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

    fn remove_body(&mut self, entity: Entity) {
        self.registered.remove(&entity);
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
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_helpers::spawn_entity;

    #[test]
    fn when_step_called_then_step_count_increments() {
        // Arrange
        let mut backend = NullPhysicsBackend::new();

        // Act
        backend.step(Seconds(0.016));
        backend.step(Seconds(0.016));
        backend.step(Seconds(0.016));

        // Assert
        assert_eq!(backend.step_count(), 3);
    }

    #[test]
    fn when_add_body_then_returns_true_and_duplicate_returns_false() {
        // Arrange
        let mut backend = NullPhysicsBackend::new();
        let entity = spawn_entity();

        // Act
        let first = backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);
        let second = backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);

        // Assert
        assert!(first);
        assert!(!second);
    }

    #[test]
    fn when_body_position_queried_for_unregistered_then_returns_none() {
        // Arrange
        let backend = NullPhysicsBackend::new();
        let entity = spawn_entity();

        // Act
        let pos = backend.body_position(entity);
        let rot = backend.body_rotation(entity);

        // Assert
        assert!(pos.is_none());
        assert!(rot.is_none());
    }

    #[test]
    fn when_remove_body_then_entity_is_deregistered() {
        // Arrange
        let mut backend = NullPhysicsBackend::new();
        let entity = spawn_entity();
        backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);

        // Act
        backend.remove_body(entity);
        let re_add = backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);

        // Assert
        assert!(re_add);
    }

    #[test]
    fn when_remove_body_for_unknown_entity_then_no_panic() {
        // Arrange
        let mut backend = NullPhysicsBackend::new();
        let entity = spawn_entity();

        // Act
        backend.remove_body(entity);
    }

    #[test]
    fn when_null_backend_drain_collision_events_then_returns_empty() {
        // Arrange
        let mut backend = NullPhysicsBackend::new();

        // Act
        let events = backend.drain_collision_events();

        // Assert
        assert!(events.is_empty());
    }

    #[test]
    fn when_add_collider_without_body_then_returns_false() {
        // Arrange
        let mut backend = NullPhysicsBackend::new();
        let entity = spawn_entity();

        // Act
        let result = backend.add_collider(entity, &Collider::Circle(1.0));

        // Assert
        assert!(!result);
    }

    #[test]
    fn when_add_collider_then_returns_true() {
        // Arrange
        let mut backend = NullPhysicsBackend::new();
        let entity = spawn_entity();
        backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);

        // Act
        let result = backend.add_collider(entity, &Collider::Circle(1.0));

        // Assert
        assert!(result);
    }
}
