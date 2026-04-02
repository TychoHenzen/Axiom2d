#![allow(clippy::unwrap_used)]

use engine_core::color::Color;
use engine_core::types::Pixels;
use engine_render::rect::Rect;

#[test]
fn when_rect_serialized_to_ron_then_deserializes_to_equal_value() {
    // Arrange
    let rect = Rect {
        x: Pixels(10.0),
        y: Pixels(-20.0),
        width: Pixels(100.0),
        height: Pixels(50.0),
        color: Color::new(0.5, 0.6, 0.7, 0.8),
    };

    // Act
    let ron = ron::to_string(&rect).unwrap();
    let back: Rect = ron::from_str(&ron).unwrap();

    // Assert
    assert_eq!(rect, back);
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
