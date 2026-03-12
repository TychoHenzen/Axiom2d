use bevy_ecs::prelude::Resource;
use engine_core::prelude::Color;
use serde::{Deserialize, Serialize};

#[derive(Resource, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct UiTheme {
    pub normal_color: Color,
    pub hovered_color: Color,
    pub pressed_color: Color,
    pub disabled_color: Color,
    pub text_color: Color,
    pub font_size: f32,
}

impl Default for UiTheme {
    fn default() -> Self {
        Self {
            normal_color: Color::from_u8(60, 60, 60, 255),
            hovered_color: Color::from_u8(80, 80, 80, 255),
            pressed_color: Color::from_u8(40, 40, 40, 255),
            disabled_color: Color::from_u8(30, 30, 30, 128),
            text_color: Color::WHITE,
            font_size: 16.0,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

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
}
