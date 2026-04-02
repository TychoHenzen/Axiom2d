#![allow(clippy::unwrap_used)]

use std::sync::{Arc, Mutex};

use bevy_ecs::prelude::*;
use card_game::card::component::{Card, CardZone};
use card_game::card::interaction::damping::{
    BASE_ANGULAR_DRAG, BASE_LINEAR_DRAG, card_damping_system, compute_card_damping,
};
use card_game::test_helpers::{DampingLog, SpyPhysicsBackend};
use engine_core::prelude::TextureId;
use engine_physics::prelude::PhysicsRes;

/// @doc: At zero spin, cards receive full base drag so they slow down
/// quickly when released. This is the default resting state — without it,
/// cards would slide indefinitely on the table after a gentle push.
#[test]
fn when_zero_angular_velocity_then_base_drag_returned() {
    // Arrange / Act
    let (linear, angular) = compute_card_damping(0.0);

    // Assert
    assert!(
        (linear - BASE_LINEAR_DRAG).abs() < 1e-6,
        "expected linear={BASE_LINEAR_DRAG}, got {linear}"
    );
    assert!(
        (angular - BASE_ANGULAR_DRAG).abs() < 1e-6,
        "expected angular={BASE_ANGULAR_DRAG}, got {angular}"
    );
}

/// @doc: Fast-spinning cards experience reduced drag to prevent premature slowdown during flicks
#[test]
fn when_high_angular_velocity_then_drag_less_than_base() {
    // Arrange / Act
    let (linear, angular) = compute_card_damping(20.0);

    // Assert
    assert!(
        linear < BASE_LINEAR_DRAG,
        "expected linear < {BASE_LINEAR_DRAG}, got {linear}"
    );
    assert!(
        angular < BASE_ANGULAR_DRAG,
        "expected angular < {BASE_ANGULAR_DRAG}, got {angular}"
    );
}

/// @doc: Damping uses absolute angular velocity — spin direction doesn't
/// affect drag. A clockwise flick should feel identical to counterclockwise.
#[test]
fn when_negative_angular_velocity_then_same_as_positive() {
    // Arrange / Act
    let positive = compute_card_damping(5.0);
    let negative = compute_card_damping(-5.0);

    // Assert
    assert!(
        (positive.0 - negative.0).abs() < 1e-6,
        "linear: positive={}, negative={}",
        positive.0,
        negative.0
    );
    assert!(
        (positive.1 - negative.1).abs() < 1e-6,
        "angular: positive={}, negative={}",
        positive.1,
        negative.1
    );
}

/// @doc: Extreme spin rates floor at minimum drag—prevents drag collapse and maintains playability
#[test]
fn when_extreme_angular_velocity_then_drag_floored_at_minimum() {
    // Arrange / Act
    let (linear, angular) = compute_card_damping(1000.0);

    // Assert
    // MIN_DRAG_FACTOR is pub(crate), so we check that drag is above zero and well below base
    assert!(linear > 0.0, "linear drag should be positive");
    assert!(angular > 0.0, "angular drag should be positive");
    assert!(
        linear < BASE_LINEAR_DRAG,
        "linear drag should be below base"
    );
    assert!(
        angular < BASE_ANGULAR_DRAG,
        "angular drag should be below base"
    );
}

/// @doc: Drag curve must be monotonically decreasing — faster spin always
/// means less drag. A non-monotonic curve would create a "dead zone" where
/// increasing spin paradoxically slows the card more, breaking the feel of
/// flick-to-spin interactions.
#[test]
fn when_increasing_angular_velocity_then_drag_monotonically_decreases() {
    // Arrange
    let omegas = [0.0, 1.0, 5.0, 10.0, 20.0];

    // Act
    let drags: Vec<(f32, f32)> = omegas.iter().map(|&w| compute_card_damping(w)).collect();

    // Assert
    for i in 1..drags.len() {
        assert!(
            drags[i].0 <= drags[i - 1].0,
            "linear drag should decrease: omega={} gave {}, omega={} gave {}",
            omegas[i - 1],
            drags[i - 1].0,
            omegas[i],
            drags[i].0
        );
    }
}

fn run_system(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(card_damping_system);
    schedule.run(world);
}

#[test]
fn when_non_card_entity_then_set_damping_not_called() {
    // Arrange
    let damping_log: DampingLog = Arc::new(Mutex::new(Vec::new()));
    let mut world = World::new();
    world.insert_resource(PhysicsRes::new(Box::new(
        SpyPhysicsBackend::new().with_damping_log(damping_log.clone()),
    )));
    world.spawn_empty(); // entity with no Card component

    // Act
    run_system(&mut world);

    // Assert
    assert!(damping_log.lock().unwrap().is_empty());
}

#[test]
fn when_card_on_table_with_zero_spin_then_base_damping_applied() {
    // Arrange
    let damping_log: DampingLog = Arc::new(Mutex::new(Vec::new()));
    let mut world = World::new();
    let entity = world
        .spawn((Card::face_down(TextureId(0), TextureId(0)), CardZone::Table))
        .id();
    let spy = SpyPhysicsBackend::new()
        .with_damping_log(damping_log.clone())
        .with_angular_velocity(entity, 0.0);
    world.insert_resource(PhysicsRes::new(Box::new(spy)));

    // Act
    run_system(&mut world);

    // Assert
    let calls = damping_log.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, entity);
    assert!((calls[0].1 - BASE_LINEAR_DRAG).abs() < 1e-4);
    assert!((calls[0].2 - BASE_ANGULAR_DRAG).abs() < 1e-4);
}

