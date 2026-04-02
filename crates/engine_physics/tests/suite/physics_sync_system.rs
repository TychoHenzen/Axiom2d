use bevy_ecs::prelude::{Schedule, World};
use engine_core::prelude::Transform2D;
use engine_physics::physics_backend::NullPhysicsBackend;
use engine_physics::physics_res::PhysicsRes;
use engine_physics::physics_sync_system::physics_sync_system;
use engine_physics::rigid_body::RigidBody;
use engine_physics::test_helpers::SpyPhysicsBackend;
use glam::Vec2;

fn run_sync(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(physics_sync_system);
    schedule.run(world);
}

#[test]
fn when_no_entities_have_rigid_body_then_system_runs_without_panic() {
    // Arrange
    let mut world = World::new();
    world.insert_resource(PhysicsRes::new(Box::new(NullPhysicsBackend::default())));

    // Act + Assert
    run_sync(&mut world);
}

/// @doc: One-way sync: physics backend -> `Transform2D`. ECS is the read side, rapier is the authority
#[test]
fn when_backend_returns_position_then_transform_position_is_updated() {
    // Arrange
    let mut world = World::new();
    let entity = world
        .spawn((RigidBody::Dynamic, Transform2D::default()))
        .id();
    world.insert_resource(PhysicsRes::new(Box::new(
        SpyPhysicsBackend::new().with_position(entity, Vec2::new(10.0, 20.0)),
    )));

    // Act
    run_sync(&mut world);

    // Assert
    let transform = world.get::<Transform2D>(entity).unwrap();
    assert_eq!(transform.position, Vec2::new(10.0, 20.0));
    assert!(transform.rotation.abs() < f32::EPSILON);
}

#[test]
fn when_backend_returns_rotation_then_transform_rotation_is_updated() {
    // Arrange
    let mut world = World::new();
    let entity = world
        .spawn((RigidBody::Dynamic, Transform2D::default()))
        .id();
    world.insert_resource(PhysicsRes::new(Box::new(
        SpyPhysicsBackend::new().with_rotation(entity, std::f32::consts::FRAC_PI_4),
    )));

    // Act
    run_sync(&mut world);

    // Assert
    let transform = world.get::<Transform2D>(entity).unwrap();
    assert!((transform.rotation - std::f32::consts::FRAC_PI_4).abs() < 1e-6);
    assert_eq!(transform.position, Vec2::ZERO);
}

