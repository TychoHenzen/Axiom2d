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
    fn body_linear_velocity(&self, entity: Entity) -> Option<Vec2>;
    fn set_linear_velocity(&mut self, entity: Entity, velocity: Vec2);
    fn set_angular_velocity(&mut self, entity: Entity, angular_velocity: f32);
    fn add_force_at_point(&mut self, entity: Entity, force: Vec2, world_point: Vec2);
    fn body_angular_velocity(&self, entity: Entity) -> Option<f32>;
    fn set_damping(&mut self, entity: Entity, linear: f32, angular: f32);
    fn set_collision_group(&mut self, entity: Entity, membership: u32, filter: u32);

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

    fn body_linear_velocity(&self, _entity: Entity) -> Option<Vec2> {
        None
    }

    fn set_linear_velocity(&mut self, _entity: Entity, _velocity: Vec2) {}

    fn set_angular_velocity(&mut self, _entity: Entity, _angular_velocity: f32) {}

    fn add_force_at_point(&mut self, _entity: Entity, _force: Vec2, _world_point: Vec2) {}

    fn body_angular_velocity(&self, _entity: Entity) -> Option<f32> {
        None
    }

    fn set_damping(&mut self, _entity: Entity, _linear: f32, _angular: f32) {}

    fn set_collision_group(&mut self, _entity: Entity, _membership: u32, _filter: u32) {}
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_helpers::spawn_entity;

    #[test]
    fn when_step_called_then_step_count_increments() {
        // Arrange
        let mut backend = NullPhysicsBackend::default();

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
        let mut backend = NullPhysicsBackend::default();
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
        let backend = NullPhysicsBackend::default();
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
        let mut backend = NullPhysicsBackend::default();
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
        let mut backend = NullPhysicsBackend::default();
        let entity = spawn_entity();

        // Act
        backend.remove_body(entity);
    }

    #[test]
    fn when_null_backend_drain_collision_events_then_returns_empty() {
        // Arrange
        let mut backend = NullPhysicsBackend::default();

        // Act
        let events = backend.drain_collision_events();

        // Assert
        assert!(events.is_empty());
    }

    #[test]
    fn when_body_linear_velocity_on_null_backend_then_returns_none() {
        // Arrange
        let backend = NullPhysicsBackend::default();
        let entity = spawn_entity();

        // Act
        let result = backend.body_linear_velocity(entity);

        // Assert
        assert!(result.is_none());
    }

    #[test]
    fn when_body_angular_velocity_on_null_backend_then_returns_none() {
        // Arrange
        let backend = NullPhysicsBackend::default();
        let entity = spawn_entity();

        // Act
        let result = backend.body_angular_velocity(entity);

        // Assert
        assert!(result.is_none());
    }

    #[test]
    fn when_add_collider_without_body_then_returns_false() {
        // Arrange
        let mut backend = NullPhysicsBackend::default();
        let entity = spawn_entity();

        // Act
        let result = backend.add_collider(entity, &Collider::Circle(1.0));

        // Assert
        assert!(!result);
    }

    #[test]
    fn when_add_collider_then_returns_true() {
        // Arrange
        let mut backend = NullPhysicsBackend::default();
        let entity = spawn_entity();
        backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);

        // Act
        let result = backend.add_collider(entity, &Collider::Circle(1.0));

        // Assert
        assert!(result);
    }

    #[test]
    fn when_add_force_at_point_on_registered_body_then_no_panic() {
        // Arrange
        let mut backend = NullPhysicsBackend::default();
        let entity = spawn_entity();
        backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);

        // Act
        backend.add_force_at_point(entity, Vec2::new(10.0, 0.0), Vec2::ZERO);
    }

    #[test]
    fn when_add_force_at_point_on_unknown_entity_then_no_panic() {
        // Arrange
        let mut backend = NullPhysicsBackend::default();
        let entity = spawn_entity();

        // Act
        backend.add_force_at_point(entity, Vec2::new(0.0, -9.8), Vec2::ZERO);
    }

    #[test]
    fn when_set_damping_on_registered_body_then_no_panic() {
        // Arrange
        let mut backend = NullPhysicsBackend::default();
        let entity = spawn_entity();
        backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);

        // Act
        backend.set_damping(entity, 0.5, 0.1);
    }

    #[test]
    fn when_set_damping_on_unknown_entity_then_no_panic() {
        // Arrange
        let mut backend = NullPhysicsBackend::default();
        let entity = spawn_entity();

        // Act
        backend.set_damping(entity, 1.0, 0.0);
    }

    #[test]
    fn when_set_collision_group_on_unknown_entity_then_no_panic() {
        // Arrange
        let mut backend = NullPhysicsBackend::default();
        let entity = spawn_entity();

        // Act
        backend.set_collision_group(entity, 1, 2);
    }

    #[test]
    fn when_set_collision_group_on_registered_body_then_no_panic() {
        // Arrange
        let mut backend = NullPhysicsBackend::default();
        let entity = spawn_entity();
        backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);
        backend.add_collider(entity, &Collider::Circle(1.0));

        // Act
        backend.set_collision_group(entity, 1, 2);
    }

    use crate::test_helpers::SpyPhysicsBackend;

    #[test]
    fn when_local_origin_then_returns_body_position() {
        // Arrange
        let entity = spawn_entity();
        let backend = SpyPhysicsBackend::new().with_body(entity, Vec2::new(5.0, 3.0), 0.0);

        // Act
        let result = backend.body_point_to_world(entity, Vec2::ZERO);

        // Assert
        let world_pos = result.unwrap();
        assert!((world_pos.x - 5.0).abs() < 1e-6);
        assert!((world_pos.y - 3.0).abs() < 1e-6);
    }

    #[test]
    fn when_unrotated_then_local_offset_translates_directly() {
        // Arrange
        let entity = spawn_entity();
        let backend = SpyPhysicsBackend::new().with_body(entity, Vec2::new(5.0, 3.0), 0.0);

        // Act
        let result = backend.body_point_to_world(entity, Vec2::new(1.0, 0.0));

        // Assert
        let world_pos = result.unwrap();
        assert!((world_pos.x - 6.0).abs() < 1e-6);
        assert!((world_pos.y - 3.0).abs() < 1e-6);
    }

    #[test]
    fn when_rotated_90_degrees_then_local_offset_rotated() {
        // Arrange
        let entity = spawn_entity();
        let quarter_turn = std::f32::consts::FRAC_PI_2;
        let backend = SpyPhysicsBackend::new().with_body(entity, Vec2::ZERO, quarter_turn);

        // Act
        let result = backend.body_point_to_world(entity, Vec2::new(1.0, 0.0));

        // Assert
        let world_pos = result.unwrap();
        assert!(world_pos.x.abs() < 1e-6);
        assert!((world_pos.y - 1.0).abs() < 1e-6);
    }

    #[test]
    fn when_rotated_and_translated_then_both_applied() {
        // Arrange
        let entity = spawn_entity();
        let quarter_turn = std::f32::consts::FRAC_PI_4; // 45 degrees
        let backend =
            SpyPhysicsBackend::new().with_body(entity, Vec2::new(10.0, 5.0), quarter_turn);

        // Act
        let result = backend.body_point_to_world(entity, Vec2::new(2.0, 0.0));

        // Assert
        let world_pos = result.unwrap();
        let cos = quarter_turn.cos();
        let sin = quarter_turn.sin();
        let expected_x = 10.0 + 2.0 * cos;
        let expected_y = 5.0 + 2.0 * sin;
        assert!((world_pos.x - expected_x).abs() < 1e-4);
        assert!((world_pos.y - expected_y).abs() < 1e-4);
    }

    #[test]
    fn when_unknown_entity_then_body_point_to_world_returns_none() {
        // Arrange
        let entity = spawn_entity();
        let backend = SpyPhysicsBackend::new();

        // Act
        let result = backend.body_point_to_world(entity, Vec2::new(1.0, 0.0));

        // Assert
        assert!(result.is_none());
    }
}
