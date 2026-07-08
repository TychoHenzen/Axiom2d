#![allow(clippy::unwrap_used, clippy::assertions_on_constants)]

use card_game::stash::constants::{
    BACKGROUND_COLOR, GRID_MARGIN, SLOT_COLOR, SLOT_GAP, SLOT_HEIGHT, SLOT_HIGHLIGHT_COLOR,
    SLOT_STRIDE_H, SLOT_STRIDE_W, SLOT_WIDTH,
};
use engine_core::color::Color;

fn color_in_0_1_range(c: &Color) -> bool {
    c.r >= 0.0
        && c.r <= 1.0
        && c.g >= 0.0
        && c.g <= 1.0
        && c.b >= 0.0
        && c.b <= 1.0
        && c.a >= 0.0
        && c.a <= 1.0
}

fn color_is_opaque(c: &Color) -> bool {
    (c.a - 1.0).abs() < f32::EPSILON
}

/// @doc: verifies `SLOT_WIDTH` is a positive pixel value
#[test]
fn when_slot_width_then_positive() {
    // Arrange & Act & Assert
    assert!(
        SLOT_WIDTH > 0.0,
        "SLOT_WIDTH must be positive, got {SLOT_WIDTH}"
    );
}

/// @doc: verifies `SLOT_HEIGHT` is a positive pixel value
#[test]
fn when_slot_height_then_positive() {
    // Arrange & Act & Assert
    assert!(
        SLOT_HEIGHT > 0.0,
        "SLOT_HEIGHT must be positive, got {SLOT_HEIGHT}"
    );
}

/// @doc: verifies `SLOT_GAP` is a non-negative spacing value
#[test]
fn when_slot_gap_then_non_negative() {
    // Arrange & Act & Assert
    assert!(
        SLOT_GAP >= 0.0,
        "SLOT_GAP must be non-negative, got {SLOT_GAP}"
    );
}

/// @doc: verifies `GRID_MARGIN` is a positive pixel value
#[test]
fn when_grid_margin_then_positive() {
    // Arrange & Act & Assert
    assert!(
        GRID_MARGIN > 0.0,
        "GRID_MARGIN must be positive, got {GRID_MARGIN}"
    );
}

/// @doc: verifies `SLOT_STRIDE_W` equals `SLOT_WIDTH` plus `SLOT_GAP`
#[test]
fn when_slot_stride_w_then_equals_slot_width_plus_gap() {
    // Arrange
    let expected = SLOT_WIDTH + SLOT_GAP;

    // Act & Assert
    assert!(
        (SLOT_STRIDE_W - expected).abs() < f32::EPSILON,
        "SLOT_STRIDE_W ({SLOT_STRIDE_W}) must equal SLOT_WIDTH ({SLOT_WIDTH}) + SLOT_GAP ({SLOT_GAP}) = {expected}"
    );
}

/// @doc: verifies `SLOT_STRIDE_H` equals `SLOT_HEIGHT` plus `SLOT_GAP`
#[test]
fn when_slot_stride_h_then_equals_slot_height_plus_gap() {
    // Arrange
    let expected = SLOT_HEIGHT + SLOT_GAP;

    // Act & Assert
    assert!(
        (SLOT_STRIDE_H - expected).abs() < f32::EPSILON,
        "SLOT_STRIDE_H ({SLOT_STRIDE_H}) must equal SLOT_HEIGHT ({SLOT_HEIGHT}) + SLOT_GAP ({SLOT_GAP}) = {expected}"
    );
}

/// @doc: verifies slots are taller than wide (portrait card aspect ratio)
#[test]
fn when_slot_dimensions_then_width_less_than_height() {
    // Arrange & Act & Assert
    assert!(
        SLOT_WIDTH < SLOT_HEIGHT,
        "card slot should be portrait: SLOT_WIDTH ({SLOT_WIDTH}) < SLOT_HEIGHT ({SLOT_HEIGHT})"
    );
}

/// @doc: verifies strides exceed their corresponding slot dimensions when gap is positive
#[test]
fn when_gap_positive_then_stride_exceeds_slot_dimension() {
    // Arrange & Act & Assert
    assert!(
        SLOT_STRIDE_W > SLOT_WIDTH,
        "SLOT_STRIDE_W ({SLOT_STRIDE_W}) must exceed SLOT_WIDTH ({SLOT_WIDTH}) when SLOT_GAP > 0"
    );
    assert!(
        SLOT_STRIDE_H > SLOT_HEIGHT,
        "SLOT_STRIDE_H ({SLOT_STRIDE_H}) must exceed SLOT_HEIGHT ({SLOT_HEIGHT}) when SLOT_GAP > 0"
    );
}

/// @doc: verifies `SLOT_COLOR` has fully opaque alpha
#[test]
fn when_slot_color_then_alpha_opaque() {
    // Arrange & Act & Assert
    assert!(
        color_is_opaque(&SLOT_COLOR),
        "SLOT_COLOR.a must be 1.0 for full opacity, got {}",
        SLOT_COLOR.a
    );
}

/// @doc: verifies `SLOT_HIGHLIGHT_COLOR` has fully opaque alpha
#[test]
fn when_slot_highlight_color_then_alpha_opaque() {
    // Arrange & Act & Assert
    assert!(
        color_is_opaque(&SLOT_HIGHLIGHT_COLOR),
        "SLOT_HIGHLIGHT_COLOR.a must be 1.0 for full opacity, got {}",
        SLOT_HIGHLIGHT_COLOR.a
    );
}

/// @doc: verifies `BACKGROUND_COLOR` has fully opaque alpha
#[test]
fn when_background_color_then_alpha_opaque() {
    // Arrange & Act & Assert
    assert!(
        color_is_opaque(&BACKGROUND_COLOR),
        "BACKGROUND_COLOR.a must be 1.0 for full opacity, got {}",
        BACKGROUND_COLOR.a
    );
}

/// @doc: verifies all color channels are in the valid [0.0, 1.0] range
#[test]
fn when_color_channels_then_in_zero_to_one_range() {
    // Arrange
    let colors = [&SLOT_COLOR, &SLOT_HIGHLIGHT_COLOR, &BACKGROUND_COLOR];
    let names = ["SLOT_COLOR", "SLOT_HIGHLIGHT_COLOR", "BACKGROUND_COLOR"];

    // Act & Assert
    for (color, name) in colors.iter().zip(names.iter()) {
        assert!(
            color_in_0_1_range(color),
            "{name} channels must be in [0,1]: r={}, g={}, b={}, a={}",
            color.r,
            color.g,
            color.b,
            color.a
        );
    }
}

/// @doc: verifies highlight color is visually brighter than default slot color
#[test]
fn when_highlight_color_then_brighter_than_slot_color() {
    // Arrange & Act & Assert
    assert!(
        SLOT_HIGHLIGHT_COLOR.r > SLOT_COLOR.r
            && SLOT_HIGHLIGHT_COLOR.g > SLOT_COLOR.g
            && SLOT_HIGHLIGHT_COLOR.b > SLOT_COLOR.b,
        "SLOT_HIGHLIGHT_COLOR ({SLOT_HIGHLIGHT_COLOR:?}) must be brighter than SLOT_COLOR ({SLOT_COLOR:?})",
    );
}
