#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::*;
use card_game::booster::sampling::sample_signatures_from_space;
use card_game::card::identity::signature::CardSignature;
use card_game::card::reader::SignatureSpace;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

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

#[test]
fn when_sample_from_single_point_space_then_all_within_radius() {
    // Arrange
    let entity = Entity::from_raw(0);
    let center = CardSignature::new([0.3, -0.2, 0.5, 0.1, -0.4, 0.6, -0.1, 0.2]);
    let space = SignatureSpace::from_single(center, 0.2, entity);
    let mut rng = ChaCha8Rng::seed_from_u64(42);

    // Act
    let samples = sample_signatures_from_space(&space, 10, &mut rng);

    // Assert
    assert_eq!(samples.len(), 10);
    for (i, sample) in samples.iter().enumerate() {
        assert!(
            space.contains(sample),
            "sample {i} at {:?} is not within the single-point space",
            sample.axes()
        );
    }
}

#[test]
fn when_sample_from_polyline_space_then_all_within_radius() {
    // Arrange
    let entity_a = Entity::from_raw(0);
    let entity_b = Entity::from_raw(1);
    let sig_a = CardSignature::new([0.2, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    let sig_b = CardSignature::new([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.2]);
    let space_a = SignatureSpace::from_single(sig_a, 0.2, entity_a);
    let space_b = SignatureSpace::from_single(sig_b, 0.2, entity_b);
    let combined = SignatureSpace::combine(&space_a, &space_b);
    let mut rng = ChaCha8Rng::seed_from_u64(99);

    // Act
    let samples = sample_signatures_from_space(&combined, 10, &mut rng);

    // Assert
    assert_eq!(samples.len(), 10);
    for (i, sample) in samples.iter().enumerate() {
        assert!(
            combined.contains(sample),
            "sample {i} at {:?} is not within the combined polyline space",
            sample.axes()
        );
    }
}

#[test]
fn when_sample_then_all_axes_clamped() {
    // Arrange — center near the edge of [-1, 1] so offset can push beyond
    let entity = Entity::from_raw(0);
    let center = CardSignature::new([0.95, 0.95, 0.95, 0.95, 0.95, 0.95, 0.95, 0.95]);
    let space = SignatureSpace::from_single(center, 0.3, entity);
    let mut rng = ChaCha8Rng::seed_from_u64(7);

    // Act
    let samples = sample_signatures_from_space(&space, 20, &mut rng);

    // Assert
    assert_eq!(samples.len(), 20);
    for (i, sample) in samples.iter().enumerate() {
        for (j, &axis) in sample.axes().iter().enumerate() {
            assert!(
                (-1.0..=1.0).contains(&axis),
                "sample {i} axis {j} = {axis} is out of [-1, 1] range"
            );
        }
    }
}
