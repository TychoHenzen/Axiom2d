#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::Entity;
use engine_core::prelude::Seconds;
use engine_physics::prelude::*;
use glam::Vec2;

fn spawn_entity() -> Entity {
    bevy_ecs::prelude::World::new().spawn(()).id()
}

fn spawn_entities(count: usize) -> Vec<Entity> {
    let mut world = bevy_ecs::prelude::World::new();
    (0..count).map(|_| world.spawn(()).id()).collect()
}

/// @doc: Empty step must not panic — physics engine must handle zero-entity case gracefully
#[test]
fn when_rapier_step_on_empty_world_then_no_panic() {
    // Arrange
    let mut backend = RapierBackend::new(Vec2::new(0.0, -9.81));

    // Act
    backend.step(Seconds(0.016));
}

/// @doc: Body type mapping: ECS Dynamic → rapier Dynamic (free motion under forces)
#[test]
fn when_dynamic_body_added_then_position_is_queryable() {
    // Arrange
    let mut backend = RapierBackend::new(Vec2::ZERO);
    let entity = spawn_entity();

    // Act
    let added = backend.add_body(entity, &RigidBody::Dynamic, Vec2::new(3.0, 7.0));
    let pos = backend.body_position(entity);

    // Assert
    assert!(added);
    let pos = pos.unwrap();
    assert!((pos.x - 3.0).abs() < 1e-4);
    assert!((pos.y - 7.0).abs() < 1e-4);
}

/// @doc: Duplicate `add_body` must return false — idempotent guard prevents double-registration in rapier world
#[test]
fn when_same_entity_added_twice_then_second_returns_false() {
    // Arrange
    let mut backend = RapierBackend::new(Vec2::ZERO);
    let entity = spawn_entity();
    backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);

    // Act
    let second = backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);

    // Assert
    assert!(!second);
}

/// @doc: Body type mapping: ECS Static → rapier Fixed (immovable), ECS Kinematic → rapier `KinematicPositionBased` (script-driven)
#[test]
fn when_body_type_mapping_then_static_is_fixed_and_kinematic_is_position_based() {
    // Arrange
    let mut backend = RapierBackend::new(Vec2::ZERO);
    let entities = spawn_entities(2);
    let static_entity = entities[0];
    let kinematic_entity = entities[1];

    // Act
    backend.add_body(static_entity, &RigidBody::Static, Vec2::ZERO);
    backend.add_body(kinematic_entity, &RigidBody::Kinematic, Vec2::ZERO);

    // Assert
    assert_eq!(
        backend.rapier_body_type(static_entity).unwrap(),
        rapier2d::prelude::RigidBodyType::Fixed
    );
    assert_eq!(
        backend.rapier_body_type(kinematic_entity).unwrap(),
        rapier2d::prelude::RigidBodyType::KinematicPositionBased
    );
}

/// @doc: All collider types must be addable — missing shape support breaks card physics shapes
#[test]
fn when_collider_variants_added_then_all_return_true() {
    // Arrange
    let mut backend = RapierBackend::new(Vec2::ZERO);
    let entities = spawn_entities(3);
    let (e1, e2, e3) = (entities[0], entities[1], entities[2]);
    backend.add_body(e1, &RigidBody::Dynamic, Vec2::ZERO);
    backend.add_body(e2, &RigidBody::Dynamic, Vec2::ZERO);
    backend.add_body(e3, &RigidBody::Dynamic, Vec2::ZERO);

    // Act
    let circle = backend.add_collider(e1, &Collider::Circle(2.0));
    let aabb = backend.add_collider(e2, &Collider::Aabb(Vec2::new(1.0, 0.5)));
    let polygon = backend.add_collider(
        e3,
        &Collider::ConvexPolygon(vec![Vec2::ZERO, Vec2::new(1.0, 0.0), Vec2::new(0.5, 1.0)]),
    );

    // Assert
    assert!(circle);
    assert!(aabb);
    assert!(polygon);
}

