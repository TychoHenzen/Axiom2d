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
