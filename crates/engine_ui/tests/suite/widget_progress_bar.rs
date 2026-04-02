#![allow(clippy::unwrap_used)]

use std::sync::{Arc, Mutex};

use bevy_ecs::prelude::{Schedule, World};
use engine_core::prelude::{Color, Pixels};
use engine_render::prelude::RendererRes;
use engine_render::testing::{RectCallLog, SpyRenderer};
use engine_scene::prelude::{EffectiveVisibility, GlobalTransform2D};
use engine_ui::layout::Anchor;
use engine_ui::theme::UiTheme;
use engine_ui::widget::{ProgressBar, UiNode, progress_bar_render_system};
use glam::{Affine2, Vec2};

fn setup_world_with_spy() -> (World, Schedule, Arc<Mutex<Vec<String>>>, RectCallLog) {
    let mut world = World::new();
    let log = Arc::new(Mutex::new(Vec::new()));
    let rect_cap: RectCallLog = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(Arc::clone(&log)).with_rect_capture(Arc::clone(&rect_cap));
    world.insert_resource(RendererRes::new(Box::new(spy)));
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
fn when_progress_bar_at_zero_then_no_fill_drawn() {
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

    // Assert — background is handled by ui_render_system, not here
    let calls = log.lock().unwrap();
    assert_eq!(calls.iter().filter(|c| *c == "draw_rect").count(), 0);
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
    assert_eq!(rects.len(), 1);
    assert_eq!(rects[0].width, Pixels(100.0));
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
    assert_eq!(rects.len(), 1);
    assert_eq!(rects[0].width, Pixels(200.0));
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
    assert_eq!(rects.len(), 1);
    assert_eq!(rects[0].width, Pixels(200.0));
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
    assert_eq!(rects.len(), 1);
    assert_eq!(rects[0].x, Pixels(200.0));
    assert_eq!(rects[0].y, Pixels(90.0));
    assert_eq!(rects[0].width, Pixels(100.0));
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