/// @doc: `add_collider` on unregistered entity must return false — guard prevents orphan colliders without bodies
#[test]
fn when_add_collider_for_unknown_entity_then_returns_false() {
    // Arrange
    let mut backend = RapierBackend::new(Vec2::ZERO);
    let entity = spawn_entity();

    // Act
    let result = backend.add_collider(entity, &Collider::Circle(1.0));

    // Assert
    assert!(!result);
}

/// @doc: Gravity must accelerate bodies downward — wrong gravity direction breaks game world feel
#[test]
fn when_dynamic_body_steps_under_gravity_then_y_changes() {
    // Arrange
    let mut backend = RapierBackend::new(Vec2::new(0.0, -9.81));
    let entity = spawn_entity();
    backend.add_body(entity, &RigidBody::Dynamic, Vec2::new(0.0, 10.0));
    backend.add_collider(entity, &Collider::Circle(0.5));

    // Act
    backend.step(Seconds(0.1));

    // Assert
    let pos = backend.body_position(entity).unwrap();
    assert!(pos.y < 10.0, "expected y < 10.0, got {}", pos.y);
}

/// @doc: New body must report zero rotation — non-zero initial rotation breaks card draw behavior
#[test]
fn when_dynamic_body_added_then_rotation_returns_some() {
    // Arrange
    let mut backend = RapierBackend::new(Vec2::ZERO);
    let entity = spawn_entity();
    backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);

    // Act
    let rotation = backend.body_rotation(entity);

    // Assert
    let rotation = rotation.expect("should return Some for living body");
    assert!(rotation.abs() < 1e-4, "initial rotation should be ~0");
}

/// @doc: Entity removal must clean up both rapier `RigidBody` and the entity↔handle map
#[test]
fn when_remove_body_on_rapier_then_position_returns_none() {
    // Arrange
    let mut backend = RapierBackend::new(Vec2::ZERO);
    let entity = spawn_entity();
    backend.add_body(entity, &RigidBody::Dynamic, Vec2::new(1.0, 2.0));

    // Act
    backend.remove_body(entity).unwrap();

    // Assert
    assert!(backend.body_position(entity).is_none());
    assert!(backend.body_rotation(entity).is_none());
}

/// @doc: Empty physics step must produce no collision events — prevents ghost events on startup
#[test]
fn when_no_colliders_step_and_drain_then_no_events() {
    // Arrange
    let mut backend = RapierBackend::new(Vec2::ZERO);

    // Act
    backend.step(Seconds(0.016));
    let events = backend.drain_collision_events();

    // Assert
    assert!(events.is_empty());
}

/// @doc: Collision events flow: rapier `ChannelEventCollector` → drain → `EventBus<CollisionEvent>` with entity resolution
#[test]
fn when_two_overlapping_circles_step_then_started_event_with_correct_entities() {
    // Arrange
    let mut backend = RapierBackend::new(Vec2::ZERO);
    let entities = spawn_entities(2);
    backend.add_body(entities[0], &RigidBody::Dynamic, Vec2::ZERO);
    backend.add_collider(entities[0], &Collider::Circle(1.0));
    backend.add_body(entities[1], &RigidBody::Dynamic, Vec2::ZERO);
    backend.add_collider(entities[1], &Collider::Circle(1.0));

    // Act
    backend.step(Seconds(0.016));
    let events = backend.drain_collision_events();

    // Assert
    assert_eq!(events.len(), 1, "expected 1 event, got {events:?}");
    assert_eq!(events[0].kind, CollisionKind::Started);
    let pair = (events[0].entity_a, events[0].entity_b);
    assert!(
        pair == (entities[0], entities[1]) || pair == (entities[1], entities[0]),
        "expected entities {:?}, got {:?}",
        (entities[0], entities[1]),
        pair
    );
}

