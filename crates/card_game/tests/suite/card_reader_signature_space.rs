#![allow(clippy::unwrap_used, clippy::assertions_on_constants)]

use bevy_ecs::prelude::Entity;

use card_game::card::identity::signature::CardSignature;
use card_game::card::reader::{SIGNATURE_SPACE_RADIUS, SignatureSpace, signature_radius};

#[test]
/// @doc: `signature_radius` returns the minimum radius when all intensities are zero.
fn when_signature_radius_with_zero_intensity_then_returns_min_radius() {
    // Arrange
    let sig = CardSignature::default();

    // Act
    let r = signature_radius(&sig);

    // Assert
    assert!((r - 0.15).abs() < f32::EPSILON, "expected 0.15, got {r}");
}

#[test]
/// @doc: `from_single` creates a `SignatureSpace` with exactly one control point.
fn when_from_single_then_has_single_control_point() {
    // Arrange
    let center = CardSignature::new([0.3, -0.5, 0.1, 0.7, -0.2, 0.4, -0.6, 0.8]);
    let entity = Entity::from_raw(1);

    // Act
    let space = SignatureSpace::from_single(center, 0.2, entity);

    // Assert
    assert_eq!(space.control_points.len(), 1);
    assert_eq!(space.control_points[0], center);
}

#[test]
/// @doc: `from_single` stores the source entity in `source_cards`.
fn when_from_single_then_source_cards_contains_entity() {
    // Arrange
    let center = CardSignature::default();
    let entity = Entity::from_raw(42);

    // Act
    let space = SignatureSpace::from_single(center, 0.2, entity);

    // Assert
    assert!(
        space.source_cards.contains(&entity),
        "expected source_cards to contain entity {entity:?}, got {:?}",
        space.source_cards
    );
}

#[test]
/// @doc: `from_single` assigns a positive volume.
fn when_from_single_then_volume_is_positive() {
    // Arrange
    let center = CardSignature::default();
    let entity = Entity::from_raw(7);

    // Act
    let space = SignatureSpace::from_single(center, 0.2, entity);

    // Assert
    assert!(
        space.volume > 0.0,
        "expected positive volume, got {}",
        space.volume
    );
}

#[test]
/// @doc: combine unions the control points of two distinct signals.
fn when_combine_two_signals_then_control_points_union() {
    // Arrange
    let center_a = CardSignature::new([0.3, -0.5, 0.1, 0.7, -0.2, 0.4, -0.6, 0.8]);
    let center_b = CardSignature::new([-0.1, 0.2, -0.3, 0.4, -0.5, 0.6, -0.7, 0.9]);
    let a = SignatureSpace::from_single(center_a, 0.2, Entity::from_raw(0));
    let b = SignatureSpace::from_single(center_b, 0.2, Entity::from_raw(1));

    // Act
    let combined = SignatureSpace::combine(&a, &b);

    // Assert
    assert_eq!(combined.control_points.len(), 2);
    assert!(combined.control_points.contains(&center_a));
    assert!(combined.control_points.contains(&center_b));
}

#[test]
/// @doc: combine deduplicates identical source entities.
fn when_combine_two_signals_then_source_cards_union_no_duplicates() {
    // Arrange — both signals share the same source entity
    let entity = Entity::from_raw(0);
    let center = CardSignature::new([0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8]);
    let a = SignatureSpace::from_single(center, 0.2, entity);
    let b = SignatureSpace::from_single(center, 0.2, entity);

    // Act
    let combined = SignatureSpace::combine(&a, &b);

    // Assert
    assert_eq!(
        combined.source_cards.len(),
        1,
        "expected 1 source card, got {}: {:?}",
        combined.source_cards.len(),
        combined.source_cards
    );
    assert!(combined.source_cards.contains(&entity));
}

#[test]
/// @doc: combine adds the volumes of the two signals.
fn when_combine_two_signals_then_volume_is_sum() {
    // Arrange
    let center_a = CardSignature::new([0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    let center_b = CardSignature::new([0.0, 0.2, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    let a = SignatureSpace::from_single(center_a, 0.2, Entity::from_raw(0));
    let b = SignatureSpace::from_single(center_b, 0.2, Entity::from_raw(1));

    // Act
    let combined = SignatureSpace::combine(&a, &b);

    // Assert
    let expected_volume = a.volume + b.volume;
    assert!(
        (combined.volume - expected_volume).abs() < 1e-12,
        "expected volume {expected_volume}, got {}",
        combined.volume
    );
}

#[test]
/// @doc: contains returns true for a point at the control point center when radius is large enough.
fn when_contains_same_point_then_returns_true() {
    // Arrange
    let center = CardSignature::new([0.2, -0.3, 0.1, 0.5, -0.4, 0.6, -0.2, 0.3]);
    let space = SignatureSpace::from_single(center, 0.5, Entity::from_raw(0));

    // Act
    let result = space.contains(&center);

    // Assert
    assert!(result, "expected contains(self center) to be true");
}

#[test]
/// @doc: contains returns false for a point far from the single control point.
fn when_contains_distant_point_then_returns_false() {
    // Arrange
    let center = CardSignature::new([0.0; 8]);
    let far = CardSignature::new([1.0; 8]); // distance = sqrt(8) ≈ 2.83
    let space = SignatureSpace::from_single(center, 0.2, Entity::from_raw(0));

    // Act
    let result = space.contains(&far);

    // Assert
    assert!(!result, "expected contains(far point) to be false");
}

#[test]
/// @doc: contains returns false when there are no control points.
fn when_empty_control_points_then_contains_returns_false() {
    // Arrange
    let empty = SignatureSpace {
        control_points: vec![],
        radius: 0.2,
        volume: 0.0,
        source_cards: vec![],
    };
    let point = CardSignature::new([0.1, -0.2, 0.3, -0.4, 0.5, -0.6, 0.7, -0.8]);

    // Act
    let result = empty.contains(&point);

    // Assert
    assert!(
        !result,
        "expected contains to be false with no control points"
    );
}

#[test]
/// @doc: `SIGNATURE_SPACE_RADIUS` is a positive constant.
fn when_constant_radius_is_positive() {
    assert!(SIGNATURE_SPACE_RADIUS > 0.0);
}
