//! Tests for compact generated shape data hydration functions.
//!
//! Verifies that `hydrate_shapes`, `hydrate_shapes_compact`, and
//! `hydrate_shapes_compact_indexed` correctly decode binary-encoded shape
//! data into `Shape` values.

#![allow(clippy::unwrap_used)]

use card_game::card::art::hydrate::{
    hydrate_shapes, hydrate_shapes_compact, hydrate_shapes_compact_indexed,
};
use engine_core::color::Color;
use engine_render::shape::{PathCommand, ShapeVariant};
use glam::Vec2;

// ---------------------------------------------------------------------------
// hydrate_shapes (f32 format)
// ---------------------------------------------------------------------------

/// @doc: returns empty vec for empty f32 data slice
#[test]
fn when_empty_data_then_empty_shapes() {
    // Arrange
    let data: &[f32] = &[];

    // Act
    let shapes = hydrate_shapes(data);

    // Assert
    assert!(shapes.is_empty(), "expected no shapes from empty data");
}

/// @doc: decodes a single shape with MoveTo, LineTo, and Close
#[test]
fn when_single_shape_with_move_line_close_then_one_shape() {
    // Arrange
    let data: &[f32] = &[
        4.0,             // tag: new shape
        1.0, 0.0, 0.0, 1.0, // RGBA red
        0.0, 10.0, 20.0, // MoveTo(10, 20)
        1.0, 100.0, 200.0, // LineTo(100, 200)
        3.0,             // Close
    ];

    // Act
    let shapes = hydrate_shapes(data);

    // Assert
    assert_eq!(shapes.len(), 1, "expected one shape");
    assert_eq!(
        shapes[0].color,
        Color::new(1.0, 0.0, 0.0, 1.0),
        "shape color mismatch"
    );
    let commands = match &shapes[0].variant {
        ShapeVariant::Path { commands } => commands,
        _ => panic!("expected ShapeVariant::Path"),
    };
    assert_eq!(commands.len(), 3, "expected 3 commands");
    assert_eq!(commands[0], PathCommand::MoveTo(Vec2::new(10.0, 20.0)));
    assert_eq!(commands[1], PathCommand::LineTo(Vec2::new(100.0, 200.0)));
    assert_eq!(commands[2], PathCommand::Close);
}

/// @doc: decodes a cubic Bézier command from f32 data
#[test]
fn when_single_shape_with_cubic_then_cubic_command() {
    // Arrange
    let data: &[f32] = &[
        4.0,             // tag: new shape
        0.0, 1.0, 0.0, 1.0, // RGBA green
        0.0, 0.0, 0.0,  // MoveTo(0, 0)
        2.0, 10.0, 0.0, 20.0, 0.0, 30.0, 0.0, // CubicTo
        3.0,             // Close
    ];

    // Act
    let shapes = hydrate_shapes(data);

    // Assert
    assert_eq!(shapes.len(), 1, "expected one shape");
    let commands = match &shapes[0].variant {
        ShapeVariant::Path { commands } => commands,
        _ => panic!("expected ShapeVariant::Path"),
    };
    assert_eq!(commands.len(), 3, "expected 3 commands");
    assert_eq!(
        commands[1],
        PathCommand::CubicTo {
            control1: Vec2::new(10.0, 0.0),
            control2: Vec2::new(20.0, 0.0),
            to: Vec2::new(30.0, 0.0),
        },
        "cubic Bézier command mismatch"
    );
}

/// @doc: decodes multiple shapes from a single data stream
#[test]
fn when_multiple_shapes_then_all_returned() {
    // Arrange
    let data: &[f32] = &[
        // Shape 1: red rectangle loop
        4.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 100.0, 0.0, 1.0, 100.0,
        100.0, 1.0, 0.0, 100.0, 3.0,
        // Shape 2: blue triangle
        4.0, 0.0, 0.0, 1.0, 1.0, 0.0, 50.0, 0.0, 1.0, 50.0, 100.0, 1.0, 0.0,
        0.0, 3.0,
    ];

    // Act
    let shapes = hydrate_shapes(data);

    // Assert
    assert_eq!(shapes.len(), 2, "expected two shapes");
    assert_eq!(
        shapes[0].color,
        Color::new(1.0, 0.0, 0.0, 1.0),
        "first shape color"
    );
    assert_eq!(
        shapes[1].color,
        Color::new(0.0, 0.0, 1.0, 1.0),
        "second shape color"
    );
}

