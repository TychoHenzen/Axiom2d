#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::*;
use card_game::card::identity::signature::CardSignature;
use card_game::card::reader::SignatureSpace;

#[test]
fn when_from_single_then_source_cards_contains_entity() {
    // Arrange
    let mut world = World::new();
    let entity = world.spawn_empty().id();
    let sig = CardSignature::new([0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8]);

    // Act
    let space = SignatureSpace::from_single(sig, 0.2, entity);

    // Assert
    assert_eq!(space.source_cards, vec![entity]);
}

#[test]
fn when_combine_then_source_cards_merged() {
    // Arrange
    let mut world = World::new();
    let entity_a = world.spawn_empty().id();
    let entity_b = world.spawn_empty().id();
    let sig_a = CardSignature::new([0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    let sig_b = CardSignature::new([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.1]);
    let space_a = SignatureSpace::from_single(sig_a, 0.2, entity_a);
    let space_b = SignatureSpace::from_single(sig_b, 0.2, entity_b);

    // Act
    let combined = SignatureSpace::combine(&space_a, &space_b);

    // Assert
    assert!(
        combined.source_cards.contains(&entity_a),
        "combined signal must contain entity_a"
    );
    assert!(
        combined.source_cards.contains(&entity_b),
        "combined signal must contain entity_b"
    );
    assert_eq!(
        combined.source_cards.len(),
        2,
        "combined signal must have exactly 2 source cards"
    );
}
