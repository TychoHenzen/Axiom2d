use engine_core::color::Color;
use engine_core::types::Pixels;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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
