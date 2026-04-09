#![allow(clippy::unwrap_used, clippy::float_cmp)]

use std::sync::{Arc, Mutex};

use bevy_ecs::prelude::*;
use engine_render::prelude::{Camera2D, RendererRes};
use engine_render::testing::{ShapeCallLog, SpyRenderer};
use glam::Vec2;

use card_game::card::component::CardZone;
use card_game::card::interaction::drag_state::{DragInfo, DragState};
use card_game::card::rendering::drop_zone_glow::hand_drop_zone_render_system;

fn run_system(world: &mut World) {
    world.insert_resource(engine_ui::draw_command::DrawQueue::default());
    let mut schedule = Schedule::default();
    schedule.add_systems(hand_drop_zone_render_system);
    schedule.run(world);
    // Drain the DrawQueue through unified_render to produce spy draw calls
    let mut render_schedule = Schedule::default();
    render_schedule.add_systems(engine_ui::unified_render::unified_render_system);
    render_schedule.run(world);
}

fn make_world_with_spy(viewport_w: u32, viewport_h: u32) -> (World, ShapeCallLog) {
    let mut world = World::new();
    let log = Arc::new(Mutex::new(Vec::new()));
    let shape_calls: ShapeCallLog = Arc::new(Mutex::new(Vec::new()));
    let spy = SpyRenderer::new(log)
        .with_shape_capture(shape_calls.clone())
        .with_viewport(viewport_w, viewport_h);
    world.insert_resource(RendererRes::new(Box::new(spy)));
    world.spawn(Camera2D {
        position: Vec2::ZERO,
        zoom: 1.0,
    });
    (world, shape_calls)
}

fn make_drag_state(world: &mut World) {
    let entity = world.spawn_empty().id();
    world.insert_resource(DragState {
        dragging: Some(DragInfo {
            entity,
            local_grab_offset: Vec2::ZERO,
            origin_zone: CardZone::Table,
            stash_cursor_follow: false,
            origin_position: Vec2::ZERO,
        }),
    });
}

/// @doc: Hand drop zone glow must render only during an active drag (`DragState::dragging` is Some).
/// The glow provides visual feedback for valid drop targets, so it must appear/disappear based on drag state and be translucent.
#[test]
fn when_drag_active_then_glow_rect_drawn() {
    // Arrange
    let (mut world, shape_calls) = make_world_with_spy(800, 600);
    make_drag_state(&mut world);

    // Act
    run_system(&mut world);

    // Assert
    let calls = shape_calls.lock().unwrap();
    assert_eq!(calls.len(), 1, "exactly one shape call for glow rect");
    assert!(
        calls[0].2.a < 1.0,
        "glow rect should be translucent, got alpha={}",
        calls[0].2.a
    );
}

/// @doc: When `DragState` has no active drag, the glow rect must not render. This prevents unnecessary draw calls
/// and visual clutter when cards are at rest. The system early-exits to avoid camera/viewport lookups.
#[test]
fn when_no_drag_then_no_glow_rect_drawn() {
    // Arrange
    let (mut world, shape_calls) = make_world_with_spy(800, 600);
    world.insert_resource(DragState::default());

    // Act
    run_system(&mut world);

    // Assert
    assert!(shape_calls.lock().unwrap().is_empty());
}

/// @doc: Edge case: zero viewport dimensions (e.g., window minimized) must not crash the system.
/// Early exit in `resolve_viewport_camera` prevents division by zero and invalid screen-to-world conversions.
#[test]
fn when_viewport_zero_then_no_glow_rect_and_no_panic() {
    // Arrange
    let (mut world, shape_calls) = make_world_with_spy(0, 0);
    make_drag_state(&mut world);

    // Act
    run_system(&mut world);

    // Assert
    assert!(shape_calls.lock().unwrap().is_empty());
}
