// EVOLVE-BLOCK-START
use bevy_ecs::component::Component;
use engine_core::prelude::Color;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Text {
    pub content: String,
    pub font_size: f32,
    pub color: Color,
    /// When set, text wraps at word boundaries to fit within this pixel width.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_width: Option<f32>,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            content: String::new(),
            font_size: 16.0,
            color: Color::WHITE,
            max_width: None,
        }
    }
}
// EVOLVE-BLOCK-END