/// @doc: unknown command tags cause inner loop to break, keeping earlier commands
#[test]
fn when_unknown_command_then_commands_before_unknown_kept() {
    // Arrange
    let data: &[f32] = &[
        4.0, 1.0, 0.0, 0.0, 1.0, // red shape
        0.0, 5.0, 5.0,           // MoveTo(5, 5)
        99.0,                     // unknown command tag
        3.0,                      // Close is never reached
    ];

    // Act
    let shapes = hydrate_shapes(data);

    // Assert
    assert_eq!(shapes.len(), 1, "expected one shape");
    let commands = match &shapes[0].variant {
        ShapeVariant::Path { commands } => commands,
        _ => panic!("expected ShapeVariant::Path"),
    };
    assert_eq!(commands.len(), 1, "expected only MoveTo after unknown break");
    assert_eq!(commands[0], PathCommand::MoveTo(Vec2::new(5.0, 5.0)));
}

/// @doc: non-four tags in the outer loop byte stream are silently skipped
#[test]
fn when_non_four_tag_then_skipped() {
    // Arrange
    let data: &[f32] = &[
        99.0,                // not tag 4 → skip
        4.0, 1.0, 0.0, 0.0, 1.0, // valid red shape
        0.0, 0.0, 0.0, 3.0,
    ];

    // Act
    let shapes = hydrate_shapes(data);

    // Assert
    assert_eq!(shapes.len(), 1, "expected one shape after skipping non-tag byte");
}

// ---------------------------------------------------------------------------
// hydrate_shapes_compact (separate colors + i16 relative data)
// ---------------------------------------------------------------------------

/// @doc: returns empty vec for empty compact data
#[test]
fn when_compact_empty_data_then_empty_shapes() {
    // Arrange
    let colors: &[u8] = &[];
    let data: &[i16] = &[];

    // Act
    let shapes = hydrate_shapes_compact(colors, data);

    // Assert
    assert!(shapes.is_empty(), "expected no shapes from empty compact data");
}

/// @doc: decodes a single compact shape with color and delta-encoded line commands
#[test]
fn when_compact_single_shape_then_correct_color_and_commands() {
    // Arrange
    let colors: &[u8] = &[255, 128, 64]; // R=255, G=128, B=64
    let data: &[i16] = &[
        4,      // tag: new shape
        0, 500, // start: (0.0, 5.0) after /100
        1, 100, 0, // LineTo delta(100, 0) → (1.0, 5.0)
        1, 0, 200, // LineTo delta(0, 200) → (1.0, 7.0)
        3,      // unknown → break inner
    ];

    // Act
    let shapes = hydrate_shapes_compact(colors, data);

    // Assert
    assert_eq!(shapes.len(), 1, "expected one shape");
    assert_eq!(
        shapes[0].color,
        Color::from_u8(255, 128, 64, u8::MAX),
        "compact color mismatch"
    );
    let commands = match &shapes[0].variant {
        ShapeVariant::Path { commands } => commands,
        _ => panic!("expected ShapeVariant::Path"),
    };
    assert_eq!(commands.len(), 4, "expected MoveTo + 2 LineTo + Close");
    assert_eq!(commands[0], PathCommand::MoveTo(Vec2::new(0.0, 5.0)));
    assert_eq!(commands[1], PathCommand::LineTo(Vec2::new(1.0, 5.0)));
    assert_eq!(commands[2], PathCommand::LineTo(Vec2::new(1.0, 7.0)));
    assert_eq!(commands[3], PathCommand::Close);
}

