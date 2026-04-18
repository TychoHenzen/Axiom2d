#![allow(clippy::unwrap_used)]

use std::sync::{Arc, Mutex};

use bevy_ecs::prelude::Schedule;
use bevy_ecs::prelude::World;
use engine_core::prelude::EventBus;
use engine_physics::collider::Collider;
use engine_physics::physics_command_apply_system::physics_command_apply_system;
use engine_physics::physics_res::PhysicsRes;
use engine_physics::prelude::PhysicsCommand;
use engine_physics::rigid_body::RigidBody;
use engine_physics::test_helpers::RecordingPhysicsBackend;
use glam::Vec2;

fn make_world() -> (World, Arc<Mutex<Vec<String>>>) {
    let calls = Arc::new(Mutex::new(Vec::<String>::new()));
    let backend = RecordingPhysicsBackend::new(Arc::clone(&calls));
    let mut world = World::new();
    world.insert_resource(PhysicsRes::new(Box::new(backend)));
    world.insert_resource(EventBus::<PhysicsCommand>::default());
    (world, calls)
}

#[test]
fn when_no_commands_queued_then_call_log_is_empty() {
    // Arrange
    let (mut world, calls) = make_world();
    let mut schedule = Schedule::default();
    schedule.add_systems(physics_command_apply_system);

    // Act
    schedule.run(&mut world);

    // Assert
    let recorded = calls.lock().unwrap();
    assert!(
        recorded.is_empty(),
        "expected no backend mutation calls but got: {recorded:?}"
    );
}

#[test]
fn when_lifecycle_commands_queued_then_all_three_backend_methods_called_in_order() {
    // Arrange
    let (mut world, calls) = make_world();
    let entity = world.spawn(()).id();
    world
        .resource_mut::<EventBus<PhysicsCommand>>()
        .push(PhysicsCommand::AddBody {
            entity,
            body_type: RigidBody::Dynamic,
            position: Vec2::new(1.0, 2.0),
        });
    world
        .resource_mut::<EventBus<PhysicsCommand>>()
        .push(PhysicsCommand::AddCollider {
            entity,
            collider: Collider::Circle(0.5),
        });
    world
        .resource_mut::<EventBus<PhysicsCommand>>()
        .push(PhysicsCommand::RemoveBody { entity });
    let mut schedule = Schedule::default();
    schedule.add_systems(physics_command_apply_system);

    // Act
    schedule.run(&mut world);

    // Assert
    let recorded = calls.lock().unwrap();
    assert_eq!(
        *recorded,
        vec!["add_body", "add_collider", "remove_body"],
        "expected lifecycle commands dispatched in insertion order, got: {recorded:?}"
    );
}

#[test]
fn when_property_commands_queued_then_all_six_backend_methods_called_in_order() {
    // Arrange
    let (mut world, calls) = make_world();
    let entity = world.spawn(()).id();
    let bus = &mut *world.resource_mut::<EventBus<PhysicsCommand>>();
    bus.push(PhysicsCommand::SetLinearVelocity {
        entity,
        velocity: Vec2::new(1.0, 0.0),
    });
    bus.push(PhysicsCommand::SetAngularVelocity {
        entity,
        angular_velocity: 0.5,
    });
    bus.push(PhysicsCommand::SetDamping {
        entity,
        linear: 0.3,
        angular: 0.1,
    });
    bus.push(PhysicsCommand::SetCollisionGroup {
        entity,
        membership: 0b0001,
        filter: 0b0011,
    });
    bus.push(PhysicsCommand::SetBodyPosition {
        entity,
        position: Vec2::new(10.0, -5.0),
    });
    bus.push(PhysicsCommand::AddForceAtPoint {
        entity,
        force: Vec2::new(0.0, 100.0),
        world_point: Vec2::new(1.0, 2.0),
    });
    let mut schedule = Schedule::default();
    schedule.add_systems(physics_command_apply_system);

    // Act
    schedule.run(&mut world);

    // Assert
    let recorded = calls.lock().unwrap();
    assert_eq!(
        *recorded,
        vec![
            "set_linear_velocity",
            "set_angular_velocity",
            "set_damping",
            "set_collision_group",
            "set_body_position",
            "add_force_at_point",
        ],
        "expected all property commands dispatched in insertion order, got: {recorded:?}"
    );
}