/// @doc: `drain_collision_events` must consume queue — calling drain twice produces empty second result
#[test]
fn when_drain_called_twice_without_step_then_second_is_empty() {
    // Arrange
    let mut backend = RapierBackend::new(Vec2::ZERO);
    let entities = spawn_entities(2);
    backend.add_body(entities[0], &RigidBody::Dynamic, Vec2::ZERO);
    backend.add_collider(entities[0], &Collider::Circle(1.0));
    backend.add_body(entities[1], &RigidBody::Dynamic, Vec2::ZERO);
    backend.add_collider(entities[1], &Collider::Circle(1.0));
    backend.step(Seconds(0.016));
    let _ = backend.drain_collision_events();

    // Act
    let events = backend.drain_collision_events();

    // Assert
    assert!(events.is_empty());
}

/// @doc: Removed bodies in collision events must not crash drain — event buffer handles stale entity IDs gracefully
#[test]
fn when_body_removed_after_collision_then_drain_does_not_panic() {
    // Arrange
    let mut backend = RapierBackend::new(Vec2::ZERO);
    let entities = spawn_entities(2);
    backend.add_body(entities[0], &RigidBody::Dynamic, Vec2::ZERO);
    backend.add_collider(entities[0], &Collider::Circle(1.0));
    backend.add_body(entities[1], &RigidBody::Dynamic, Vec2::ZERO);
    backend.add_collider(entities[1], &Collider::Circle(1.0));
    backend.step(Seconds(0.016));
    backend.remove_body(entities[0]).unwrap();

    // Act
    backend.step(Seconds(0.016));
    let _ = backend.drain_collision_events();
}

/// @doc: `remove_body` on non-existent entity must return Err — prevents phantom cleanup in removal systems
#[test]
fn when_remove_body_for_unknown_entity_on_rapier_then_returns_err() {
    // Arrange
    let mut backend = RapierBackend::new(Vec2::ZERO);
    let entity = spawn_entity();

    // Act
    let result = backend.remove_body(entity);

    // Assert
    assert!(result.is_err());
}

/// @doc: `add_force_at_point` on unknown entity must return Err — prevents silent force loss in drag system
#[test]
fn when_add_force_at_point_for_unknown_entity_then_returns_err() {
    // Arrange
    let mut backend = RapierBackend::new(Vec2::ZERO);
    let entity = spawn_entity();

    // Act
    let result = backend.add_force_at_point(entity, Vec2::new(100.0, 0.0), Vec2::ZERO);

    // Assert
    assert!(result.is_err());
}

/// @doc: `set_damping` on unknown entity must return Err — prevents silent damping loss in physics setup
#[test]
fn when_set_damping_on_unknown_entity_then_returns_err() {
    // Arrange
    let mut backend = RapierBackend::new(Vec2::ZERO);
    let entity = spawn_entity();

    // Act
    let result = backend.set_damping(entity, 5.0, 3.0);

    // Assert
    assert!(result.is_err());
}

fn apply_impulse(backend: &mut RapierBackend, entity: Entity, force: Vec2, point: Vec2) {
    backend.add_force_at_point(entity, force, point).unwrap();
    backend.step(Seconds(0.016));
    backend.reset_body_forces(entity);
}

fn damped_body(gravity: Vec2, linear: f32, angular: f32) -> (RapierBackend, Entity) {
    let mut backend = RapierBackend::new(gravity);
    let entity = spawn_entity();
    backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);
    backend.add_collider(entity, &Collider::Circle(0.5));
    backend.set_damping(entity, linear, angular).unwrap();
    (backend, entity)
}

fn manual_rotate_point(backend: &RapierBackend, entity: Entity, offset: Vec2) -> Vec2 {
    let pos = backend.body_position(entity).unwrap();
    let rot = backend.body_rotation(entity).unwrap();
    let (sin, cos) = rot.sin_cos();
    pos + Vec2::new(
        offset.x * cos - offset.y * sin,
        offset.x * sin + offset.y * cos,
    )
}