/// @doc: decodes a cubic Bézier command from compact delta-encoded data
#[test]
fn when_compact_shape_with_cubic_then_cubic_commands() {
    // Arrange
    let colors: &[u8] = &[0, 128, 255];
    let data: &[i16] = &[
        4,    // tag: new shape
        0, 0, // start: (0.0, 0.0)
        2, 100, 0, 200, 0, 300, 0, // CubicTo relative deltas
        3,    // unknown → break inner
    ];

    // Act
    let shapes = hydrate_shapes_compact(colors, data);

    // Assert
    assert_eq!(shapes.len(), 1, "expected one shape");
    let commands = match &shapes[0].variant {
        ShapeVariant::Path { commands } => commands,
        _ => panic!("expected ShapeVariant::Path"),
    };
    assert_eq!(commands.len(), 3, "expected MoveTo + CubicTo + Close");
    assert_eq!(
        commands[1],
        PathCommand::CubicTo {
            control1: Vec2::new(1.0, 0.0),
            control2: Vec2::new(2.0, 0.0),
            to: Vec2::new(3.0, 0.0),
        },
        "compact cubic Bézier mismatch"
    );
}

/// @doc: decodes multiple compact shapes with per-shape colors
#[test]
fn when_compact_multiple_shapes_then_all_returned() {
    // Arrange
    let colors: &[u8] = &[255, 0, 0, 0, 255, 0]; // red then green
    let data: &[i16] = &[
        4, 0, 0, 1, 100, 0, 3, // shape 0: LineTo(1.0, 0.0)
        4, 200, 0, 1, 0, 100, 3, // shape 1: LineTo(2.0, 1.0)
    ];

    // Act
    let shapes = hydrate_shapes_compact(colors, data);

    // Assert
    assert_eq!(shapes.len(), 2, "expected two shapes");
    assert_eq!(
        shapes[0].color,
        Color::from_u8(255, 0, 0, u8::MAX),
        "first compact color"
    );
    assert_eq!(
        shapes[1].color,
        Color::from_u8(0, 255, 0, u8::MAX),
        "second compact color"
    );
}

/// @doc: breaks outer loop when color entries are exhausted before data ends
#[test]
fn when_compact_color_exhausted_then_early_break() {
    // Arrange — only one color entry, but data has two shapes
    let colors: &[u8] = &[255, 0, 0];
    let data: &[i16] = &[
        4, 0, 0, 1, 100, 0, 3, // shape 0: valid colors
        4, 0, 0, 1, 100, 0, 3, // shape 1: color_index + 2 >= colors.len()
    ];

    // Act
    let shapes = hydrate_shapes_compact(colors, data);

    // Assert
    assert_eq!(shapes.len(), 1, "expected only first shape");
}

/// @doc: breaks outer loop when data ends after tag but before start coordinates
#[test]
fn when_compact_data_ends_mid_start_then_early_break() {
    // Arrange
    let colors: &[u8] = &[255, 0, 0];
    let data: &[i16] = &[4]; // tag 4 but no start coords

    // Act
    let shapes = hydrate_shapes_compact(colors, data);

    // Assert
    assert!(shapes.is_empty(), "expected no shapes from truncated data");
}

/// @doc: breaks inner LineTo parsing when fewer than 2 i16 values remain
#[test]
fn when_compact_line_to_truncated_then_breaks_inner() {
    // Arrange
    let colors: &[u8] = &[255, 0, 0];
    let data: &[i16] = &[
        4, 0, 0, // valid start
        1, 100, // LineTo tag with only one delta value
    ];

    // Act
    let shapes = hydrate_shapes_compact(colors, data);

    // Assert — shape still pushed with MoveTo and Close
    assert_eq!(shapes.len(), 1, "expected one shape with partial commands");
    let commands = match &shapes[0].variant {
        ShapeVariant::Path { commands } => commands,
        _ => panic!("expected ShapeVariant::Path"),
    };
    assert_eq!(commands.len(), 2, "expected MoveTo + Close");
    assert_eq!(commands[0], PathCommand::MoveTo(Vec2::new(0.0, 0.0)));
    assert_eq!(commands[1], PathCommand::Close);
}

