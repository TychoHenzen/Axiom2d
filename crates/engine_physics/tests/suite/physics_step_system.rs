use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use bevy_ecs::prelude::{Schedule, World};
use engine_core::prelude::{DeltaTime, EventBus, Seconds};
use engine_physics::collision_event::{CollisionEvent, CollisionKind};
use engine_physics::physics_res::PhysicsRes;
use engine_physics::physics_step_system::physics_step_system;
use engine_physics::test_helpers::{SpyPhysicsBackend, spawn_entities};

fn setup_world(step_count: Arc<AtomicU32>) -> World {
    let mut world = World::new();
    world.insert_resource(PhysicsRes::new(Box::new(
        SpyPhysicsBackend::new().with_step_count(Arc::clone(&step_count)),
    )));
    world.insert_resource(EventBus::<CollisionEvent>::default());
    world.insert_resource(DeltaTime(Seconds(0.016)));
    world
}

/// @doc: The physics step system must call the backend's step method exactly once per frame.
#[test]
fn when_system_runs_then_backend_is_stepped() {
    // Arrange
    let step_count = Arc::new(AtomicU32::new(0));
    let mut world = setup_world(Arc::clone(&step_count));
    let mut schedule = Schedule::default();
    schedule.add_systems(physics_step_system);

    // Act
    schedule.run(&mut world);

    // Assert
    assert_eq!(
        step_count.load(Ordering::Relaxed),
        1,
        "backend step count should be 1 after one system tick"
    );
}

/// @doc: When no collisions occur during the step, the CollisionEvent bus must remain empty.
#[test]
fn when_system_runs_with_no_events_then_buffer_remains_empty() {
    // Arrange
    let step_count = Arc::new(AtomicU32::new(0));
    let mut world = setup_world(Arc::clone(&step_count));
    let mut schedule = Schedule::default();
    schedule.add_systems(physics_step_system);

    // Act
    schedule.run(&mut world);

    // Assert
    let mut bus = world.resource_mut::<EventBus<CollisionEvent>>();
    let events: Vec<_> = bus.drain().collect();
    assert!(
        events.is_empty(),
        "collision event bus should be empty when no collisions occur"
    );
}

/// @doc: Running the system twice must step the backend twice (no skipped or collapsed steps).
#[test]
fn when_system_runs_twice_then_backend_stepped_twice() {
    // Arrange
    let step_count = Arc::new(AtomicU32::new(0));
    let mut world = setup_world(Arc::clone(&step_count));
    let mut schedule = Schedule::default();
    schedule.add_systems(physics_step_system);

    // Act
    schedule.run(&mut world);
    schedule.run(&mut world);

    // Assert
    assert_eq!(
        step_count.load(Ordering::Relaxed),
        2,
        "backend step count should be 2 after two system ticks"
    );
}

/// @doc: When the backend reports collision events, the system must forward them into the ECS EventBus.
#[test]
fn when_backend_produces_events_then_buffer_contains_them() {
    // Arrange
    let step_count = Arc::new(AtomicU32::new(0));
    let entities = spawn_entities(2);
    let event = CollisionEvent {
        entity_a: entities[0],
        entity_b: entities[1],
        kind: CollisionKind::Started,
    };
    let mut world = World::new();
    world.insert_resource(PhysicsRes::new(Box::new(
        SpyPhysicsBackend::new()
            .with_step_count(Arc::clone(&step_count))
            .with_events(vec![event]),
    )));
    world.insert_resource(EventBus::<CollisionEvent>::default());
    world.insert_resource(DeltaTime(Seconds(0.016)));
    let mut schedule = Schedule::default();
    schedule.add_systems(physics_step_system);

    // Act
    schedule.run(&mut world);

    // Assert
    let mut bus = world.resource_mut::<EventBus<CollisionEvent>>();
    let events: Vec<_> = bus.drain().collect();
    assert_eq!(
        events.len(),
        1,
        "should have exactly 1 collision event after system tick"
    );
    assert_eq!(
        events[0], event,
        "the forwarded event must match the backend's output"
    );
}