fn assert_vec2_approx(actual: Vec2, expected: Vec2, epsilon: f32) {
    assert!(
        (actual.x - expected.x).abs() < epsilon,
        "x: got {}, expected {}",
        actual.x,
        expected.x
    );
    assert!(
        (actual.y - expected.y).abs() < epsilon,
        "y: got {}, expected {}",
        actual.y,
        expected.y
    );
}

/// @doc: Undamped bodies coast indefinitely after impulse — zero damping = persistent free motion
#[test]
fn when_zero_damping_body_given_impulse_then_keeps_moving_after_force_stops() {
    // Arrange
    let (mut backend, entity) = damped_body(Vec2::ZERO, 0.0, 0.0);
    apply_impulse(&mut backend, entity, Vec2::new(5000.0, 0.0), Vec2::ZERO);

    // Act
    for _ in 0..10 {
        backend.step(Seconds(0.016));
    }

    // Assert
    let pos = backend.body_position(entity).unwrap();
    assert!(
        pos.x > 0.5,
        "expected undamped body to coast to x > 0.5 after impulse, got {}",
        pos.x
    );
}

/// @doc: Linear damping reduces travel distance — higher damping = faster energy dissipation
#[test]
fn when_high_linear_damping_then_travels_less_distance_than_zero_damping() {
    // Arrange
    let (mut undamped, entity_u) = damped_body(Vec2::ZERO, 0.0, 0.0);
    let (mut damped, entity_d) = damped_body(Vec2::ZERO, 20.0, 0.0);

    // Act — one step with force, reset forces, then coast
    apply_impulse(&mut undamped, entity_u, Vec2::new(5000.0, 0.0), Vec2::ZERO);
    apply_impulse(&mut damped, entity_d, Vec2::new(5000.0, 0.0), Vec2::ZERO);

    for _ in 0..30 {
        undamped.step(Seconds(0.016));
        damped.step(Seconds(0.016));
    }

    // Assert
    let x_undamped = undamped.body_position(entity_u).unwrap().x;
    let x_damped = damped.body_position(entity_d).unwrap().x;
    assert!(
        x_damped < x_undamped * 0.5,
        "expected damped x ({x_damped}) < 50% of undamped x ({x_undamped})"
    );
}

/// @doc: Angular damping slows spin — higher damping = faster rotational energy loss
#[test]
fn when_high_angular_damping_then_rotates_less_than_undamped() {
    // Arrange
    let (mut undamped, entity_u) = damped_body(Vec2::ZERO, 0.0, 0.0);
    let (mut damped, entity_d) = damped_body(Vec2::ZERO, 0.0, 20.0);

    // Act — off-center force to induce spin
    let spin_force = Vec2::new(50.0, 0.0);
    let spin_point = Vec2::new(0.0, 1.0);
    apply_impulse(&mut undamped, entity_u, spin_force, spin_point);
    apply_impulse(&mut damped, entity_d, spin_force, spin_point);

    for _ in 0..10 {
        undamped.step(Seconds(0.016));
        damped.step(Seconds(0.016));
    }

    // Assert — compare angular velocity since cumulative rotation wraps
    let angvel_undamped = undamped.body_angular_velocity(entity_u).unwrap().abs();
    let angvel_damped = damped.body_angular_velocity(entity_d).unwrap().abs();
    assert!(
        angvel_damped < angvel_undamped * 0.5,
        "expected damped angvel ({angvel_damped}) < 50% of undamped angvel ({angvel_undamped})"
    );
}

