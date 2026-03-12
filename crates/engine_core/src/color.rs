use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
    pub const RED: Self = Self {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const GREEN: Self = Self {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    pub const BLUE: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: f32::from(r) / 255.0,
            g: f32::from(g) / 255.0,
            b: f32::from(b) / 255.0,
            a: f32::from(a) / 255.0,
        }
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::Color;

    #[test]
    fn when_color_serialized_to_ron_then_deserializes_to_equal_value() {
        // Arrange
        let color = Color::new(0.1, 0.5, 0.9, 0.75);

        // Act
        let ron = ron::to_string(&color).unwrap();
        let back: Color = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(color, back);
    }

    #[test]
    fn when_transparent_color_serialized_to_ron_then_roundtrip_preserves_zero_alpha() {
        // Arrange
        let color = Color::TRANSPARENT;

        // Act
        let ron = ron::to_string(&color).unwrap();
        let back: Color = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(color, back);
    }

    #[test]
    fn when_color_from_u8_called_then_converts_to_normalized_f32() {
        // Act
        let c = Color::from_u8(255, 128, 64, 255);

        // Assert
        assert_eq!(c.r, 1.0);
        assert!((c.g - 128.0 / 255.0).abs() < 1e-6);
        assert!((c.b - 64.0 / 255.0).abs() < 1e-6);
        assert_eq!(c.a, 1.0);
    }
}
