use bevy_ecs::prelude::{Query, ResMut};
use engine_render::prelude::RendererRes;
use engine_scene::prelude::{EffectiveVisibility, GlobalTransform2D};

use crate::is_hidden;
use crate::widget::{UiNode, node_rect};

pub fn ui_render_system(
    nodes: Query<(&UiNode, &GlobalTransform2D, Option<&EffectiveVisibility>)>,
    mut renderer: ResMut<RendererRes>,
) {
    for (node, global_transform, visibility) in &nodes {
        if is_hidden(visibility) {
            continue;
        }

        let Some(color) = node.background else {
            continue;
        };

        renderer.draw_rect(node_rect(node, global_transform, color));
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    use bevy_ecs::prelude::{Schedule, World};
    use engine_core::prelude::{Color, Pixels};
    use engine_render::testing::RectCallLog;
    use engine_scene::prelude::GlobalTransform2D;
    use glam::{Affine2, Vec2};

    use crate::layout::Anchor;
    use crate::test_helpers::make_spy_world;
    use crate::widget::UiNode;

    fn setup_world_with_spy() -> (World, Schedule, Arc<Mutex<Vec<String>>>, RectCallLog) {
        let (world, log, rect_cap) = make_spy_world();
        let mut schedule = Schedule::default();
        schedule.add_systems(ui_render_system);
        (world, schedule, log, rect_cap)
    }

    #[test]
    fn when_ui_node_has_background_then_draw_rect_called() {
        // Arrange
        let (mut world, mut schedule, log, _) = setup_world_with_spy();
        world.spawn((
            UiNode {
                size: Vec2::new(100.0, 50.0),
                background: Some(Color::WHITE),
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
    fn when_ui_node_no_background_then_no_draw() {
        // Arrange
        let (mut world, mut schedule, log, _) = setup_world_with_spy();
        world.spawn((
            UiNode {
                background: None,
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::IDENTITY),
        ));

        // Act
        schedule.run(&mut world);

        // Assert
        let calls = log.lock().unwrap();
        assert_eq!(calls.iter().filter(|c| *c == "draw_rect").count(), 0);
    }

    #[test]
    fn when_top_left_anchor_then_rect_at_transform_position() {
        // Arrange
        let (mut world, mut schedule, _, rects) = setup_world_with_spy();
        world.spawn((
            UiNode {
                size: Vec2::new(80.0, 40.0),
                anchor: Anchor::TopLeft,
                background: Some(Color::RED),
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 150.0))),
        ));

        // Act
        schedule.run(&mut world);

        // Assert
        let rects = rects.lock().unwrap();
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].x, Pixels(200.0));
        assert_eq!(rects[0].y, Pixels(150.0));
    }

    #[test]
    fn when_center_anchor_then_rect_adjusted_by_half_size() {
        // Arrange
        let (mut world, mut schedule, _, rects) = setup_world_with_spy();
        world.spawn((
            UiNode {
                size: Vec2::new(100.0, 60.0),
                anchor: Anchor::Center,
                background: Some(Color::BLUE),
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::from_translation(Vec2::new(300.0, 200.0))),
        ));

        // Act
        schedule.run(&mut world);

        // Assert
        let rects = rects.lock().unwrap();
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].x, Pixels(250.0));
        assert_eq!(rects[0].y, Pixels(170.0));
    }

    #[test]
    fn when_ui_node_rendered_then_rect_size_matches_node() {
        // Arrange
        let (mut world, mut schedule, _, rects) = setup_world_with_spy();
        world.spawn((
            UiNode {
                size: Vec2::new(120.0, 80.0),
                background: Some(Color::WHITE),
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::IDENTITY),
        ));

        // Act
        schedule.run(&mut world);

        // Assert
        let rects = rects.lock().unwrap();
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].width, Pixels(120.0));
        assert_eq!(rects[0].height, Pixels(80.0));
    }

    #[test]
    fn when_ui_node_rendered_then_rect_color_matches_background() {
        // Arrange
        let (mut world, mut schedule, _, rects) = setup_world_with_spy();
        let color = Color::new(1.0, 0.0, 0.5, 1.0);
        world.spawn((
            UiNode {
                size: Vec2::new(50.0, 50.0),
                background: Some(color),
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::IDENTITY),
        ));

        // Act
        schedule.run(&mut world);

        // Assert
        let rects = rects.lock().unwrap();
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].color, color);
    }

    #[test]
    fn when_effective_visibility_false_then_no_draw() {
        // Arrange
        let (mut world, mut schedule, log, _) = setup_world_with_spy();
        world.spawn((
            UiNode {
                size: Vec2::new(100.0, 50.0),
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

    #[test]
    fn when_two_ui_nodes_then_both_drawn() {
        // Arrange
        let (mut world, mut schedule, log, _) = setup_world_with_spy();
        for _ in 0..2 {
            world.spawn((
                UiNode {
                    size: Vec2::new(50.0, 30.0),
                    background: Some(Color::WHITE),
                    ..UiNode::default()
                },
                GlobalTransform2D(Affine2::IDENTITY),
            ));
        }

        // Act
        schedule.run(&mut world);

        // Assert
        let calls = log.lock().unwrap();
        assert_eq!(calls.iter().filter(|c| *c == "draw_rect").count(), 2);
    }
}