/// @doc: Damping reset to zero restores undamped motion — used when cards transition between zones
#[test]
fn when_damping_reset_to_zero_then_body_moves_like_undamped() {
    // Arrange
    let mut reference = RapierBackend::new(Vec2::ZERO);
    let entity_r = spawn_entity();
    reference.add_body(entity_r, &RigidBody::Dynamic, Vec2::ZERO);
    reference.add_collider(entity_r, &Collider::Circle(0.5));
    reference.set_damping(entity_r, 0.0, 0.0).unwrap();

    let mut reset = RapierBackend::new(Vec2::ZERO);
    let entity_s = spawn_entity();
    reset.add_body(entity_s, &RigidBody::Dynamic, Vec2::ZERO);
    reset.add_collider(entity_s, &Collider::Circle(0.5));
    reset.set_damping(entity_s, 20.0, 20.0).unwrap();
    reset.set_damping(entity_s, 0.0, 0.0).unwrap();

    // Act — one step with force, reset forces, then coast
    apply_impulse(&mut reference, entity_r, Vec2::new(5000.0, 0.0), Vec2::ZERO);
    apply_impulse(&mut reset, entity_s, Vec2::new(5000.0, 0.0), Vec2::ZERO);

    for _ in 0..30 {
        reference.step(Seconds(0.016));
        reset.step(Seconds(0.016));
    }

    // Assert
    let x_reference = reference.body_position(entity_r).unwrap().x;
    let x_reset = reset.body_position(entity_s).unwrap().x;
    assert!(
        (x_reset - x_reference).abs() < 1e-3,
        "expected reset body x ({x_reset}) ≈ reference x ({x_reference})"
    );
}

/// @doc: Zero force must cause no motion — verifies force accumulation doesn't drift at zero
#[test]
fn when_zero_force_applied_at_center_then_body_does_not_move() {
    // Arrange
    let mut backend = RapierBackend::new(Vec2::ZERO);
    let entity = spawn_entity();
    backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);
    backend.add_collider(entity, &Collider::Circle(0.5));

    // Act
    backend
        .add_force_at_point(entity, Vec2::ZERO, Vec2::ZERO)
        .unwrap();
    backend.step(Seconds(0.016));

    // Assert
    let pos = backend.body_position(entity).unwrap();
    assert!(pos.x.abs() < 1e-4, "expected ~0 x, got {}", pos.x);
    assert!(pos.y.abs() < 1e-4, "expected ~0 y, got {}", pos.y);
}

/// @doc: Force at center produces translation only — no torque when force passes through COM
#[test]
fn when_sustained_x_force_at_center_then_only_x_grows_and_rotation_stays_zero() {
    // Arrange
    let mut backend = RapierBackend::new(Vec2::ZERO);
    let entity = spawn_entity();
    backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);
    backend.add_collider(entity, &Collider::Circle(0.5));

    // Act
    for _ in 0..5 {
        backend
            .add_force_at_point(entity, Vec2::new(1000.0, 0.0), Vec2::ZERO)
            .unwrap();
        backend.step(Seconds(0.016));
    }

    // Assert
    let pos = backend.body_position(entity).unwrap();
    let rot = backend.body_rotation(entity).unwrap();
    assert!(
        pos.x > 0.0,
        "expected x > 0 after repeated +x force, got {}",
        pos.x
    );
    assert!(pos.y.abs() < 1e-4, "expected y ≈ 0, got {}", pos.y);
    assert!(rot.abs() < 1e-5, "expected no rotation, got {rot}");
}

/// @doc: Force applied at offset point produces both translation and torque — off-center hits spin cards
#[test]
fn when_sustained_x_force_at_offset_y_point_then_body_translates_and_rotates() {
    // Arrange
    let mut backend = RapierBackend::new(Vec2::ZERO);
    let entity = spawn_entity();
    backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);
    backend.add_collider(entity, &Collider::Circle(0.5));

    // Act
    for _ in 0..5 {
        backend
            .add_force_at_point(entity, Vec2::new(1000.0, 0.0), Vec2::new(0.0, 1.0))
            .unwrap();
        backend.step(Seconds(0.016));
    }

    // Assert
    let pos = backend.body_position(entity).unwrap();
    let rot = backend.body_rotation(entity).unwrap();
    assert!(pos.x > 0.0, "expected x > 0, got {}", pos.x);
    assert!(rot.abs() > 1e-5, "expected rotation from torque, got {rot}");
}

