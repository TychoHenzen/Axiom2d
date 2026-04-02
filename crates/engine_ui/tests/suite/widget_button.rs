#![allow(clippy::unwrap_used)]

use std::sync::{Arc, Mutex};

use bevy_ecs::prelude::{Schedule, World};
use engine_core::prelude::Pixels;
use engine_render::prelude::RendererRes;
use engine_render::testing::{RectCallLog, SpyRenderer};
use engine_scene::prelude::{EffectiveVisibility, GlobalTransform2D};
use engine_ui::interaction::Interaction;
use engine_ui::layout::Anchor;
use engine_ui::theme::UiTheme;
use engine_ui::widget::{Button, UiNode, button_render_system};
use glam::{Affine2, Vec2};

fn setup_world_with_spy() -> (World, Schedule, Arc<Mutex<Vec<String>>>, RectCallLog) {
    let mut world = World::new();
    let log = Arc::new(Mutex::new(Vec::new()));
    let rect_cap: RectCallLog = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(Arc::clone(&log)).with_rect_capture(Arc::clone(&rect_cap));
    world.insert_resource(RendererRes::new(Box::new(spy)));
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
fn when_button_with_center_anchor_then_draw_rect_offset_applied() {
    // Arrange
    let (mut world, mut schedule, _, rects) = setup_world_with_spy();
    world.spawn((
        Button { disabled: false },
        UiNode {
            size: Vec2::new(100.0, 40.0),
            anchor: Anchor::Center,
            ..UiNode::default()
        },
        GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 100.0))),
        Interaction::None,
    ));

    // Act
    schedule.run(&mut world);

    // Assert — Center anchor offset = (-50, -20), so top_left = (150, 80)
    let rects = rects.lock().unwrap();
    assert_eq!(rects.len(), 1);
    assert_eq!(rects[0].x, Pixels(150.0));
    assert_eq!(rects[0].y, Pixels(80.0));
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
