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

impl Default for Rect {
    fn default() -> Self {
        Self {
            x: Pixels(0.0),
            y: Pixels(0.0),
            width: Pixels(0.0),
            height: Pixels(0.0),
            color: Color::WHITE,
        }
    }
}

#[cfg(test)]
mod tests {
    use engine_core::color::Color;
    use engine_core::types::Pixels;

    use super::Rect;

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
