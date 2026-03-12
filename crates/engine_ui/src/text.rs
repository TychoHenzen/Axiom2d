use bevy_ecs::component::Component;
use engine_core::prelude::Color;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Text {
    pub content: String,
    pub font_size: f32,
    pub color: Color,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            content: String::new(),
            font_size: 16.0,
            color: Color::WHITE,
        }
    }
}
