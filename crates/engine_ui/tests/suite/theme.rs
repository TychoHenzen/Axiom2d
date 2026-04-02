#![allow(clippy::unwrap_used)]

use engine_core::prelude::Color;
use engine_ui::theme::UiTheme;

#[test]
fn when_ui_theme_roundtrip_ron_then_all_fields_preserved() {
    // Arrange
    let theme = UiTheme {
        normal_color: Color::from_u8(10, 20, 30, 255),
        hovered_color: Color::from_u8(40, 50, 60, 255),
        pressed_color: Color::from_u8(70, 80, 90, 255),
        disabled_color: Color::from_u8(100, 110, 120, 128),
        text_color: Color::from_u8(200, 210, 220, 255),
        font_size: 24.0,
    };

    // Act
    let ron_str = ron::to_string(&theme).unwrap();
    let restored: UiTheme = ron::from_str(&ron_str).unwrap();

    // Assert
    assert_eq!(restored, theme);
}
