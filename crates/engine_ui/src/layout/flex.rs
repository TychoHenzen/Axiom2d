use bevy_ecs::component::Component;
use glam::Vec2;
use serde::{Deserialize, Serialize};

use super::margin::Margin;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FlexDirection {
    Row,
    Column,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FlexLayout {
    pub direction: FlexDirection,
    pub gap: f32,
}

pub fn compute_flex_offsets(layout: &FlexLayout, children: &[(Vec2, Margin)]) -> Vec<Vec2> {
    let mut offsets = Vec::with_capacity(children.len());
    let mut cursor = 0.0_f32;

    for (i, (size, margin)) in children.iter().enumerate() {
        let leading = match layout.direction {
            FlexDirection::Row => margin.left,
            FlexDirection::Column => margin.top,
        };
        cursor += leading;

        let offset = match layout.direction {
            FlexDirection::Row => Vec2::new(cursor, 0.0),
            FlexDirection::Column => Vec2::new(0.0, cursor),
        };
        offsets.push(offset);

        let extent = match layout.direction {
            FlexDirection::Row => size.x + margin.right,
            FlexDirection::Column => size.y + margin.bottom,
        };
        cursor += extent;

        if i + 1 < children.len() {
            cursor += layout.gap;
        }
    }

    offsets
}
