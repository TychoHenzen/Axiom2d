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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::layout::{FlexDirection, FlexLayout};
    use crate::widget::Text;

    #[test]
    fn when_ui_node_roundtrip_ron_then_preserved() {
        // Arrange
        let node = UiNode {
            size: Vec2::new(200.0, 100.0),
            anchor: Anchor::Center,
            margin: Margin {
                top: 5.0,
                right: 10.0,
                bottom: 15.0,
                left: 20.0,
            },
            background: Some(Color::new(1.0, 0.0, 0.5, 0.8)),
        };

        // Act
        let ron_str = ron::to_string(&node).unwrap();
        let restored: UiNode = ron::from_str(&ron_str).unwrap();

        // Assert
        assert_eq!(restored, node);
    }

    #[test]
    fn when_flex_layout_roundtrip_ron_then_preserved() {
        // Arrange
        let layout = FlexLayout {
            direction: FlexDirection::Column,
            gap: 12.5,
        };

        // Act
        let ron_str = ron::to_string(&layout).unwrap();
        let restored: FlexLayout = ron::from_str(&ron_str).unwrap();

        // Assert
        assert_eq!(restored, layout);
    }

    #[test]
    fn when_text_roundtrip_ron_then_preserved() {
        // Arrange
        let text = Text {
            content: "Hello UI".into(),
            font_size: 24.0,
            color: Color::new(0.2, 0.8, 0.4, 1.0),
            max_width: None,
        };

        // Act
        let ron_str = ron::to_string(&text).unwrap();
        let restored: Text = ron::from_str(&ron_str).unwrap();

        // Assert
        assert_eq!(restored, text);
    }
}
