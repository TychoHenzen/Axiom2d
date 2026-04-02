#![allow(clippy::unwrap_used)]

use glam::Vec2;

use engine_input::action_map::ActionMap;
use engine_input::mouse::MouseButton;
use engine_input::mouse::MouseState;

#[test]
fn when_button_pressed_then_pressed_returns_true() {
    // Arrange
    let mut state = MouseState::default();

    // Act
    state.press(MouseButton::Left);

    // Assert
    assert!(state.pressed(MouseButton::Left));
}

#[test]
fn when_button_pressed_then_just_pressed_returns_true() {
    // Arrange
    let mut state = MouseState::default();

    // Act
    state.press(MouseButton::Left);

    // Assert
    assert!(state.just_pressed(MouseButton::Left));
}

#[test]
fn when_button_pressed_then_just_released_returns_false() {
    // Arrange
    let mut state = MouseState::default();

    // Act
    state.press(MouseButton::Left);

    // Assert
    assert!(!state.just_released(MouseButton::Left));
}

#[test]
fn when_button_released_after_press_then_pressed_returns_false() {
    // Arrange
    let mut state = MouseState::default();
    state.press(MouseButton::Right);

    // Act
    state.release(MouseButton::Right);

    // Assert
    assert!(!state.pressed(MouseButton::Right));
}

#[test]
fn when_button_released_after_press_then_just_released_returns_true() {
    // Arrange
    let mut state = MouseState::default();
    state.press(MouseButton::Right);

    // Act
    state.release(MouseButton::Right);

    // Assert
    assert!(state.just_released(MouseButton::Right));
}

#[test]
fn when_frame_cleared_then_just_pressed_is_false_for_held_button() {
    // Arrange
    let mut state = MouseState::default();
    state.press(MouseButton::Left);

    // Act
    state.clear_frame_state();

    // Assert
    assert!(!state.just_pressed(MouseButton::Left));
    assert!(state.pressed(MouseButton::Left));
}

#[test]
fn when_frame_cleared_then_just_released_is_false() {
    // Arrange
    let mut state = MouseState::default();
    state.press(MouseButton::Left);
    state.release(MouseButton::Left);

    // Act
    state.clear_frame_state();

    // Assert
    assert!(!state.just_released(MouseButton::Left));
}

#[test]
fn when_cursor_moved_then_screen_pos_is_updated() {
    // Arrange
    let mut state = MouseState::default();

    // Act
    state.set_screen_pos(Vec2::new(100.0, 200.0));

    // Assert
    assert_eq!(state.screen_pos(), Vec2::new(100.0, 200.0));
}

#[test]
fn when_scroll_accumulated_then_scroll_delta_reflects_total() {
    // Arrange
    let mut state = MouseState::default();

    // Act
    state.add_scroll_delta(Vec2::new(0.0, 3.0));

    // Assert
    assert_eq!(state.scroll_delta(), Vec2::new(0.0, 3.0));
}

#[test]
fn when_frame_cleared_then_scroll_delta_is_preserved() {
    // Arrange
    let mut state = MouseState::default();
    state.add_scroll_delta(Vec2::new(2.0, 5.0));

    // Act
    state.clear_frame_state();

    // Assert
    assert_eq!(state.scroll_delta(), Vec2::new(2.0, 5.0));
}

#[test]
fn when_clear_scroll_delta_called_then_scroll_delta_is_zero() {
    // Arrange
    let mut state = MouseState::default();
    state.add_scroll_delta(Vec2::new(2.0, 5.0));

    // Act
    state.clear_scroll_delta();

    // Assert
    assert_eq!(state.scroll_delta(), Vec2::ZERO);
}

#[test]
fn when_multiple_scroll_events_in_one_frame_then_delta_is_sum() {
    // Arrange
    let mut state = MouseState::default();

    // Act
    state.add_scroll_delta(Vec2::new(0.0, 1.0));
    state.add_scroll_delta(Vec2::new(0.0, 1.0));

    // Assert
    assert_eq!(state.scroll_delta(), Vec2::new(0.0, 2.0));
}

#[test]
fn when_screen_pos_set_then_clear_frame_state_does_not_reset_it() {
    // Arrange
    let mut state = MouseState::default();
    state.set_screen_pos(Vec2::new(50.0, 75.0));

    // Act
    state.clear_frame_state();

    // Assert
    assert_eq!(state.screen_pos(), Vec2::new(50.0, 75.0));
}

#[test]
fn when_world_pos_set_then_world_pos_is_readable() {
    // Arrange
    let mut state = MouseState::default();

    // Act
    state.set_world_pos(Vec2::new(300.0, -150.0));

    // Assert
    assert_eq!(state.world_pos(), Vec2::new(300.0, -150.0));
}

#[test]
fn when_action_bound_to_mouse_button_and_button_pressed_then_action_pressed_returns_true() {
    // Arrange
    let mut state = MouseState::default();
    state.press(MouseButton::Left);
    let mut map = ActionMap::default();
    map.bind_mouse("fire", vec![MouseButton::Left]);

    // Act
    let result = state.action_pressed(&map, "fire");

    // Assert
    assert!(result);
}

#[test]
fn when_action_bound_to_mouse_button_and_button_not_pressed_then_action_pressed_returns_false() {
    // Arrange
    let state = MouseState::default();
    let mut map = ActionMap::default();
    map.bind_mouse("fire", vec![MouseButton::Left]);

    // Act
    let result = state.action_pressed(&map, "fire");

    // Assert
    assert!(!result);
}

#[test]
fn when_action_bound_to_mouse_button_and_button_just_pressed_then_action_just_pressed_returns_true()
{
    // Arrange
    let mut state = MouseState::default();
    state.press(MouseButton::Left);
    let mut map = ActionMap::default();
    map.bind_mouse("fire", vec![MouseButton::Left]);

    // Act
    let result = state.action_just_pressed(&map, "fire");

    // Assert
    assert!(result);
}

#[test]
fn when_button_held_but_frame_cleared_then_action_just_pressed_returns_false() {
    // Arrange
    let mut state = MouseState::default();
    state.press(MouseButton::Left);
    state.clear_frame_state();
    let mut map = ActionMap::default();
    map.bind_mouse("fire", vec![MouseButton::Left]);

    // Act
    let result = state.action_just_pressed(&map, "fire");

    // Assert
    assert!(!result);
}

#[test]
fn when_unbound_mouse_action_queried_then_action_pressed_returns_false() {
    // Arrange
    let mut state = MouseState::default();
    state.press(MouseButton::Left);
    let map = ActionMap::default();

    // Act
    let result = state.action_pressed(&map, "fire");

    // Assert
    assert!(!result);
}
