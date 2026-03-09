#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const WHITE: Self = Self { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const BLACK: Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const TRANSPARENT: Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
    pub const RED: Self = Self { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const GREEN: Self = Self { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const BLUE: Self = Self { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };

    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Color;

    #[test]
    fn when_color_new_called_then_stores_all_components() {
        // Act
        let c = Color::new(0.1, 0.2, 0.3, 1.0);

        // Assert
        assert_eq!(c.r, 0.1);
        assert_eq!(c.g, 0.2);
        assert_eq!(c.b, 0.3);
        assert_eq!(c.a, 1.0);
    }

    #[test]
    fn when_color_from_u8_called_then_converts_to_normalized_f32() {
        // Act
        let c = Color::from_u8(255, 128, 0, 255);

        // Assert
        assert_eq!(c.r, 1.0);
        assert!((c.g - 128.0 / 255.0).abs() < 1e-6);
        assert_eq!(c.b, 0.0);
        assert_eq!(c.a, 1.0);
    }

    #[test]
    fn when_using_color_then_supports_copy_clone_eq_and_debug() {
        // Arrange
        let c = Color::new(1.0, 0.0, 0.0, 1.0);

        // Act
        let c2 = c;

        // Assert
        assert_eq!(c, c2);
        assert_eq!(Color::new(0.0, 0.0, 0.0, 1.0), Color::new(0.0, 0.0, 0.0, 1.0));
        assert_ne!(Color::new(1.0, 0.0, 0.0, 1.0), Color::new(0.0, 1.0, 0.0, 1.0));
        let s = format!("{:?}", c);
        assert!(s.contains("Color"));
    }

    #[test]
    fn when_accessing_color_constants_then_returns_expected_values() {
        // Assert
        assert_eq!(Color::WHITE, Color::new(1.0, 1.0, 1.0, 1.0));
        assert_eq!(Color::BLACK, Color::new(0.0, 0.0, 0.0, 1.0));
        assert_eq!(Color::TRANSPARENT, Color::new(0.0, 0.0, 0.0, 0.0));
        assert_eq!(Color::RED, Color::new(1.0, 0.0, 0.0, 1.0));
        assert_eq!(Color::GREEN, Color::new(0.0, 1.0, 0.0, 1.0));
        assert_eq!(Color::BLUE, Color::new(0.0, 0.0, 1.0, 1.0));
    }
}
