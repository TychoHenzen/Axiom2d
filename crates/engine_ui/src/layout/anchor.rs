use glam::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Anchor {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

pub fn anchor_offset(anchor: Anchor, size: Vec2) -> Vec2 {
    let half = size * 0.5;
    match anchor {
        Anchor::TopLeft => Vec2::ZERO,
        Anchor::TopCenter => Vec2::new(-half.x, 0.0),
        Anchor::TopRight => Vec2::new(-size.x, 0.0),
        Anchor::CenterLeft => Vec2::new(0.0, -half.y),
        Anchor::Center => -half,
        Anchor::CenterRight => Vec2::new(-size.x, -half.y),
        Anchor::BottomLeft => Vec2::new(0.0, -size.y),
        Anchor::BottomCenter => Vec2::new(-half.x, -size.y),
        Anchor::BottomRight => -size,
    }
}
