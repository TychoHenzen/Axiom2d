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

    proptest::proptest! {
        #[test]
        fn when_any_row_children_then_output_length_matches_and_x_offsets_increase(
            gap in 0.0_f32..=20.0,
            widths in proptest::collection::vec(1.0_f32..=200.0, 1..=8),
        ) {
            // Arrange
            let layout = FlexLayout {
                direction: FlexDirection::Row,
                gap,
            };
            let children: Vec<(Vec2, Margin)> = widths
                .iter()
                .map(|&w| (Vec2::new(w, 20.0), Margin::default()))
                .collect();

            // Act
            let offsets = compute_flex_offsets(&layout, &children);

            // Assert — length matches
            assert_eq!(offsets.len(), children.len());

            // Assert — x offsets are strictly increasing (positive sizes, zero margin)
            for i in 1..offsets.len() {
                assert!(
                    offsets[i].x > offsets[i - 1].x,
                    "offsets[{}].x={} should be > offsets[{}].x={}",
                    i,
                    offsets[i].x,
                    i - 1,
                    offsets[i - 1].x
                );
            }
        }
    }

    #[test]
    fn when_column_with_bottom_margin_and_gap_then_spacing_accumulates() {
        // Arrange
        let layout = FlexLayout {
            direction: FlexDirection::Column,
            gap: 5.0,
        };
        let children = [
            (
                Vec2::new(50.0, 20.0),
                Margin {
                    bottom: 10.0,
                    ..Margin::default()
                },
            ),
            (
                Vec2::new(50.0, 30.0),
                Margin {
                    top: 3.0,
                    bottom: 7.0,
                    ..Margin::default()
                },
            ),
            (Vec2::new(50.0, 25.0), Margin::default()),
        ];

        // Act
        let offsets = compute_flex_offsets(&layout, &children);

        // Assert
        // child[0]: cursor starts 0, leading=0, offset=(0,0), extent=20+10=30, gap=5, cursor=35
        // child[1]: leading=3, cursor=38, offset=(0,38), extent=30+7=37, gap=5, cursor=80
        // child[2]: leading=0, cursor=80, offset=(0,80)
        assert_eq!(offsets.len(), 3);
        assert_eq!(offsets[0], Vec2::new(0.0, 0.0));
        assert_eq!(offsets[1], Vec2::new(0.0, 38.0));
        assert_eq!(offsets[2], Vec2::new(0.0, 80.0));
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
