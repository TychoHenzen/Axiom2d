#![allow(clippy::unwrap_used)]

use bevy_ecs::prelude::{Schedule, World};
use engine_app::mouse_world_pos_system::mouse_world_pos_system;
use engine_app::window_size::WindowSize;
use engine_core::types::Pixels;
use engine_input::mouse::MouseState;
use engine_render::camera::Camera2D;
use glam::Vec2;

fn setup_world(screen_pos: Vec2, width: u32, height: u32) -> World {
    let mut world = World::new();
    let mut mouse = MouseState::default();
    mouse.set_screen_pos(screen_pos);
    world.insert_resource(mouse);
    world.insert_resource(WindowSize {
        width: Pixels(width as f32),
        height: Pixels(height as f32),
    });
    world
}

fn run_system(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(mouse_world_pos_system);
    schedule.run(world);
}

/// @doc: Mouse world position is derived from screen coordinates via the
/// camera's inverse projection. Every hit test (card pick, stash hover,
/// hand drop zone) reads `world_pos()` — a wrong conversion would cause
/// clicks to miss their targets by the camera offset.
#[test]
fn when_world_pos_system_runs_with_camera_then_world_pos_is_screen_to_world_converted() {
    // Arrange
    let mut world = setup_world(Vec2::new(400.0, 300.0), 800, 600);
    world.spawn(Camera2D::default());

    // Act
    run_system(&mut world);

    // Assert
    let mouse = world.resource::<MouseState>();
    assert_eq!(mouse.world_pos(), Vec2::new(0.0, 0.0));
}

#[test]
fn when_world_pos_system_runs_with_no_camera_then_uses_default_camera() {
    // Arrange
    let mut world = setup_world(Vec2::new(400.0, 300.0), 800, 600);

    // Act
    run_system(&mut world);

    // Assert
    let mouse = world.resource::<MouseState>();
    assert_eq!(mouse.world_pos(), Vec2::new(0.0, 0.0));
}

#[test]
fn when_world_pos_system_runs_with_zoomed_camera_then_center_still_maps_to_camera_pos() {
    // Arrange
    let mut world = setup_world(Vec2::new(400.0, 300.0), 800, 600);
    world.spawn(Camera2D {
        zoom: 2.0,
        ..Camera2D::default()
    });

    // Act
    run_system(&mut world);

    // Assert
    let mouse = world.resource::<MouseState>();
    assert_eq!(mouse.world_pos(), Vec2::new(0.0, 0.0));
}

/// @doc: Cursor offset from viewport center at zoom 2x should produce half
/// the world-space displacement (200px screen offset -> 100 world units).
/// If zoom weren't factored in, card picking would become increasingly
/// inaccurate as the player zooms in or out.
#[test]
fn when_world_pos_system_runs_with_offset_cursor_and_zoom_then_world_pos_is_scaled() {
    // Arrange
    let mut world = setup_world(Vec2::new(600.0, 300.0), 800, 600);
    world.spawn(Camera2D {
        zoom: 2.0,
        ..Camera2D::default()
    });

    // Act
    run_system(&mut world);

    // Assert
    let mouse = world.resource::<MouseState>();
    assert!((mouse.world_pos().x - 100.0).abs() < 1e-4);
    assert!(mouse.world_pos().y.abs() < 1e-4);
}
