#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::World;
use engine_input::prelude::{InputState, KeyCode};
use engine_physics::prelude::PhysicsRes;

use card_game::card::component::CardZone;
use card_game::card::rendering::debug_sleep_indicator::{
    DebugSleepTint, debug_sleep_indicator_system,
};

use crate::test_helpers::{SpyPhysicsBackend, make_test_card};

fn setup_world() -> World {
    let mut world = World::new();
    world.insert_resource(InputState::default());
    world
}

/// @doc: When the debug sleep key is held, sleeping table cards must receive a
/// `DebugSleepTint` component so the renderer can visually distinguish them from
/// awake cards. Without this, developers have no way to verify that the sleep system
/// is working correctly during gameplay — they'd have to add logging or breakpoints
/// to confirm bodies are actually sleeping.
#[test]
fn when_debug_key_held_then_sleeping_cards_get_tint() {
    // Arrange
    let mut world = setup_world();
    let sleeping_entity = world.spawn((make_test_card(), CardZone::Table)).id();
    let awake_entity = world.spawn((make_test_card(), CardZone::Table)).id();
    let spy = SpyPhysicsBackend::new()
        .with_body_sleeping(sleeping_entity, true)
        .with_body_sleeping(awake_entity, false);
    world.insert_resource(PhysicsRes::new(Box::new(spy)));
    world.resource_mut::<InputState>().press(KeyCode::F9);

    // Act
    debug_sleep_indicator_system(&mut world);

    // Assert
    assert!(
        world
            .entity(sleeping_entity)
            .get::<DebugSleepTint>()
            .is_some(),
        "sleeping card should have DebugSleepTint"
    );
    assert!(
        world.entity(awake_entity).get::<DebugSleepTint>().is_none(),
        "awake card should NOT have DebugSleepTint"
    );
}

/// @doc: When the debug sleep key is not held, no tint components should be inserted.
/// The indicator is opt-in via key press — it must never activate on its own,
/// otherwise it would interfere with normal gameplay rendering.
#[test]
fn when_debug_key_not_held_then_no_tint_inserted() {
    // Arrange
    let mut world = setup_world();
    let entity = world.spawn((make_test_card(), CardZone::Table)).id();
    let spy = SpyPhysicsBackend::new().with_body_sleeping(entity, true);
    world.insert_resource(PhysicsRes::new(Box::new(spy)));
    // No key pressed

    // Act
    debug_sleep_indicator_system(&mut world);

    // Assert
    assert!(
        world.entity(entity).get::<DebugSleepTint>().is_none(),
        "no tint should be inserted when debug key is not held"
    );
}

/// @doc: When a card wakes up while the debug overlay is active, its stale
/// `DebugSleepTint` must be removed. Without this cleanup, a card that was sleeping
/// when the player pressed the debug key would keep its tint after being picked up,
/// making the overlay misleading and the awake card visually indistinguishable from
/// sleeping ones.
#[test]
fn when_tinted_card_wakes_up_then_tint_removed() {
    // Arrange
    let mut world = setup_world();
    let entity = world
        .spawn((make_test_card(), CardZone::Table, DebugSleepTint))
        .id();
    let spy = SpyPhysicsBackend::new().with_body_sleeping(entity, false);
    world.insert_resource(PhysicsRes::new(Box::new(spy)));
    world.resource_mut::<InputState>().press(KeyCode::F9);

    // Act
    debug_sleep_indicator_system(&mut world);

    // Assert
    assert!(
        world.entity(entity).get::<DebugSleepTint>().is_none(),
        "tint must be removed when card is no longer sleeping"
    );
}
