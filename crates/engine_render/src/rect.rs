use engine_core::color::Color;
use engine_core::types::Pixels;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: Pixels,
    pub y: Pixels,
    pub width: Pixels,
    pub height: Pixels,
    pub color: Color,
}

#[cfg(test)]
mod tests {
    use engine_core::color::Color;
    use engine_core::types::Pixels;

    use super::Rect;

    #[test]
    fn when_rect_constructed_with_literal_values_then_stores_all_fields_exactly() {
        // Arrange
        let color = Color::new(1.0, 0.5, 0.0, 1.0);
        let r = Rect {
            x: Pixels(10.0),
            y: Pixels(20.0),
            width: Pixels(100.0),
            height: Pixels(50.0),
            color,
        };

        // Act
        let r2 = r;

        // Assert
        assert_eq!(r, r2);
        assert_eq!(r.x, Pixels(10.0));
        assert_eq!(r.y, Pixels(20.0));
        assert_eq!(r.width, Pixels(100.0));
        assert_eq!(r.height, Pixels(50.0));
        assert_eq!(r.color, color);
    }

    #[test]
    fn when_rect_derives_compared_then_unequal_on_any_differing_field_clone_equals_and_debug_names_type() {
        // Arrange
        let base = Rect {
            x: Pixels(10.0),
            y: Pixels(20.0),
            width: Pixels(100.0),
            height: Pixels(50.0),
            color: Color::new(1.0, 0.5, 0.0, 1.0),
        };

        // Act
        let cloned = base.clone();
        let debug_str = format!("{:?}", base);

        // Assert
        assert_eq!(base, cloned);
        assert_ne!(base, Rect { x: Pixels(99.0), ..base });
        assert_ne!(base, Rect { y: Pixels(99.0), ..base });
        assert_ne!(base, Rect { width: Pixels(99.0), ..base });
        assert_ne!(base, Rect { height: Pixels(99.0), ..base });
        assert_ne!(base, Rect { color: Color::RED, ..base });
        assert!(debug_str.contains("Rect"));
    }

    #[test]
    fn when_rect_has_negative_pixel_values_then_stores_without_clamping() {
        // Act
        let r = Rect {
            x: Pixels(-10.0),
            y: Pixels(-20.0),
            width: Pixels(-100.0),
            height: Pixels(-50.0),
            color: Color::WHITE,
        };

        // Assert
        assert_eq!(r.x, Pixels(-10.0));
        assert_eq!(r.y, Pixels(-20.0));
        assert_eq!(r.width, Pixels(-100.0));
        assert_eq!(r.height, Pixels(-50.0));
    }
}
