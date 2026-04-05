#![allow(clippy::unwrap_used, clippy::float_cmp)]

use bevy_ecs::prelude::*;
use card_game::card::interaction::camera_drag::{
    CameraDragState, ZOOM_MIN, camera_drag_system, camera_zoom_system,
};
use engine_input::prelude::{MouseButton, MouseState};
use engine_render::prelude::Camera2D;
use glam::Vec2;

fn run_system(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(camera_drag_system);
    schedule.run(world);
}

#[test]
fn when_rmb_not_pressed_then_drag_state_anchor_remains_none() {
    // Arrange
    let mut world = World::new();
    world.spawn(Camera2D::default());
    world.insert_resource(MouseState::default());
    world.insert_resource(CameraDragState::default());

    // Act
    run_system(&mut world);

    // Assert
    assert_eq!(world.resource::<CameraDragState>().anchor_screen_pos, None);
}

#[test]
fn when_rmb_just_pressed_then_drag_state_anchor_set_to_screen_pos() {
    // Arrange
    let mut world = World::new();
    world.spawn(Camera2D::default());
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Right);
    mouse.set_screen_pos(Vec2::new(100.0, 200.0));
    world.insert_resource(mouse);
    world.insert_resource(CameraDragState::default());

    // Act
    run_system(&mut world);

    // Assert
    assert_eq!(
        world.resource::<CameraDragState>().anchor_screen_pos,
        Some(Vec2::new(100.0, 200.0)),
    );
}


/// @doc: Drag delta inverted for camera movement—moving mouse right pans camera left
#[test]
fn when_rmb_held_and_mouse_moved_then_camera_moves_inversely() {
    // Arrange
    let mut world = World::new();
    world.spawn(Camera2D::default());
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Right);
    mouse.clear_frame_state();
    mouse.set_screen_pos(Vec2::new(110.0, 220.0));
    world.insert_resource(mouse);
    world.insert_resource(CameraDragState {
        anchor_screen_pos: Some(Vec2::new(100.0, 200.0)),
    });

    // Act
    run_system(&mut world);

    // Assert
    let camera = world.query::<&Camera2D>().single(&world).unwrap();
    assert_eq!(camera.position, Vec2::new(-10.0, -20.0));
}

#[test]
fn when_rmb_held_then_anchor_updated_to_current_screen_pos() {
    // Arrange
    let mut world = World::new();
    world.spawn(Camera2D::default());
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Right);
    mouse.clear_frame_state();
    mouse.set_screen_pos(Vec2::new(110.0, 220.0));
    world.insert_resource(mouse);
    world.insert_resource(CameraDragState {
        anchor_screen_pos: Some(Vec2::new(100.0, 200.0)),
    });

    // Act
    run_system(&mut world);

    // Assert
    assert_eq!(
        world.resource::<CameraDragState>().anchor_screen_pos,
        Some(Vec2::new(110.0, 220.0)),
    );
}

/// @doc: Screen drag scaled by zoom reciprocal—zoomed in makes same motion move camera less
#[test]
fn when_zoomed_in_then_same_screen_drag_moves_camera_less() {
    // Arrange
    let mut world = World::new();
    world.spawn(Camera2D {
        position: Vec2::ZERO,
        zoom: 2.0,
    });
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Right);
    mouse.clear_frame_state();
    mouse.set_screen_pos(Vec2::new(110.0, 100.0));
    world.insert_resource(mouse);
    world.insert_resource(CameraDragState {
        anchor_screen_pos: Some(Vec2::new(100.0, 100.0)),
    });

    // Act
    run_system(&mut world);

    // Assert
    let camera = world.query::<&Camera2D>().single(&world).unwrap();
    assert_eq!(camera.position.x, -5.0);
}

#[test]
fn when_rmb_released_then_drag_state_becomes_none() {
    // Arrange
    let mut world = World::new();
    world.spawn(Camera2D {
        position: Vec2::new(50.0, 50.0),
        zoom: 1.0,
    });
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Right);
    mouse.release(MouseButton::Right);
    world.insert_resource(mouse);
    world.insert_resource(CameraDragState {
        anchor_screen_pos: Some(Vec2::new(100.0, 200.0)),
    });

    // Act
    run_system(&mut world);

    // Assert
    assert_eq!(world.resource::<CameraDragState>().anchor_screen_pos, None);
}

#[test]
fn when_rmb_released_then_camera_position_unchanged() {
    // Arrange
    let mut world = World::new();
    world.spawn(Camera2D {
        position: Vec2::new(50.0, 50.0),
        zoom: 1.0,
    });
    let mut mouse = MouseState::default();
    mouse.press(MouseButton::Right);
    mouse.release(MouseButton::Right);
    mouse.set_screen_pos(Vec2::new(110.0, 210.0));
    world.insert_resource(mouse);
    world.insert_resource(CameraDragState {
        anchor_screen_pos: Some(Vec2::new(100.0, 200.0)),
    });

    // Act
    run_system(&mut world);

    // Assert
    let camera = world.query::<&Camera2D>().single(&world).unwrap();
    assert_eq!(camera.position, Vec2::new(50.0, 50.0));
}


fn run_zoom_system(world: &mut World) {
    let mut schedule = Schedule::default();
    schedule.add_systems(camera_zoom_system);
    schedule.run(world);
}

#[test]
fn when_scroll_up_then_zoom_increases() {
    // Arrange
    let mut world = World::new();
    world.spawn(Camera2D::default());
    let mut mouse = MouseState::default();
    mouse.add_scroll_delta(Vec2::new(0.0, 1.0));
    world.insert_resource(mouse);

    // Act
    run_zoom_system(&mut world);

    // Assert
    let camera = world.query::<&Camera2D>().single(&world).unwrap();
    assert!(camera.zoom > 1.0);
}

#[test]
fn when_scroll_down_then_zoom_decreases() {
    // Arrange
    let mut world = World::new();
    world.spawn(Camera2D::default());
    let mut mouse = MouseState::default();
    mouse.add_scroll_delta(Vec2::new(0.0, -1.0));
    world.insert_resource(mouse);

    // Act
    run_zoom_system(&mut world);

    // Assert
    let camera = world.query::<&Camera2D>().single(&world).unwrap();
    assert!(camera.zoom < 1.0);
}

#[test]
fn when_zoom_at_floor_and_scroll_down_then_zoom_stays_at_minimum() {
    // Arrange
    let mut world = World::new();
    world.spawn(Camera2D {
        position: Vec2::ZERO,
        zoom: ZOOM_MIN,
    });
    let mut mouse = MouseState::default();
    mouse.add_scroll_delta(Vec2::new(0.0, -5.0));
    world.insert_resource(mouse);

    // Act
    run_zoom_system(&mut world);

    // Assert
    let camera = world.query::<&Camera2D>().single(&world).unwrap();
    assert!(camera.zoom >= ZOOM_MIN);
}

/// @doc: Zoom accumulates linearly by scroll delta × speed constant—controls zoom sensitivity
#[test]
fn when_scroll_by_two_then_zoom_equals_initial_plus_speed_times_delta() {
    // Arrange
    let mut world = World::new();
    world.spawn(Camera2D {
        position: Vec2::ZERO,
        zoom: 1.0,
    });
    let mut mouse = MouseState::default();
    mouse.add_scroll_delta(Vec2::new(0.0, 2.0));
    world.insert_resource(mouse);

    // Act
    run_zoom_system(&mut world);

    // Assert
    let camera = world.query::<&Camera2D>().single(&world).unwrap();
    assert_eq!(camera.zoom, 1.2);
}

