use bevy_ecs::component::Component;
use bevy_ecs::prelude::{Query, Res, ResMut};
use engine_core::prelude::Pixels;
use engine_render::prelude::{Rect, RendererRes};
use engine_scene::prelude::{EffectiveVisibility, GlobalTransform2D};
use serde::{Deserialize, Serialize};

use crate::anchor::anchor_offset;
use crate::theme::UiTheme;
use crate::ui_node::UiNode;

#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ProgressBar {
    pub value: f32,
    pub max: f32,
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self {
            value: 0.0,
            max: 100.0,
        }
    }
}

pub fn progress_bar_render_system(
    bars: Query<(
        &ProgressBar,
        &UiNode,
        &GlobalTransform2D,
        Option<&EffectiveVisibility>,
    )>,
    theme: Res<UiTheme>,
    mut renderer: ResMut<RendererRes>,
) {
    for (bar, node, transform, visibility) in &bars {
        if visibility.is_some_and(|v| !v.0) {
            continue;
        }

        let offset = anchor_offset(node.anchor, node.size);
        let top_left = transform.0.translation + offset;

        if let Some(bg) = node.background {
            renderer.draw_rect(Rect {
                x: Pixels(top_left.x),
                y: Pixels(top_left.y),
                width: Pixels(node.size.x),
                height: Pixels(node.size.y),
                color: bg,
            });
        }

        let fraction = if bar.max == 0.0 {
            0.0
        } else {
            (bar.value / bar.max).clamp(0.0, 1.0)
        };
        let fill_width = fraction * node.size.x;

        if fill_width > 0.0 {
            renderer.draw_rect(Rect {
                x: Pixels(top_left.x),
                y: Pixels(top_left.y),
                width: Pixels(fill_width),
                height: Pixels(node.size.y),
                color: theme.normal_color,
            });
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    use bevy_ecs::prelude::{Schedule, World};
    use engine_core::prelude::Color;
    use engine_render::testing::RectCallLog;
    use glam::{Affine2, Vec2};

    use crate::anchor::Anchor;
    use crate::test_helpers::make_spy_world;

    fn setup_world_with_spy() -> (World, Schedule, Arc<Mutex<Vec<String>>>, RectCallLog) {
        let (mut world, log, rect_cap) = make_spy_world();
        world.insert_resource(UiTheme::default());
        let mut schedule = Schedule::default();
        schedule.add_systems(progress_bar_render_system);
        (world, schedule, log, rect_cap)
    }

    #[test]
    fn when_progress_bar_roundtrip_ron_then_value_and_max_preserved() {
        // Arrange
        let bar = ProgressBar {
            value: 37.5,
            max: 200.0,
        };

        // Act
        let ron_str = ron::to_string(&bar).unwrap();
        let restored: ProgressBar = ron::from_str(&ron_str).unwrap();

        // Assert
        assert_eq!(restored, bar);
    }

    #[test]
    fn when_progress_bar_at_zero_then_only_background_drawn() {
        // Arrange
        let (mut world, mut schedule, log, _) = setup_world_with_spy();
        world.spawn((
            ProgressBar {
                value: 0.0,
                max: 100.0,
            },
            UiNode {
                size: Vec2::new(200.0, 20.0),
                anchor: Anchor::TopLeft,
                background: Some(Color::from_u8(50, 50, 50, 255)),
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::IDENTITY),
        ));

        // Act
        schedule.run(&mut world);

        // Assert
        let calls = log.lock().unwrap();
        assert_eq!(calls.iter().filter(|c| *c == "draw_rect").count(), 1);
    }

    #[test]
    fn when_progress_bar_at_half_then_filled_rect_is_half_width() {
        // Arrange
        let (mut world, mut schedule, _, rects) = setup_world_with_spy();
        world.spawn((
            ProgressBar {
                value: 50.0,
                max: 100.0,
            },
            UiNode {
                size: Vec2::new(200.0, 20.0),
                anchor: Anchor::TopLeft,
                background: Some(Color::from_u8(50, 50, 50, 255)),
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::IDENTITY),
        ));

        // Act
        schedule.run(&mut world);

        // Assert
        let rects = rects.lock().unwrap();
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[1].width, Pixels(100.0));
    }

    #[test]
    fn when_progress_bar_at_full_then_filled_rect_matches_node_width() {
        // Arrange
        let (mut world, mut schedule, _, rects) = setup_world_with_spy();
        world.spawn((
            ProgressBar {
                value: 100.0,
                max: 100.0,
            },
            UiNode {
                size: Vec2::new(200.0, 20.0),
                anchor: Anchor::TopLeft,
                background: Some(Color::from_u8(50, 50, 50, 255)),
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::IDENTITY),
        ));

        // Act
        schedule.run(&mut world);

        // Assert
        let rects = rects.lock().unwrap();
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[1].width, Pixels(200.0));
    }

    #[test]
    fn when_progress_bar_exceeds_max_then_filled_rect_capped_at_node_width() {
        // Arrange
        let (mut world, mut schedule, _, rects) = setup_world_with_spy();
        world.spawn((
            ProgressBar {
                value: 150.0,
                max: 100.0,
            },
            UiNode {
                size: Vec2::new(200.0, 20.0),
                anchor: Anchor::TopLeft,
                background: Some(Color::from_u8(50, 50, 50, 255)),
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::IDENTITY),
        ));

        // Act
        schedule.run(&mut world);

        // Assert
        let rects = rects.lock().unwrap();
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[1].width, Pixels(200.0));
    }

    #[test]
    fn when_progress_bar_with_center_anchor_then_draw_rect_offset_applied() {
        // Arrange
        let (mut world, mut schedule, _, rects) = setup_world_with_spy();
        world.spawn((
            ProgressBar {
                value: 50.0,
                max: 100.0,
            },
            UiNode {
                size: Vec2::new(200.0, 20.0),
                anchor: Anchor::Center,
                background: Some(Color::from_u8(50, 50, 50, 255)),
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::from_translation(Vec2::new(300.0, 100.0))),
        ));

        // Act
        schedule.run(&mut world);

        // Assert — Center anchor offset = (-100, -10), so top_left = (200, 90)
        let rects = rects.lock().unwrap();
        assert_eq!(rects.len(), 2);
        assert_eq!(rects[0].x, Pixels(200.0));
        assert_eq!(rects[0].y, Pixels(90.0));
        assert_eq!(rects[1].x, Pixels(200.0));
        assert_eq!(rects[1].y, Pixels(90.0));
        assert_eq!(rects[1].width, Pixels(100.0));
    }

    #[test]
    fn when_progress_bar_invisible_then_no_draw() {
        // Arrange
        let (mut world, mut schedule, log, _) = setup_world_with_spy();
        world.spawn((
            ProgressBar {
                value: 50.0,
                max: 100.0,
            },
            UiNode {
                size: Vec2::new(200.0, 20.0),
                background: Some(Color::WHITE),
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            EffectiveVisibility(false),
        ));

        // Act
        schedule.run(&mut world);

        // Assert
        let calls = log.lock().unwrap();
        assert_eq!(calls.iter().filter(|c| *c == "draw_rect").count(), 0);
    }
}