#[test]
fn when_card_on_table_with_high_spin_then_reduced_damping_applied() {
    // Arrange
    let damping_log: DampingLog = Arc::new(Mutex::new(Vec::new()));
    let mut world = World::new();
    let entity = world
        .spawn((Card::face_down(TextureId(0), TextureId(0)), CardZone::Table))
        .id();
    let spy = SpyPhysicsBackend::new()
        .with_damping_log(damping_log.clone())
        .with_angular_velocity(entity, 20.0);
    world.insert_resource(PhysicsRes::new(Box::new(spy)));

    // Act
    run_system(&mut world);

    // Assert
    let calls = damping_log.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert!(
        calls[0].1 < BASE_LINEAR_DRAG,
        "expected reduced linear drag"
    );
    assert!(
        calls[0].2 < BASE_ANGULAR_DRAG,
        "expected reduced angular drag"
    );
}

#[test]
fn when_card_with_no_physics_body_then_no_panic_and_no_damping() {
    // Arrange
    let damping_log: DampingLog = Arc::new(Mutex::new(Vec::new()));
    let mut world = World::new();
    world.spawn((Card::face_down(TextureId(0), TextureId(0)), CardZone::Table));
    // Spy has no angular velocity entry -> body_angular_velocity returns None
    world.insert_resource(PhysicsRes::new(Box::new(
        SpyPhysicsBackend::new().with_damping_log(damping_log.clone()),
    )));

    // Act
    run_system(&mut world);

    // Assert
    assert!(damping_log.lock().unwrap().is_empty());
}

#[test]
fn when_multiple_cards_on_table_then_set_damping_called_for_each() {
    // Arrange
    let damping_log: DampingLog = Arc::new(Mutex::new(Vec::new()));
    let mut world = World::new();
    let e1 = world
        .spawn((Card::face_down(TextureId(0), TextureId(0)), CardZone::Table))
        .id();
    let e2 = world
        .spawn((Card::face_down(TextureId(0), TextureId(0)), CardZone::Table))
        .id();
    let e3 = world
        .spawn((Card::face_down(TextureId(0), TextureId(0)), CardZone::Table))
        .id();
    let spy = SpyPhysicsBackend::new()
        .with_damping_log(damping_log.clone())
        .with_angular_velocity(e1, 0.0)
        .with_angular_velocity(e2, 5.0)
        .with_angular_velocity(e3, 20.0);
    world.insert_resource(PhysicsRes::new(Box::new(spy)));

    // Act
    run_system(&mut world);

    // Assert
    let calls = damping_log.lock().unwrap();
    assert_eq!(calls.len(), 3);
    let entities: Vec<Entity> = calls.iter().map(|c| c.0).collect();
    assert!(entities.contains(&e1));
    assert!(entities.contains(&e2));
    assert!(entities.contains(&e3));
}

/// @doc: Cards loaded in a reader have no physics body, so applying damping
/// would query a non-existent body. The damping system must skip Reader-zone
/// cards just as it skips Hand and Stash cards.
#[test]
fn when_card_in_reader_zone_then_set_damping_not_called() {
    // Arrange
    let damping_log: DampingLog = Arc::new(Mutex::new(Vec::new()));
    let mut world = World::new();
    let table_entity = world
        .spawn((Card::face_down(TextureId(0), TextureId(0)), CardZone::Table))
        .id();
    let reader_entity = world.spawn_empty().id();
    world.spawn((
        Card::face_down(TextureId(0), TextureId(0)),
        CardZone::Reader(reader_entity),
    ));
    let spy = SpyPhysicsBackend::new()
        .with_damping_log(damping_log.clone())
        .with_angular_velocity(table_entity, 0.0);
    world.insert_resource(PhysicsRes::new(Box::new(spy)));

    // Act
    run_system(&mut world);

    // Assert
    let calls = damping_log.lock().unwrap();
    assert_eq!(calls.len(), 1, "only the Table card should get damping");
    assert_eq!(calls[0].0, table_entity);
}

/// @doc: Hand cards skip damping—they're never physics-driven and don't have bodies
#[test]
fn when_card_in_hand_then_set_damping_not_called() {
    // Arrange
    let damping_log: DampingLog = Arc::new(Mutex::new(Vec::new()));
    let mut world = World::new();
    let table_entity = world
        .spawn((Card::face_down(TextureId(0), TextureId(0)), CardZone::Table))
        .id();
    world.spawn((
        Card::face_down(TextureId(0), TextureId(0)),
        CardZone::Hand(0),
    ));
    let spy = SpyPhysicsBackend::new()
        .with_damping_log(damping_log.clone())
        .with_angular_velocity(table_entity, 0.0);
    world.insert_resource(PhysicsRes::new(Box::new(spy)));

    // Act
    run_system(&mut world);

    // Assert
    let calls = damping_log.lock().unwrap();
    assert_eq!(calls.len(), 1, "only the Table card should get damping");
    assert_eq!(calls[0].0, table_entity);
}
