#![allow(clippy::unwrap_used)]

use engine_core::types::{Pixels, Seconds};

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
