use bevy_ecs::component::Component;
use engine_core::prelude::{Color, Pixels};
use engine_render::prelude::Rect;
use engine_scene::prelude::GlobalTransform2D;
use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::layout::Anchor;
use crate::layout::Margin;
use crate::layout::anchor_offset;

#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiNode {
    pub size: Vec2,
    pub anchor: Anchor,
    pub margin: Margin,
    pub background: Option<Color>,
}

impl Default for UiNode {
    fn default() -> Self {
        Self {
            size: Vec2::ZERO,
            anchor: Anchor::TopLeft,
            margin: Margin::default(),
            background: None,
        }
    }
}

pub(crate) fn node_rect(node: &UiNode, transform: &GlobalTransform2D, color: Color) -> Rect {
    let offset = anchor_offset(node.anchor, node.size);
    let top_left = transform.0.translation + offset;
    Rect {
        x: Pixels(top_left.x),
        y: Pixels(top_left.y),
        width: Pixels(node.size.x),
        height: Pixels(node.size.y),
        color,
    }
}