/// @doc: `body_linear_velocity` must report exact set velocity — velocity query feeds drag calculation and feedback
#[test]
fn when_body_linear_velocity_queried_then_returns_current_velocity() {
    // Arrange
    let mut backend = RapierBackend::new(Vec2::ZERO);
    let entity = spawn_entity();
    backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);
    backend.add_collider(entity, &Collider::Circle(0.5));
    backend.set_damping(entity, 0.0, 0.0).unwrap();
    backend
        .set_linear_velocity(entity, Vec2::new(5.0, -3.0))
        .unwrap();

    // Act
    let vel = backend.body_linear_velocity(entity);

    // Assert
    let vel = vel.expect("should return Some for living body");
    assert!((vel.x - 5.0).abs() < 1e-4, "expected vx=5.0, got {}", vel.x);
    assert!(
        (vel.y - (-3.0)).abs() < 1e-4,
        "expected vy=-3.0, got {}",
        vel.y
    );
}

/// @doc: Velocity query on non-existent entity must return None — prevents crash in input query systems
#[test]
fn when_body_linear_velocity_on_unknown_entity_then_returns_none() {
    // Arrange
    let backend = RapierBackend::new(Vec2::ZERO);
    let entity = spawn_entity();

    // Act
    let vel = backend.body_linear_velocity(entity);

    // Assert
    assert!(vel.is_none());
}

/// @doc: `set_linear_velocity` must immediately affect motion — velocity change feeds physics-based drag animation
#[test]
fn when_set_linear_velocity_then_body_moves_accordingly() {
    // Arrange
    let mut backend = RapierBackend::new(Vec2::ZERO);
    let entity = spawn_entity();
    backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);
    backend.add_collider(entity, &Collider::Circle(0.5));
    backend.set_damping(entity, 0.0, 0.0).unwrap();

    // Act
    backend
        .set_linear_velocity(entity, Vec2::new(100.0, 0.0))
        .unwrap();
    backend.step(Seconds(0.1));

    // Assert
    let pos = backend.body_position(entity).unwrap();
    assert!(pos.x > 5.0, "expected body to move right, got x={}", pos.x);
}

/// @doc: `set_angular_velocity` must cause rotation — wrong angular velocity breaks card rotation feedback
#[test]
fn when_set_angular_velocity_then_body_rotates() {
    // Arrange
    let mut backend = RapierBackend::new(Vec2::ZERO);
    let entity = spawn_entity();
    backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);
    backend.add_collider(entity, &Collider::Circle(0.5));
    backend.set_damping(entity, 0.0, 0.0).unwrap();

    // Act
    backend.set_angular_velocity(entity, 10.0).unwrap();
    backend.step(Seconds(0.1));

    // Assert
    let rot = backend.body_rotation(entity).unwrap();
    assert!(rot.abs() > 0.5, "expected body to rotate, got rot={rot}");
}

/// @doc: `body_point_to_world` must apply rotation matrix correctly — wrong transform breaks offsets in torque/drag calculations
#[test]
fn when_rapier_body_rotated_then_body_point_to_world_matches_manual_transform() {
    // Arrange
    let mut backend = RapierBackend::new(Vec2::ZERO);
    let entity = spawn_entity();
    backend.add_body(entity, &RigidBody::Dynamic, Vec2::new(3.0, 4.0));
    backend.add_collider(entity, &Collider::Circle(0.5));
    apply_impulse(
        &mut backend,
        entity,
        Vec2::new(50.0, 0.0),
        Vec2::new(3.0, 5.0),
    );
    for _ in 0..5 {
        backend.step(Seconds(0.016));
    }

    // Act
    let local_offset = Vec2::new(1.0, 0.5);
    let world_pt = backend.body_point_to_world(entity, local_offset).unwrap();

    // Assert
    let expected = manual_rotate_point(&backend, entity, local_offset);
    assert_vec2_approx(world_pt, expected, 1e-4);
}