/// @doc: breaks inner CubicTo parsing when fewer than 6 i16 values remain
#[test]
fn when_compact_cubic_to_truncated_then_breaks_inner() {
    // Arrange
    let colors: &[u8] = &[255, 0, 0];
    let data: &[i16] = &[
        4, 0, 0, // valid start
        2, 10, 20, // CubicTo tag with only 2 of 6 delta values
    ];

    // Act
    let shapes = hydrate_shapes_compact(colors, data);

    // Assert
    assert_eq!(shapes.len(), 1, "expected one shape with partial commands");
    let commands = match &shapes[0].variant {
        ShapeVariant::Path { commands } => commands,
        _ => panic!("expected ShapeVariant::Path"),
    };
    assert_eq!(commands.len(), 2, "expected MoveTo + Close");
    assert_eq!(commands[0], PathCommand::MoveTo(Vec2::new(0.0, 0.0)));
    assert_eq!(commands[1], PathCommand::Close);
}

// ---------------------------------------------------------------------------
// hydrate_shapes_compact_indexed (palette-indexed colors + i16 relative data)
// ---------------------------------------------------------------------------

/// @doc: returns empty vec for empty indexed data
#[test]
fn when_indexed_empty_data_then_empty_shapes() {
    // Arrange
    let color_indexes: &[u8] = &[];
    let data: &[i16] = &[];

    // Act
    let shapes = hydrate_shapes_compact_indexed(color_indexes, data);

    // Assert
    assert!(shapes.is_empty(), "expected no shapes from empty indexed data");
}

/// @doc: reads colors from SHARED_PALETTE via the given index
#[test]
fn when_indexed_single_shape_then_palette_color_applied() {
    // Arrange
    // SHARED_PALETTE[0..3] = [227, 12, 33]
    let color_indexes: &[u8] = &[0];
    let expected_color = Color::from_u8(227, 12, 33, u8::MAX);
    let data: &[i16] = &[
        4, 0, 0, // start (0.0, 0.0)
        1, 100, 0, // LineTo(1.0, 0.0)
        3,        // break inner
    ];

    // Act
    let shapes = hydrate_shapes_compact_indexed(color_indexes, data);

    // Assert
    assert_eq!(shapes.len(), 1, "expected one shape");
    assert_eq!(
        shapes[0].color, expected_color,
        "palette-indexed color mismatch"
    );
}

/// @doc: breaks outer loop when color_indexes are exhausted
#[test]
fn when_indexed_color_index_exhausted_then_early_break() {
    // Arrange — one color index but data for two shapes
    let color_indexes: &[u8] = &[0];
    let data: &[i16] = &[
        4, 0, 0, 1, 100, 0, 3, // shape 0: valid
        4, 0, 0, 1, 100, 0, 3, // shape 1: no color_index left
    ];

    // Act
    let shapes = hydrate_shapes_compact_indexed(color_indexes, data);

    // Assert
    assert_eq!(shapes.len(), 1, "expected only first shape");
}

/// @doc: breaks outer loop when start coordinates are truncated
#[test]
fn when_indexed_data_truncated_mid_start_then_early_break() {
    // Arrange
    let color_indexes: &[u8] = &[0];
    let data: &[i16] = &[4]; // tag but no start coords

    // Act
    let shapes = hydrate_shapes_compact_indexed(color_indexes, data);

    // Assert
    assert!(shapes.is_empty(), "expected no shapes from truncated indexed data");
}

/// @doc: decodes cubic commands from indexed compact data
#[test]
fn when_indexed_shape_with_cubic_then_cubic_commands() {
    // Arrange
    let color_indexes: &[u8] = &[1]; // palette index 1 → [10, 76, 132]
    let data: &[i16] = &[
        4, 0, 0, // start
        2, 100, 0, 200, 0, 300, 0, // CubicTo
        3,        // break
    ];

    // Act
    let shapes = hydrate_shapes_compact_indexed(color_indexes, data);

    // Assert
    assert_eq!(shapes.len(), 1, "expected one shape");
    let commands = match &shapes[0].variant {
        ShapeVariant::Path { commands } => commands,
        _ => panic!("expected ShapeVariant::Path"),
    };
    assert_eq!(commands.len(), 3, "expected MoveTo + CubicTo + Close");
    assert_eq!(
        commands[1],
        PathCommand::CubicTo {
            control1: Vec2::new(1.0, 0.0),
            control2: Vec2::new(2.0, 0.0),
            to: Vec2::new(3.0, 0.0),
        },
        "indexed cubic Bézier mismatch"
    );
}