#[test]
fn when_backend_returns_both_position_and_rotation_then_both_fields_updated() {
    // Arrange
    let mut world = World::new();
    let entity = world
        .spawn((RigidBody::Dynamic, Transform2D::default()))
        .id();
    world.insert_resource(PhysicsRes::new(Box::new(
        SpyPhysicsBackend::new().with_body(
            entity,
            Vec2::new(5.0, -3.0),
            std::f32::consts::FRAC_PI_2,
        ),
    )));

    // Act
    run_sync(&mut world);

    // Assert
    let transform = world.get::<Transform2D>(entity).unwrap();
    assert_eq!(transform.position, Vec2::new(5.0, -3.0));
    assert!((transform.rotation - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
}

#[test]
fn when_backend_returns_none_for_unregistered_entity_then_transform_is_unchanged() {
    // Arrange
    let mut world = World::new();
    let entity = world
        .spawn((
            RigidBody::Dynamic,
            Transform2D {
                position: Vec2::new(99.0, 99.0),
                rotation: 1.0,
                ..Transform2D::default()
            },
        ))
        .id();
    world.insert_resource(PhysicsRes::new(Box::new(SpyPhysicsBackend::new())));

    // Act
    run_sync(&mut world);

    // Assert
    let transform = world.get::<Transform2D>(entity).unwrap();
    assert_eq!(transform.position, Vec2::new(99.0, 99.0));
    assert!((transform.rotation - 1.0).abs() < f32::EPSILON);
}

/// @doc: Position and rotation are synced independently -- either can be None without affecting the other
#[test]
fn when_backend_returns_position_only_then_rotation_field_is_unchanged() {
    // Arrange
    let mut world = World::new();
    let entity = world
        .spawn((
            RigidBody::Dynamic,
            Transform2D {
                rotation: 2.5,
                ..Transform2D::default()
            },
        ))
        .id();
    world.insert_resource(PhysicsRes::new(Box::new(
        SpyPhysicsBackend::new().with_position(entity, Vec2::new(1.0, 2.0)),
    )));

    // Act
    run_sync(&mut world);

    // Assert
    let transform = world.get::<Transform2D>(entity).unwrap();
    assert_eq!(transform.position, Vec2::new(1.0, 2.0));
    assert!((transform.rotation - 2.5).abs() < f32::EPSILON);
}

#[test]
fn when_backend_returns_rotation_only_then_position_field_is_unchanged() {
    // Arrange
    let mut world = World::new();
    let entity = world
        .spawn((
            RigidBody::Dynamic,
            Transform2D {
                position: Vec2::new(7.0, 8.0),
                ..Transform2D::default()
            },
        ))
        .id();
    world.insert_resource(PhysicsRes::new(Box::new(
        SpyPhysicsBackend::new().with_rotation(entity, 0.5),
    )));

    // Act
    run_sync(&mut world);

    // Assert
    let transform = world.get::<Transform2D>(entity).unwrap();
    assert_eq!(transform.position, Vec2::new(7.0, 8.0));
    assert!((transform.rotation - 0.5).abs() < f32::EPSILON);
}

#[test]
fn when_system_runs_then_transform_scale_is_never_modified() {
    // Arrange
    let mut world = World::new();
    let entity = world
        .spawn((
            RigidBody::Dynamic,
            Transform2D {
                scale: Vec2::new(3.0, 0.5),
                ..Transform2D::default()
            },
        ))
        .id();
    world.insert_resource(PhysicsRes::new(Box::new(
        SpyPhysicsBackend::new().with_body(entity, Vec2::new(1.0, 1.0), 1.0),
    )));

    // Act
    run_sync(&mut world);

    // Assert
    let transform = world.get::<Transform2D>(entity).unwrap();
    assert_eq!(transform.scale, Vec2::new(3.0, 0.5));
}

#[test]
fn when_multiple_entities_registered_then_each_entity_receives_its_own_transform() {
    // Arrange
    let mut world = World::new();
    let entity_a = world
        .spawn((RigidBody::Dynamic, Transform2D::default()))
        .id();
    let entity_b = world
        .spawn((RigidBody::Dynamic, Transform2D::default()))
        .id();
    world.insert_resource(PhysicsRes::new(Box::new(
        SpyPhysicsBackend::new()
            .with_position(entity_a, Vec2::new(1.0, 0.0))
            .with_position(entity_b, Vec2::new(0.0, 2.0)),
    )));

    // Act
    run_sync(&mut world);

    // Assert
    let transform_a = world.get::<Transform2D>(entity_a).unwrap();
    assert_eq!(transform_a.position, Vec2::new(1.0, 0.0));
    let transform_b = world.get::<Transform2D>(entity_b).unwrap();
    assert_eq!(transform_b.position, Vec2::new(0.0, 2.0));
}

/// @doc: Only entities with `RigidBody` participate in physics sync -- plain transforms are untouched
#[test]
fn when_entity_has_no_rigid_body_then_its_transform_is_not_touched() {
    // Arrange
    let mut world = World::new();
    let physics_entity = world
        .spawn((RigidBody::Dynamic, Transform2D::default()))
        .id();
    let plain_entity = world
        .spawn(Transform2D {
            position: Vec2::new(50.0, 50.0),
            ..Transform2D::default()
        })
        .id();
    world.insert_resource(PhysicsRes::new(Box::new(
        SpyPhysicsBackend::new().with_position(physics_entity, Vec2::new(1.0, 1.0)),
    )));

    // Act
    run_sync(&mut world);

    // Assert
    let transform = world.get::<Transform2D>(plain_entity).unwrap();
    assert_eq!(transform.position, Vec2::new(50.0, 50.0));
}

#[test]
fn when_rigid_body_is_static_then_transform_is_still_synced() {
    // Arrange
    let mut world = World::new();
    let entity = world
        .spawn((RigidBody::Static, Transform2D::default()))
        .id();
    world.insert_resource(PhysicsRes::new(Box::new(
        SpyPhysicsBackend::new().with_position(entity, Vec2::new(3.0, 4.0)),
    )));

    // Act
    run_sync(&mut world);

    // Assert
    let transform = world.get::<Transform2D>(entity).unwrap();
    assert_eq!(transform.position, Vec2::new(3.0, 4.0));
}

#[test]
fn when_rigid_body_is_kinematic_then_transform_is_still_synced() {
    // Arrange
    let mut world = World::new();
    let entity = world
        .spawn((RigidBody::Kinematic, Transform2D::default()))
        .id();
    world.insert_resource(PhysicsRes::new(Box::new(
        SpyPhysicsBackend::new().with_position(entity, Vec2::new(6.0, 7.0)),
    )));

    // Act
    run_sync(&mut world);

    // Assert
    let transform = world.get::<Transform2D>(entity).unwrap();
    assert_eq!(transform.position, Vec2::new(6.0, 7.0));
}
