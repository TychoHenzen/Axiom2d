use std::ops::{Add, Mul, Sub};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Pixels(pub f32);

impl Add for Pixels {
    type Output = Pixels;
    fn add(self, rhs: Pixels) -> Pixels {
        Pixels(self.0 + rhs.0)
    }
}

impl Sub for Pixels {
    type Output = Pixels;
    fn sub(self, rhs: Pixels) -> Pixels {
        Pixels(self.0 - rhs.0)
    }
}

impl Mul<f32> for Pixels {
    type Output = Pixels;
    fn mul(self, rhs: f32) -> Pixels {
        Pixels(self.0 * rhs)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Seconds(pub f32);

impl Add for Seconds {
    type Output = Seconds;
    fn add(self, rhs: Seconds) -> Seconds {
        Seconds(self.0 + rhs.0)
    }
}

impl Sub for Seconds {
    type Output = Seconds;
    fn sub(self, rhs: Seconds) -> Seconds {
        Seconds(self.0 - rhs.0)
    }
}

impl Mul<f32> for Seconds {
    type Output = Seconds;
    fn mul(self, rhs: f32) -> Seconds {
        Seconds(self.0 * rhs)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TextureId(pub u32);

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

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

    #[test]
    fn when_pixels_arithmetic_then_add_sub_mul_produce_correct_results() {
        assert_eq!(Pixels(1.5) + Pixels(2.5), Pixels(4.0));
        assert_eq!(Pixels(5.0) - Pixels(2.0), Pixels(3.0));
        assert_eq!(Pixels(4.0) * 0.5, Pixels(2.0));
    }

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
}