#[test]
fn when_mixed_commands_queued_then_all_applied_in_insertion_order() {
    // Arrange
    let (mut world, calls) = make_world();
    let entity = world.spawn(()).id();
    let bus = &mut *world.resource_mut::<EventBus<PhysicsCommand>>();
    bus.push(PhysicsCommand::AddBody {
        entity,
        body_type: RigidBody::Dynamic,
        position: Vec2::ZERO,
    });
    bus.push(PhysicsCommand::SetLinearVelocity {
        entity,
        velocity: Vec2::X,
    });
    bus.push(PhysicsCommand::RemoveBody { entity });
    let mut schedule = Schedule::default();
    schedule.add_systems(physics_command_apply_system);

    // Act
    schedule.run(&mut world);

    // Assert
    let recorded = calls.lock().unwrap();
    assert_eq!(
        *recorded,
        vec!["add_body", "set_linear_velocity", "remove_body"],
        "mixed command types must preserve insertion order"
    );
}

#[test]
fn when_system_runs_twice_then_commands_do_not_replay() {
    // Arrange
    let (mut world, calls) = make_world();
    let entity = world.spawn(()).id();
    world
        .resource_mut::<EventBus<PhysicsCommand>>()
        .push(PhysicsCommand::AddBody {
            entity,
            body_type: RigidBody::Dynamic,
            position: Vec2::ZERO,
        });
    let mut schedule = Schedule::default();
    schedule.add_systems(physics_command_apply_system);

    // Act
    schedule.run(&mut world);
    schedule.run(&mut world);

    // Assert
    let recorded = calls.lock().unwrap();
    assert_eq!(
        recorded.len(),
        1,
        "command must be applied once, not replayed on second run; got: {recorded:?}"
    );
}

/// @doc: The `SleepBody` command must route through `physics_command_apply_system`
/// to call `backend.sleep_body()`. The auto-sleep system and forced-sleep-on-release
/// both emit this command — if the dispatch arm is missing, sleeping cards would
/// remain awake and keep consuming physics budget indefinitely.
#[test]
fn when_sleep_body_command_queued_then_backend_sleep_body_called() {
    // Arrange
    let (mut world, calls) = make_world();
    let entity = world.spawn(()).id();
    world
        .resource_mut::<EventBus<PhysicsCommand>>()
        .push(PhysicsCommand::SleepBody { entity });
    let mut schedule = Schedule::default();
    schedule.add_systems(physics_command_apply_system);

    // Act
    schedule.run(&mut world);

    // Assert
    let recorded = calls.lock().unwrap();
    assert_eq!(*recorded, vec!["sleep_body"]);
}

/// @doc: The `WakeBody` command must route to `backend.wake_body()`. The pick-up
/// system emits this when grabbing a sleeping card — without the dispatch arm,
/// the drag system would apply forces to a body that rapier still considers
/// sleeping, making the card unresponsive to player input.
#[test]
fn when_wake_body_command_queued_then_backend_wake_body_called() {
    // Arrange
    let (mut world, calls) = make_world();
    let entity = world.spawn(()).id();
    world
        .resource_mut::<EventBus<PhysicsCommand>>()
        .push(PhysicsCommand::WakeBody { entity });
    let mut schedule = Schedule::default();
    schedule.add_systems(physics_command_apply_system);

    // Act
    schedule.run(&mut world);

    // Assert
    let recorded = calls.lock().unwrap();
    assert_eq!(*recorded, vec!["wake_body"]);
}

/// @doc: Sleep and wake commands queued in the same frame must execute in insertion
/// order. A card that is force-slept then immediately picked up in the same frame
/// would queue `SleepBody` followed by `WakeBody` — reversing the order would
/// leave the card sleeping when the player expects it to be draggable.
#[test]
fn when_sleep_then_wake_commands_queued_then_both_called_in_order() {
    // Arrange
    let (mut world, calls) = make_world();
    let entity = world.spawn(()).id();
    let bus = &mut *world.resource_mut::<EventBus<PhysicsCommand>>();
    bus.push(PhysicsCommand::SleepBody { entity });
    bus.push(PhysicsCommand::WakeBody { entity });
    let mut schedule = Schedule::default();
    schedule.add_systems(physics_command_apply_system);

    // Act
    schedule.run(&mut world);

    // Assert
    let recorded = calls.lock().unwrap();
    assert_eq!(*recorded, vec!["sleep_body", "wake_body"]);
}