/// @doc: New body must report zero angular velocity — non-zero initial spin breaks consistency
#[test]
fn when_body_angular_velocity_on_new_body_then_returns_zero() {
    // Arrange
    let mut backend = RapierBackend::new(Vec2::ZERO);
    let entity = spawn_entity();
    backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);

    // Act
    let angvel = backend.body_angular_velocity(entity);

    // Assert
    let angvel = angvel.expect("should return Some for living body");
    assert!(
        angvel.abs() < 1e-4,
        "initial angular velocity should be ~0, got {angvel}"
    );
}

/// @doc: Angular velocity query on non-existent entity must return None — prevents crash in spin feedback
#[test]
fn when_body_angular_velocity_on_unknown_entity_then_returns_none() {
    // Arrange
    let backend = RapierBackend::new(Vec2::ZERO);
    let entity = spawn_entity();

    // Act
    let angvel = backend.body_angular_velocity(entity);

    // Assert
    assert!(angvel.is_none());
}

/// @doc: `body_angular_velocity` must return exact set value — mismatch breaks spin feedback synchronization
#[test]
fn when_body_angular_velocity_after_set_then_returns_set_value() {
    // Arrange
    let mut backend = RapierBackend::new(Vec2::ZERO);
    let entity = spawn_entity();
    backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);
    backend.add_collider(entity, &Collider::Circle(0.5));
    backend.set_damping(entity, 0.0, 0.0).unwrap();
    backend.set_angular_velocity(entity, 5.0).unwrap();

    // Act
    let angvel = backend.body_angular_velocity(entity);

    // Assert
    let angvel = angvel.expect("should return Some");
    assert!((angvel - 5.0).abs() < 1e-4, "expected ~5.0, got {angvel}");
}

/// @doc: Angular velocity sign must be preserved — wrong sign reverses card spin direction
#[test]
fn when_body_angular_velocity_negative_then_sign_preserved() {
    // Arrange
    let mut backend = RapierBackend::new(Vec2::ZERO);
    let entity = spawn_entity();
    backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);
    backend.add_collider(entity, &Collider::Circle(0.5));
    backend.set_damping(entity, 0.0, 0.0).unwrap();
    backend.set_angular_velocity(entity, -3.0).unwrap();

    // Act
    let angvel = backend.body_angular_velocity(entity);

    // Assert
    let angvel = angvel.expect("should return Some");
    assert!(
        (angvel - (-3.0)).abs() < 1e-4,
        "expected ~-3.0, got {angvel}"
    );
}

/// @doc: `set_collision_group` on unknown entity must return Err — prevents silent filter application
#[test]
fn when_set_collision_group_on_unknown_entity_then_returns_err() {
    // Arrange
    let mut backend = RapierBackend::new(Vec2::ZERO);
    let entity = spawn_entity();

    // Act
    let result = backend.set_collision_group(entity, 1, 2);

    // Assert
    assert!(result.is_err());
}

/// @doc: Collision group isolation — entities in same exclusive group never produce collision events
#[test]
fn when_same_exclusive_group_then_no_collision_event() {
    // Arrange
    let mut backend = RapierBackend::new(Vec2::ZERO);
    let entities = spawn_entities(2);
    backend.add_body(entities[0], &RigidBody::Dynamic, Vec2::ZERO);
    backend.add_collider(entities[0], &Collider::Circle(1.0));
    backend.add_body(entities[1], &RigidBody::Dynamic, Vec2::ZERO);
    backend.add_collider(entities[1], &Collider::Circle(1.0));
    // Both cards: member of group 1, filter allows only group 2
    backend
        .set_collision_group(entities[0], 0b0001, 0b0010)
        .unwrap();
    backend
        .set_collision_group(entities[1], 0b0001, 0b0010)
        .unwrap();

    // Act
    backend.step(Seconds(0.016));
    let events = backend.drain_collision_events();

    // Assert
    assert!(
        events.is_empty(),
        "cards in same exclusive group should not collide, got {events:?}"
    );
}

