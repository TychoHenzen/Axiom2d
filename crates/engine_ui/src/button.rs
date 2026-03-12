use bevy_ecs::component::Component;
use bevy_ecs::prelude::{Query, Res, ResMut};
use engine_core::prelude::Pixels;
use engine_render::prelude::{Rect, RendererRes};
use engine_scene::prelude::{EffectiveVisibility, GlobalTransform2D};
use serde::{Deserialize, Serialize};

use crate::anchor::anchor_offset;
use crate::interaction::Interaction;
use crate::theme::UiTheme;
use crate::ui_node::UiNode;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Button {
    pub disabled: bool,
}

#[allow(clippy::type_complexity)]
pub fn button_render_system(
    buttons: Query<(
        &Button,
        &UiNode,
        &GlobalTransform2D,
        Option<&Interaction>,
        Option<&EffectiveVisibility>,
    )>,
    theme: Res<UiTheme>,
    mut renderer: ResMut<RendererRes>,
) {
    for (button, node, transform, interaction, visibility) in &buttons {
        if visibility.is_some_and(|v| !v.0) {
            continue;
        }

        let color = if button.disabled {
            theme.disabled_color
        } else {
            match interaction.copied().unwrap_or_default() {
                Interaction::Pressed => theme.pressed_color,
                Interaction::Hovered => theme.hovered_color,
                Interaction::None => theme.normal_color,
            }
        };

        let offset = anchor_offset(node.anchor, node.size);
        let top_left = transform.0.translation + offset;

        renderer.draw_rect(Rect {
            x: Pixels(top_left.x),
            y: Pixels(top_left.y),
            width: Pixels(node.size.x),
            height: Pixels(node.size.y),
            color,
        });
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
        schedule.add_systems(button_render_system);
        (world, schedule, log, rect_cap)
    }

    #[test]
    fn when_button_roundtrip_ron_then_disabled_preserved() {
        // Arrange
        let button = Button { disabled: true };

        // Act
        let ron_str = ron::to_string(&button).unwrap();
        let restored: Button = ron::from_str(&ron_str).unwrap();

        // Assert
        assert_eq!(restored, button);
    }

    #[test]
    fn when_button_not_hovered_then_normal_color_used() {
        // Arrange
        let (mut world, mut schedule, _, rects) = setup_world_with_spy();
        world.spawn((
            Button { disabled: false },
            UiNode {
                size: Vec2::new(100.0, 40.0),
                anchor: Anchor::TopLeft,
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::from_translation(Vec2::new(50.0, 80.0))),
            Interaction::None,
        ));

        // Act
        schedule.run(&mut world);

        // Assert
        let rects = rects.lock().unwrap();
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].color, UiTheme::default().normal_color);
    }

    #[test]
    fn when_button_hovered_then_hovered_color_used() {
        // Arrange
        let (mut world, mut schedule, _, rects) = setup_world_with_spy();
        world.spawn((
            Button { disabled: false },
            UiNode {
                size: Vec2::new(100.0, 40.0),
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            Interaction::Hovered,
        ));

        // Act
        schedule.run(&mut world);

        // Assert
        let rects = rects.lock().unwrap();
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].color, UiTheme::default().hovered_color);
    }

    #[test]
    fn when_button_pressed_then_pressed_color_used() {
        // Arrange
        let (mut world, mut schedule, _, rects) = setup_world_with_spy();
        world.spawn((
            Button { disabled: false },
            UiNode {
                size: Vec2::new(100.0, 40.0),
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            Interaction::Pressed,
        ));

        // Act
        schedule.run(&mut world);

        // Assert
        let rects = rects.lock().unwrap();
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].color, UiTheme::default().pressed_color);
    }

    #[test]
    fn when_button_disabled_then_disabled_color_used_regardless_of_interaction() {
        // Arrange
        let (mut world, mut schedule, _, rects) = setup_world_with_spy();
        world.spawn((
            Button { disabled: true },
            UiNode {
                size: Vec2::new(100.0, 40.0),
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::IDENTITY),
            Interaction::Hovered,
        ));

        // Act
        schedule.run(&mut world);

        // Assert
        let rects = rects.lock().unwrap();
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].color, UiTheme::default().disabled_color);
    }

    #[test]
    fn when_button_invisible_then_no_draw() {
        // Arrange
        let (mut world, mut schedule, log, _) = setup_world_with_spy();
        world.spawn((
            Button { disabled: false },
            UiNode {
                size: Vec2::new(100.0, 40.0),
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
    fn when_button_rendered_then_position_and_size_match_node() {
        // Arrange
        let (mut world, mut schedule, _, rects) = setup_world_with_spy();
        world.spawn((
            Button { disabled: false },
            UiNode {
                size: Vec2::new(100.0, 40.0),
                anchor: Anchor::TopLeft,
                ..UiNode::default()
            },
            GlobalTransform2D(Affine2::from_translation(Vec2::new(50.0, 80.0))),
            Interaction::None,
        ));

        // Act
        schedule.run(&mut world);

        // Assert
        let rects = rects.lock().unwrap();
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].x, Pixels(50.0));
        assert_eq!(rects[0].y, Pixels(80.0));
        assert_eq!(rects[0].width, Pixels(100.0));
        assert_eq!(rects[0].height, Pixels(40.0));
    }
}
