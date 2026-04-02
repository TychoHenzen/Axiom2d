#![allow(clippy::unwrap_used)]

use std::sync::{Arc, Mutex};

use bevy_ecs::prelude::{Schedule, World};
use engine_core::prelude::{Color, Pixels};
use engine_render::prelude::RendererRes;
use engine_render::testing::{RectCallLog, SpyRenderer};
use engine_scene::prelude::{EffectiveVisibility, GlobalTransform2D};
use engine_ui::layout::Anchor;
use engine_ui::widget::{Panel, UiNode, panel_render_system};
use glam::{Affine2, Vec2};

fn setup_world_with_spy() -> (World, Schedule, Arc<Mutex<Vec<String>>>, RectCallLog) {
    let mut world = World::new();
    let log = Arc::new(Mutex::new(Vec::new()));
    let rect_cap: RectCallLog = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(Arc::clone(&log)).with_rect_capture(Arc::clone(&rect_cap));
    world.insert_resource(RendererRes::new(Box::new(spy)));
    let mut schedule = Schedule::default();
    schedule.add_systems(panel_render_system);
    (world, schedule, log, rect_cap)
}

#[test]
fn when_panel_roundtrip_ron_then_border_preserved() {
    // Arrange
    let panel = Panel {
        border_color: Some(Color::from_u8(255, 0, 0, 255)),
        border_width: 3.0,
    };

    // Act
    let ron_str = ron::to_string(&panel).unwrap();
    let restored: Panel = ron::from_str(&ron_str).unwrap();

    // Assert
    assert_eq!(restored, panel);
}

#[test]
fn when_panel_no_border_then_no_draw() {
    // Arrange
    let (mut world, mut schedule, log, _) = setup_world_with_spy();
    world.spawn((
        Panel {
            border_color: None,
            border_width: 0.0,
        },
        UiNode {
            size: Vec2::new(200.0, 150.0),
            anchor: Anchor::TopLeft,
            background: Some(Color::WHITE),
            ..UiNode::default()
        },
        GlobalTransform2D(Affine2::IDENTITY),
    ));

    // Act
    schedule.run(&mut world);

    // Assert — background is drawn by ui_render_system, not panel_render_system
    let calls = log.lock().unwrap();
    assert_eq!(calls.iter().filter(|c| *c == "draw_rect").count(), 0);
}

#[test]
fn when_panel_with_border_then_only_border_rects_drawn() {
    // Arrange
    let (mut world, mut schedule, log, _) = setup_world_with_spy();
    world.spawn((
        Panel {
            border_color: Some(Color::RED),
            border_width: 4.0,
        },
        UiNode {
            size: Vec2::new(200.0, 150.0),
            anchor: Anchor::TopLeft,
            background: Some(Color::WHITE),
            ..UiNode::default()
        },
        GlobalTransform2D(Affine2::IDENTITY),
    ));

    // Act
    schedule.run(&mut world);

    // Assert — only 4 border edges; background is handled by ui_render_system
    let calls = log.lock().unwrap();
    assert_eq!(calls.iter().filter(|c| *c == "draw_rect").count(), 4);
}

#[test]
fn when_panel_with_center_anchor_and_border_then_exact_border_positions() {
    // Arrange — panel at (200, 100) with Center anchor, size 120x80, border_width 4
    // Center offset = (-60, -40), so top_left = (140, 60)
    let (mut world, mut schedule, _, rects) = setup_world_with_spy();
    world.spawn((
        Panel {
            border_color: Some(Color::RED),
            border_width: 4.0,
        },
        UiNode {
            size: Vec2::new(120.0, 80.0),
            anchor: Anchor::Center,
            background: Some(Color::WHITE),
            ..UiNode::default()
        },
        GlobalTransform2D(Affine2::from_translation(Vec2::new(200.0, 100.0))),
    ));

    // Act
    schedule.run(&mut world);

    // Assert — 4 border edges only (background handled by ui_render_system)
    let rects = rects.lock().unwrap();
    assert_eq!(rects.len(), 4);

    // Top edge: x=140, y=60, w=120, h=4
    assert_eq!(rects[0].x, Pixels(140.0));
    assert_eq!(rects[0].y, Pixels(60.0));
    assert_eq!(rects[0].width, Pixels(120.0));
    assert_eq!(rects[0].height, Pixels(4.0));

    // Bottom edge: x=140, y=60+80-4=136, w=120, h=4
    assert_eq!(rects[1].x, Pixels(140.0));
    assert_eq!(rects[1].y, Pixels(136.0));
    assert_eq!(rects[1].width, Pixels(120.0));
    assert_eq!(rects[1].height, Pixels(4.0));

    // Left edge: x=140, y=60+4=64, w=4, h=80-2*4=72
    assert_eq!(rects[2].x, Pixels(140.0));
    assert_eq!(rects[2].y, Pixels(64.0));
    assert_eq!(rects[2].width, Pixels(4.0));
    assert_eq!(rects[2].height, Pixels(72.0));

    // Right edge: x=140+120-4=256, y=64, w=4, h=72
    assert_eq!(rects[3].x, Pixels(256.0));
    assert_eq!(rects[3].y, Pixels(64.0));
    assert_eq!(rects[3].width, Pixels(4.0));
    assert_eq!(rects[3].height, Pixels(72.0));
}

#[test]
fn when_panel_invisible_then_no_draw() {
    // Arrange
    let (mut world, mut schedule, log, _) = setup_world_with_spy();
    world.spawn((
        Panel {
            border_color: Some(Color::RED),
            border_width: 2.0,
        },
        UiNode {
            size: Vec2::new(200.0, 150.0),
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