/// @doc: Collision groups must respect membership filters — wrong group setup breaks zone isolation
#[test]
fn when_card_and_wall_groups_then_collision_event_fires() {
    // Arrange
    let mut backend = RapierBackend::new(Vec2::ZERO);
    let entities = spawn_entities(2);
    let card = entities[0];
    let wall = entities[1];
    backend.add_body(card, &RigidBody::Dynamic, Vec2::ZERO);
    backend.add_collider(card, &Collider::Circle(1.0));
    backend.add_body(wall, &RigidBody::Dynamic, Vec2::ZERO);
    backend.add_collider(wall, &Collider::Circle(1.0));
    // Card: member of group 1, collides with group 2
    backend.set_collision_group(card, 0b0001, 0b0010).unwrap();
    // Wall: member of group 2, collides with group 1
    backend.set_collision_group(wall, 0b0010, 0b0001).unwrap();

    // Act
    backend.step(Seconds(0.016));
    let events = backend.drain_collision_events();

    // Assert
    assert_eq!(
        events.len(),
        1,
        "card-wall collision should fire, got {events:?}"
    );
    let pair = (events[0].entity_a, events[0].entity_b);
    assert!(
        pair == (card, wall) || pair == (wall, card),
        "expected card-wall pair, got {pair:?}"
    );
}

/// @doc: Collision filters applied after collider creation must still take effect — backward-compat for zone setup
#[test]
fn when_collision_group_set_after_collider_added_then_filter_applied() {
    // Arrange — add bodies+colliders first, THEN set groups (the expected card game usage)
    let mut backend = RapierBackend::new(Vec2::ZERO);
    let entities = spawn_entities(2);
    backend.add_body(entities[0], &RigidBody::Dynamic, Vec2::ZERO);
    backend.add_collider(entities[0], &Collider::Circle(1.0));
    backend.add_body(entities[1], &RigidBody::Dynamic, Vec2::ZERO);
    backend.add_collider(entities[1], &Collider::Circle(1.0));

    // Retroactively set exclusive groups
    backend
        .set_collision_group(entities[0], 0b0001, 0b0010)
        .unwrap();
    backend
        .set_collision_group(entities[1], 0b0001, 0b0010)
        .unwrap();

    // Act
    backend.step(Seconds(0.016));
    let events = backend.drain_collision_events();

    // Assert
    assert!(
        events.is_empty(),
        "retroactive group filter should suppress collision, got {events:?}"
    );
}

/// @doc: Duplicate `add_body` calls must return false — silently succeeding would
/// create a second rapier handle for the same entity, leaking the first handle and
/// making `remove_body` only clean up one of them. The warning log aids debugging
/// when a system accidentally re-registers an entity.
#[test]
fn when_add_body_called_twice_for_same_entity_then_returns_false() {
    // Arrange
    let mut backend = RapierBackend::new(Vec2::ZERO);
    let entity = spawn_entity();
    backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);

    // Act
    let result = backend.add_body(entity, &RigidBody::Dynamic, Vec2::new(10.0, 10.0));

    // Assert
    assert!(!result, "duplicate add_body must return false");
}

/// @doc: A degenerate convex hull (e.g., collinear points) causes rapier's
/// `convex_hull` builder to return None. `add_collider` must return false rather
/// than panicking — the warning log surfaces the geometry issue to the developer.
#[test]
fn when_add_collider_with_degenerate_hull_then_returns_false() {
    // Arrange
    let mut backend = RapierBackend::new(Vec2::ZERO);
    let entity = spawn_entity();
    backend.add_body(entity, &RigidBody::Dynamic, Vec2::ZERO);
    // Three collinear points — rapier cannot form a 2D convex hull.
    let degenerate = Collider::ConvexPolygon(vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(2.0, 0.0),
    ]);

    // Act
    let result = backend.add_collider(entity, &degenerate);

    // Assert
    assert!(!result, "degenerate convex hull must return false");
}
