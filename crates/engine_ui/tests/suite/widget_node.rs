#![allow(clippy::unwrap_used)]

use engine_core::prelude::Color;
use engine_ui::layout::{Anchor, FlexDirection, FlexLayout, Margin};
use engine_ui::widget::{Text, UiNode};
use glam::Vec2;

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
