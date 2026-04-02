#![allow(clippy::unwrap_used)]

use engine_core::types::{Pixels, Seconds, TextureId};

#[test]
fn when_newtypes_serialized_to_ron_then_deserialize_to_equal_value() {
    // Arrange
    let pixels = Pixels(123.456);
    let seconds = Seconds(0.016);
    let texture_id = TextureId(42);

    // Act
    let pixels_ron = ron::to_string(&pixels).unwrap();
    let seconds_ron = ron::to_string(&seconds).unwrap();
    let texture_id_ron = ron::to_string(&texture_id).unwrap();

    let pixels_back: Pixels = ron::from_str(&pixels_ron).unwrap();
    let seconds_back: Seconds = ron::from_str(&seconds_ron).unwrap();
    let texture_id_back: TextureId = ron::from_str(&texture_id_ron).unwrap();

    // Assert
    assert_eq!(pixels, pixels_back);
    assert_eq!(seconds, seconds_back);
    assert_eq!(texture_id, texture_id_back);
}

#[test]
fn when_negative_pixels_serialized_to_ron_then_roundtrip_preserves_sign() {
    // Arrange
    let pixels = Pixels(-42.5);

    // Act
    let ron = ron::to_string(&pixels).unwrap();
    let back: Pixels = ron::from_str(&ron).unwrap();

    // Assert
    assert_eq!(pixels, back);
}

/// @doc: Custom `Add`/`Sub`/`Mul` impls on `Pixels` newtype — verifies the
/// operator overloads forward correctly to the inner f32. A broken impl
/// would cause layout math (card spacing, stash grid stride) to silently
/// produce wrong values.
#[test]
fn when_pixels_arithmetic_then_add_sub_mul_produce_correct_results() {
    assert_eq!(Pixels(1.5) + Pixels(2.5), Pixels(4.0));
    assert_eq!(Pixels(5.0) - Pixels(2.0), Pixels(3.0));
    assert_eq!(Pixels(4.0) * 0.5, Pixels(2.0));
}

/// @doc: Custom `Add`/`Sub`/`Mul` impls on `Seconds` newtype — time arithmetic
/// is used throughout the fixed timestep and animation systems. A broken
/// operator would cause simulation to run at the wrong speed.
#[test]
fn when_seconds_arithmetic_then_add_sub_mul_produce_correct_results() {
    assert_eq!(Seconds(0.5) + Seconds(0.25), Seconds(0.75));
    assert_eq!(Seconds(1.0) - Seconds(0.25), Seconds(0.75));
    assert_eq!(Seconds(0.016) * 2.0, Seconds(0.032));
}

proptest::proptest! {
    #[test]
    fn when_any_finite_pixels_then_ron_roundtrip_preserves_value(v in -1e10_f32..=1e10) {
        // Arrange
        let pixels = Pixels(v);

        // Act
        let ron = ron::to_string(&pixels).unwrap();
        let back: Pixels = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(pixels, back);
    }

    #[test]
    fn when_any_finite_seconds_then_ron_roundtrip_preserves_value(v in -1e10_f32..=1e10) {
        // Arrange
        let seconds = Seconds(v);

        // Act
        let ron = ron::to_string(&seconds).unwrap();
        let back: Seconds = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(seconds, back);
    }
}
