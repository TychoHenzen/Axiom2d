use bevy_ecs::component::Component;
use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::margin::Margin;

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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn when_row_no_gap_then_children_horizontal() {
        // Arrange
        let layout = FlexLayout {
            direction: FlexDirection::Row,
            gap: 0.0,
        };
        let children = [
            (Vec2::new(40.0, 20.0), Margin::default()),
            (Vec2::new(60.0, 30.0), Margin::default()),
        ];

        // Act
        let offsets = compute_flex_offsets(&layout, &children);

        // Assert
        assert_eq!(offsets, vec![Vec2::ZERO, Vec2::new(40.0, 0.0)]);
    }

    #[test]
    fn when_row_with_gap_then_gap_between_children() {
        // Arrange
        let layout = FlexLayout {
            direction: FlexDirection::Row,
            gap: 8.0,
        };
        let children = [
            (Vec2::new(40.0, 20.0), Margin::default()),
            (Vec2::new(60.0, 30.0), Margin::default()),
        ];

        // Act
        let offsets = compute_flex_offsets(&layout, &children);

        // Assert
        assert_eq!(offsets, vec![Vec2::ZERO, Vec2::new(48.0, 0.0)]);
    }

    #[test]
    fn when_column_with_gap_then_children_vertical() {
        // Arrange
        let layout = FlexLayout {
            direction: FlexDirection::Column,
            gap: 4.0,
        };
        let children = [
            (Vec2::new(50.0, 20.0), Margin::default()),
            (Vec2::new(50.0, 30.0), Margin::default()),
        ];

        // Act
        let offsets = compute_flex_offsets(&layout, &children);

        // Assert
        assert_eq!(offsets, vec![Vec2::ZERO, Vec2::new(0.0, 24.0)]);
    }

    #[test]
    fn when_row_with_margins_then_margins_in_spacing() {
        // Arrange
        let layout = FlexLayout {
            direction: FlexDirection::Row,
            gap: 0.0,
        };
        let children = [
            (
                Vec2::new(40.0, 20.0),
                Margin {
                    right: 5.0,
                    ..Margin::default()
                },
            ),
            (
                Vec2::new(60.0, 30.0),
                Margin {
                    left: 3.0,
                    ..Margin::default()
                },
            ),
        ];

        // Act
        let offsets = compute_flex_offsets(&layout, &children);

        // Assert
        assert_eq!(offsets, vec![Vec2::ZERO, Vec2::new(48.0, 0.0)]);
    }

    #[test]
    fn when_single_child_then_offset_at_origin() {
        // Arrange
        let layout = FlexLayout {
            direction: FlexDirection::Row,
            gap: 10.0,
        };
        let children = [(Vec2::new(50.0, 30.0), Margin::default())];

        // Act
        let offsets = compute_flex_offsets(&layout, &children);

        // Assert
        assert_eq!(offsets, vec![Vec2::ZERO]);
    }

    #[test]
    fn when_empty_children_then_empty_offsets() {
        // Arrange
        let layout = FlexLayout {
            direction: FlexDirection::Row,
            gap: 5.0,
        };

        // Act
        let offsets = compute_flex_offsets(&layout, &[]);

        // Assert
        assert!(offsets.is_empty());
    }
}
