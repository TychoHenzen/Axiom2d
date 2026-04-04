#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::World;
use engine_core::prelude::EventBus;
use engine_input::prelude::{InputState, KeyCode};
use engine_physics::prelude::PhysicsCommand;

use card_game::card::component::Card;
use card_game::card::identity::base_type::{BaseCardTypeRegistry, populate_default_types};
use card_game::card::rendering::debug_spawn::{DebugSpawnRng, debug_spawn_system};

fn setup_world() -> World {
    let mut world = World::new();
    world.insert_resource(InputState::default());
    world.insert_resource(DebugSpawnRng::default());
    world.insert_resource(EventBus::<PhysicsCommand>::default());
    let mut registry = BaseCardTypeRegistry::default();
    populate_default_types(&mut registry);
    world.insert_resource(registry);
    world
}

#[test]
fn when_key1_pressed_then_one_card_spawned() {
    // Arrange
    let mut world = setup_world();
    world.resource_mut::<InputState>().press(KeyCode::Digit1);

    // Act
    debug_spawn_system(&mut world);

    // Assert
    let card_count = world.query::<&Card>().iter(&world).count();
    assert_eq!(card_count, 1);
}

#[test]
fn when_key2_pressed_then_ten_cards_spawned() {
    // Arrange
    let mut world = setup_world();
    world.resource_mut::<InputState>().press(KeyCode::Digit2);

    // Act
    debug_spawn_system(&mut world);

    // Assert
    let card_count = world.query::<&Card>().iter(&world).count();
    assert_eq!(card_count, 10);
}

#[test]
fn when_key3_pressed_then_hundred_cards_spawned() {
    // Arrange
    let mut world = setup_world();
    world.resource_mut::<InputState>().press(KeyCode::Digit3);

    // Act
    debug_spawn_system(&mut world);

    // Assert
    let card_count = world.query::<&Card>().iter(&world).count();
    assert_eq!(card_count, 100);
}

#[test]
fn when_no_key_pressed_then_no_cards_spawned() {
    // Arrange
    let mut world = setup_world();

    // Act
    debug_spawn_system(&mut world);

    // Assert
    let card_count = world.query::<&Card>().iter(&world).count();
    assert_eq!(card_count, 0);
}

/// @doc: Debug-spawned cards must each receive a unique signature from the
/// seeded RNG. Duplicate signatures would produce identical-looking cards
/// with the same name, stats, and art — useless for testing visual variety.
#[test]
fn when_cards_spawned_then_each_has_unique_signature() {
    // Arrange
    let mut world = setup_world();
    world.resource_mut::<InputState>().press(KeyCode::Digit2);

    // Act
    debug_spawn_system(&mut world);

    // Assert
    let signatures: Vec<_> = world
        .query::<&Card>()
        .iter(&world)
        .map(|c| c.signature)
        .collect();
    for i in 0..signatures.len() {
        for j in (i + 1)..signatures.len() {
            assert_ne!(
                signatures[i], signatures[j],
                "cards {i} and {j} should have different signatures"
            );
        }
    }
}
