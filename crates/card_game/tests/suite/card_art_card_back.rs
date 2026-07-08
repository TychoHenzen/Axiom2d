//! Tests for the card back / `card_front2` shape data hydration.

#![allow(clippy::unwrap_used)]

use card_game::card::art::card_back::card_front2;
use engine_render::shape::ShapeVariant;

/// @doc: `card_front2` returns a non-empty vector of shapes from the embedded DATA
#[test]
fn when_card_front2_generated_then_shapes_non_empty() {
    // Arrange & Act
    let shapes = card_front2();

    // Assert
    assert!(
        !shapes.is_empty(),
        "card_front2 should produce at least one shape"
    );
}

/// @doc: `card_front2` returns the same shapes on repeated calls (deterministic)
#[test]
fn when_card_front2_called_twice_then_identical_output() {
    // Arrange & Act
    let shapes_a = card_front2();
    let shapes_b = card_front2();

    // Assert
    assert_eq!(
        shapes_a, shapes_b,
        "card_front2 should be deterministic (same DATA always)"
    );
}

/// @doc: each shape from `card_front2` has a Path variant with at least one command
#[test]
fn when_card_front2_generated_then_each_shape_is_path_with_commands() {
    // Arrange
    let shapes = card_front2();

    // Act & Assert
    for (i, shape) in shapes.iter().enumerate() {
        if let ShapeVariant::Path { commands } = &shape.variant {
            assert!(!commands.is_empty(), "shape {i} has empty path commands");
        }
    }
}

/// @doc: each shape from `card_front2` has RGBA color channels in valid range [0, 1]
#[test]
fn when_card_front2_generated_then_each_shape_has_valid_color() {
    // Arrange
    let shapes = card_front2();

    // Act & Assert
    for (i, shape) in shapes.iter().enumerate() {
        let c = &shape.color;
        assert!(
            (0.0..=1.0).contains(&c.r),
            "shape {i} red channel {:.3} out of [0, 1]",
            c.r
        );
        assert!(
            (0.0..=1.0).contains(&c.g),
            "shape {i} green channel {:.3} out of [0, 1]",
            c.g
        );
        assert!(
            (0.0..=1.0).contains(&c.b),
            "shape {i} blue channel {:.3} out of [0, 1]",
            c.b
        );
        assert!(
            (0.0..=1.0).contains(&c.a),
            "shape {i} alpha channel {:.3} out of [0, 1]",
            c.a
        );
    }
}
