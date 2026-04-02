use engine_core::prelude::Seconds;
use engine_physics::collider::Collider;
use engine_physics::physics_backend::{NullPhysicsBackend, PhysicsBackend};
use engine_physics::rigid_body::RigidBody;
use engine_physics::test_helpers::{SpyPhysicsBackend, spawn_entity};
use glam::Vec2;

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
    backend.remove_body(entity).unwrap();
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
    backend.remove_body(entity).unwrap();
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
    backend
        .add_force_at_point(entity, Vec2::new(10.0, 0.0), Vec2::ZERO)
        .unwrap();
}

#[test]
fn when_add_force_at_point_on_unknown_entity_then_no_panic() {
    // Arrange
    let mut backend = NullPhysicsBackend::default();
    let entity = spawn_entity();

    // Act
    backend
        .add_force_at_point(entity, Vec2::new(0.0, -9.8), Vec2::ZERO)
        .unwrap();
}

#[test]
fn when_set_damping_on_registered_body_then_no_panic() {
    // Arrange
    let mut backend = NullPhysicsBackend::default();
    let entity = spawn_entity();
    backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);

    // Act
    backend.set_damping(entity, 0.5, 0.1).unwrap();
}

#[test]
fn when_set_damping_on_unknown_entity_then_no_panic() {
    // Arrange
    let mut backend = NullPhysicsBackend::default();
    let entity = spawn_entity();

    // Act
    backend.set_damping(entity, 1.0, 0.0).unwrap();
}

#[test]
fn when_set_collision_group_on_unknown_entity_then_no_panic() {
    // Arrange
    let mut backend = NullPhysicsBackend::default();
    let entity = spawn_entity();

    // Act
    backend.set_collision_group(entity, 1, 2).unwrap();
}

#[test]
fn when_set_collision_group_on_registered_body_then_no_panic() {
    // Arrange
    let mut backend = NullPhysicsBackend::default();
    let entity = spawn_entity();
    backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);
    backend.add_collider(entity, &Collider::Circle(1.0));

    // Act
    backend.set_collision_group(entity, 1, 2).unwrap();
}

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
    let backend = SpyPhysicsBackend::new().with_body(entity, Vec2::new(10.0, 5.0), quarter_turn);

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
