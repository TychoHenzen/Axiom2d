#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::{Schedule, World};
use engine_core::prelude::EventBus;
use engine_physics::prelude::{PhysicsCommand, PhysicsRes};

use card_game::card::component::CardZone;
use card_game::card::interaction::sleep::card_sleep_system;

use crate::suite::helpers::default_card_collider;
use crate::test_helpers::{SpyPhysicsBackend, make_test_card};

fn run_system(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(card_sleep_system);
    schedule.run(world);
}

fn sleep_commands(world: &mut World) -> Vec<bevy_ecs::prelude::Entity> {
    world
        .resource_mut::<EventBus<PhysicsCommand>>()
        .drain()
        .filter_map(|cmd| match cmd {
            PhysicsCommand::SleepBody { entity } => Some(entity),
            _ => None,
        })
        .collect()
}

/// @doc: Table cards that have come to rest (both linear and angular velocity below
/// the sleep threshold) must receive a `SleepBody` command so rapier stops simulating
/// them. Without this, every card on the table consumes physics budget every tick
/// even when completely stationary, which wastes CPU and causes floating-point
/// drift that can slowly nudge cards out of position over time.
#[test]
fn when_table_card_below_rest_threshold_then_sleep_body_command_emitted() {
    // Arrange
    let mut world = World::new();
    let entity = world
        .spawn((make_test_card(), CardZone::Table, default_card_collider()))
        .id();
    let spy = SpyPhysicsBackend::new().with_angular_velocity(entity, 0.0);
    world.insert_resource(PhysicsRes::new(Box::new(spy)));
    world.insert_resource(EventBus::<PhysicsCommand>::default());

    // Act
    run_system(&mut world);

    // Assert
    let sleeping = sleep_commands(&mut world);
    assert_eq!(
        sleeping,
        vec![entity],
        "resting table card should get SleepBody command"
    );
}

/// @doc: Cards still moving linearly must not be put to sleep — a sliding card
/// that gets force-slept would freeze mid-motion and teleport-stop, which looks
/// broken and defeats the physics simulation's purpose.
#[test]
fn when_table_card_above_linear_threshold_then_no_sleep_command() {
    // Arrange
    let mut world = World::new();
    let entity = world
        .spawn((make_test_card(), CardZone::Table, default_card_collider()))
        .id();
    let spy = SpyPhysicsBackend::new()
        .with_linear_velocity(entity, glam::Vec2::new(10.0, 5.0))
        .with_angular_velocity(entity, 0.0);
    world.insert_resource(PhysicsRes::new(Box::new(spy)));
    world.insert_resource(EventBus::<PhysicsCommand>::default());

    // Act
    run_system(&mut world);

    // Assert
    let sleeping = sleep_commands(&mut world);
    assert!(
        sleeping.is_empty(),
        "moving card should not be put to sleep"
    );
}

/// @doc: Cards still spinning must not be put to sleep — a spinning card that gets
/// force-slept would freeze at an arbitrary angle, which looks glitchy and breaks
/// the satisfying physics-based settling animation.
#[test]
fn when_table_card_above_angular_threshold_then_no_sleep_command() {
    // Arrange
    let mut world = World::new();
    let entity = world
        .spawn((make_test_card(), CardZone::Table, default_card_collider()))
        .id();
    let spy = SpyPhysicsBackend::new().with_angular_velocity(entity, 5.0);
    world.insert_resource(PhysicsRes::new(Box::new(spy)));
    world.insert_resource(EventBus::<PhysicsCommand>::default());

    // Act
    run_system(&mut world);

    // Assert
    let sleeping = sleep_commands(&mut world);
    assert!(
        sleeping.is_empty(),
        "spinning card should not be put to sleep"
    );
}

