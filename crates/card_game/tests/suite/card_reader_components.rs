#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::*;
use glam::Vec2;

use card_game::card::reader::{CardReader, READER_CARD_SCALE, card_overlaps_reader};

/// @doc: A card whose center lies inside the reader's AABB must be detected as
/// overlapping. This is the primary positive case — without it, no card could
/// ever be inserted into a reader.
#[test]
fn when_card_inside_reader_then_overlaps_returns_true() {
    // Arrange
    let reader_pos = Vec2::new(100.0, 100.0);
    let reader_half = Vec2::new(40.0, 60.0);
    let card_pos = Vec2::new(120.0, 80.0);

    // Act
    let result = card_overlaps_reader(card_pos, reader_pos, reader_half);

    // Assert
    assert!(
        result,
        "card at {card_pos} should overlap reader at {reader_pos} +/- {reader_half}"
    );
}

/// @doc: A card whose center is outside the reader's AABB must not be detected as
/// overlapping. A false positive would cause cards far from any reader to teleport
/// into it when dropped.
#[test]
fn when_card_outside_reader_then_overlaps_returns_false() {
    // Arrange
    let reader_pos = Vec2::new(100.0, 100.0);
    let reader_half = Vec2::new(40.0, 60.0);
    let card_pos = Vec2::new(200.0, 100.0);

    // Act
    let result = card_overlaps_reader(card_pos, reader_pos, reader_half);

    // Assert
    assert!(
        !result,
        "card at {card_pos} should NOT overlap reader at {reader_pos} +/- {reader_half}"
    );
}

/// @doc: A card exactly on the reader's left boundary must be accepted. The
/// inclusive check (`<=`) ensures a card touching the left edge is valid,
/// preventing a pixel-wide dead zone on the boundary.
#[test]
fn when_card_on_left_boundary_then_overlaps_returns_true() {
    // Arrange
    let reader_pos = Vec2::new(100.0, 100.0);
    let reader_half = Vec2::new(40.0, 60.0);
    let card_pos = Vec2::new(60.0, 100.0);

    // Act
    let result = card_overlaps_reader(card_pos, reader_pos, reader_half);

    // Assert
    assert!(result, "card exactly on left boundary should be accepted");
}

/// @doc: A card exactly on the reader's right boundary must be accepted. The
/// inclusive check (`<=`) ensures a card touching the right edge is valid,
/// consistent with the left boundary treatment.
#[test]
fn when_card_on_right_boundary_then_overlaps_returns_true() {
    // Arrange
    let reader_pos = Vec2::new(100.0, 100.0);
    let reader_half = Vec2::new(40.0, 60.0);
    let card_pos = Vec2::new(140.0, 100.0);

    // Act
    let result = card_overlaps_reader(card_pos, reader_pos, reader_half);

    // Assert
    assert!(result, "card exactly on right boundary should be accepted");
}

/// @doc: A card positioned just one unit beyond the reader's boundary must be
/// rejected. This tests that the inclusive boundary does not leak into the
/// exclusive zone, which would cause cards visibly outside the reader to still
/// be accepted.
#[test]
fn when_card_just_beyond_boundary_then_overlaps_returns_false() {
    // Arrange
    let reader_pos = Vec2::new(100.0, 100.0);
    let reader_half = Vec2::new(40.0, 60.0);
    let card_pos = Vec2::new(141.0, 100.0);

    // Act
    let result = card_overlaps_reader(card_pos, reader_pos, reader_half);

    // Assert
    assert!(
        !result,
        "card one unit beyond right boundary should NOT overlap"
    );
}

/// @doc: When the reader's half extents are zero, only a card positioned
/// exactly at the reader's center is accepted. This edge case covers readers
/// with zero-volume frames (e.g., during initialization or if a dimension is
/// set to zero as a sentinel).
#[test]
fn when_reader_half_extents_zero_then_only_exact_match() {
    // Arrange
    let reader_pos = Vec2::new(100.0, 100.0);
    let reader_half = Vec2::ZERO;
    let exact_card_pos = Vec2::new(100.0, 100.0);
    let offset_card_pos = Vec2::new(100.1, 100.0);

    // Act
    let exact_result = card_overlaps_reader(exact_card_pos, reader_pos, reader_half);
    let offset_result = card_overlaps_reader(offset_card_pos, reader_pos, reader_half);

    // Assert
    assert!(
        exact_result,
        "card at exact reader position should overlap zero-extent reader"
    );
    assert!(
        !offset_result,
        "card 0.1 units away should NOT overlap zero-extent reader"
    );
}

/// @doc: The `READER_CARD_SCALE` constant must be positive so that cards
/// inserted into a reader are scaled down (not up or inverted). A negative or
/// zero value would cause visual artifacts or invisible cards.
#[test]
fn when_reader_card_scale_constant_is_positive() {
    // Arrange & Act & Assert
    assert!(
        READER_CARD_SCALE > 0.0,
        "READER_CARD_SCALE = {READER_CARD_SCALE} should be positive"
    );
    assert!(
        READER_CARD_SCALE < 1.0,
        "READER_CARD_SCALE = {READER_CARD_SCALE} should be less than 1.0 for downsizing"
    );
}

/// @doc: A `CardReader` component stores the loaded card entity, half extents,
/// and jack entity. All three fields must be accessible after construction,
/// verifying the struct layout matches expectations.
#[test]
fn when_card_reader_created_then_fields_accessible() {
    // Arrange
    let jack = Entity::from_raw(42);
    let loaded = Entity::from_raw(7);
    let half = Vec2::new(40.0, 60.0);

    // Act
    let reader = CardReader {
        loaded: Some(loaded),
        half_extents: half,
        jack_entity: jack,
    };

    // Assert
    assert_eq!(
        reader.loaded,
        Some(loaded),
        "CardReader.loaded should store the loaded card entity"
    );
    assert_eq!(
        reader.half_extents, half,
        "CardReader.half_extents should be accessible"
    );
    assert_eq!(
        reader.jack_entity, jack,
        "CardReader.jack_entity should be accessible"
    );
    assert!(
        reader.loaded.is_some(),
        "CardReader.loaded should be Some when a card is loaded"
    );
}