/// @doc: A card already sleeping must not receive a redundant `SleepBody` command.
/// Without this guard, the sleep system would push a command every tick for every
/// resting card, flooding the event bus with no-ops and potentially interfering
/// with rapier's internal sleep bookkeeping.
#[test]
fn when_table_card_already_sleeping_then_no_redundant_sleep_command() {
    // Arrange
    let mut world = World::new();
    let entity = world
        .spawn((make_test_card(), CardZone::Table, default_card_collider()))
        .id();
    let spy = SpyPhysicsBackend::new()
        .with_angular_velocity(entity, 0.0)
        .with_body_sleeping(entity, true);
    world.insert_resource(PhysicsRes::new(Box::new(spy)));
    world.insert_resource(EventBus::<PhysicsCommand>::default());

    // Act
    run_system(&mut world);

    // Assert
    let sleeping = sleep_commands(&mut world);
    assert!(
        sleeping.is_empty(),
        "already-sleeping card should not get redundant SleepBody"
    );
}

/// @doc: Sleep is only relevant for table cards that have physics bodies. Hand and
/// stash cards have no rigid body — the sleep system must skip them entirely.
/// Emitting `SleepBody` for a hand card would produce an `EntityNotFound` error
/// from the physics backend.
#[test]
fn when_hand_card_then_no_sleep_command() {
    // Arrange
    let mut world = World::new();
    let table_entity = world
        .spawn((make_test_card(), CardZone::Table, default_card_collider()))
        .id();
    let _hand_entity = world
        .spawn((make_test_card(), CardZone::Hand(0), default_card_collider()))
        .id();
    let spy = SpyPhysicsBackend::new().with_angular_velocity(table_entity, 0.0);
    world.insert_resource(PhysicsRes::new(Box::new(spy)));
    world.insert_resource(EventBus::<PhysicsCommand>::default());

    // Act
    run_system(&mut world);

    // Assert
    let sleeping = sleep_commands(&mut world);
    assert_eq!(
        sleeping,
        vec![table_entity],
        "only table card should get SleepBody"
    );
}

/// @doc: The sleep system must iterate all qualifying table cards, not short-circuit
/// after the first match. Multiple cards settling simultaneously must all enter
/// sleep on the same frame.
#[test]
fn when_multiple_resting_table_cards_then_sleep_body_emitted_for_each() {
    // Arrange
    let mut world = World::new();
    let e1 = world
        .spawn((make_test_card(), CardZone::Table, default_card_collider()))
        .id();
    let e2 = world
        .spawn((make_test_card(), CardZone::Table, default_card_collider()))
        .id();
    let e3 = world
        .spawn((make_test_card(), CardZone::Table, default_card_collider()))
        .id();
    let spy = SpyPhysicsBackend::new()
        .with_angular_velocity(e1, 0.0)
        .with_angular_velocity(e2, 0.0)
        .with_angular_velocity(e3, 0.0);
    world.insert_resource(PhysicsRes::new(Box::new(spy)));
    world.insert_resource(EventBus::<PhysicsCommand>::default());

    // Act
    run_system(&mut world);

    // Assert
    let mut sleeping = sleep_commands(&mut world);
    sleeping.sort();
    let mut expected = vec![e1, e2, e3];
    expected.sort();
    assert_eq!(
        sleeping, expected,
        "all resting table cards should get SleepBody"
    );
}

/// @doc: A table card whose physics body hasn't been registered yet (velocity
/// queries return `None`) must be silently skipped. This happens during the brief
/// window between spawning a card entity and the physics command system creating
/// its rigid body — the sleep system runs every `FixedUpdate` tick and must handle
/// this transient state without errors.
#[test]
fn when_table_card_has_no_physics_body_then_no_command_and_no_error() {
    // Arrange
    let mut world = World::new();
    let _entity = world
        .spawn((make_test_card(), CardZone::Table, default_card_collider()))
        .id();
    // Default spy returns None for angular_velocity on unknown entities
    let spy = SpyPhysicsBackend::new();
    world.insert_resource(PhysicsRes::new(Box::new(spy)));
    world.insert_resource(EventBus::<PhysicsCommand>::default());

    // Act
    run_system(&mut world);

    // Assert
    let sleeping = sleep_commands(&mut world);
    assert!(
        sleeping.is_empty(),
        "card with no physics body should be skipped"
    );
}
