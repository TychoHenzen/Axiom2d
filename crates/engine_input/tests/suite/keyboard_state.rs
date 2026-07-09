#![allow(clippy::unwrap_used)]

use engine_input::action_map::ActionMap;
use engine_input::key_code::KeyCode;
use engine_input::keyboard::InputState;

/// @doc: Verifies that a pressed key is reported as currently pressed.
#[test]
fn when_key_pressed_then_pressed_returns_true() {
    // Arrange
    let mut state = InputState::default();

    // Act
    state.press(KeyCode::ArrowRight);

    // Assert
    assert!(state.pressed(KeyCode::ArrowRight));
}

/// @doc: Verifies that a freshly pressed key is reported as just pressed.
#[test]
fn when_key_pressed_then_just_pressed_returns_true() {
    // Arrange
    let mut state = InputState::default();

    // Act
    state.press(KeyCode::ArrowRight);

    // Assert
    assert!(state.just_pressed(KeyCode::ArrowRight));
}

/// @doc: Verifies that a pressed key is not reported as just released.
#[test]
fn when_key_pressed_then_just_released_returns_false() {
    // Arrange
    let mut state = InputState::default();

    // Act
    state.press(KeyCode::ArrowRight);

    // Assert
    assert!(!state.just_released(KeyCode::ArrowRight));
}

/// @doc: Verifies that a released key is no longer reported as pressed.
#[test]
fn when_key_released_after_press_then_pressed_returns_false() {
    // Arrange
    let mut state = InputState::default();
    state.press(KeyCode::Space);

    // Act
    state.release(KeyCode::Space);

    // Assert
    assert!(!state.pressed(KeyCode::Space));
}

/// @doc: Verifies that a released key is reported as just released.
#[test]
fn when_key_released_after_press_then_just_released_returns_true() {
    // Arrange
    let mut state = InputState::default();
    state.press(KeyCode::Space);

    // Act
    state.release(KeyCode::Space);

    // Assert
    assert!(state.just_released(KeyCode::Space));
}

/// @doc: Verifies that just-pressed state for a held key is cleared on frame boundary.
#[test]
fn when_frame_cleared_then_just_pressed_is_false_for_held_key() {
    // Arrange
    let mut state = InputState::default();
    state.press(KeyCode::ArrowUp);

    // Act
    state.clear_frame_state();

    // Assert
    assert!(!state.just_pressed(KeyCode::ArrowUp));
}

/// @doc: Verifies that just-released state is cleared on frame boundary.
#[test]
fn when_frame_cleared_then_just_released_is_false() {
    // Arrange
    let mut state = InputState::default();
    state.press(KeyCode::Space);
    state.release(KeyCode::Space);

    // Act
    state.clear_frame_state();

    // Assert
    assert!(!state.just_released(KeyCode::Space));
}

/// @doc: Verifies that action_pressed returns false when the action is not bound in the map.
#[test]
fn when_action_not_in_map_then_action_pressed_returns_false() {
    // Arrange
    let mut state = InputState::default();
    state.press(KeyCode::Space);
    let map = ActionMap::default();

    // Act
    let result = state.action_pressed(&map, "jump");

    // Assert
    assert!(!result);
}

/// @doc: Verifies that action_pressed returns false when the bound key is not pressed.
#[test]
fn when_bound_key_is_not_pressed_then_action_pressed_returns_false() {
    // Arrange
    let state = InputState::default();
    let mut map = ActionMap::default();
    map.bind("jump", vec![KeyCode::Space]);

    // Act
    let result = state.action_pressed(&map, "jump");

    // Assert
    assert!(!result);
}

/// @doc: Verifies that action_pressed returns true when the bound key is pressed.
#[test]
fn when_bound_key_is_pressed_then_action_pressed_returns_true() {
    // Arrange
    let mut state = InputState::default();
    state.press(KeyCode::Space);
    let mut map = ActionMap::default();
    map.bind("jump", vec![KeyCode::Space]);

    // Act
    let result = state.action_pressed(&map, "jump");

    // Assert
    assert!(result);
}

/// @doc: Verifies that action_just_pressed returns false when the action is not bound in the map.
#[test]
fn when_action_not_in_map_then_action_just_pressed_returns_false() {
    // Arrange
    let mut state = InputState::default();
    state.press(KeyCode::Space);
    let map = ActionMap::default();

    // Act
    let result = state.action_just_pressed(&map, "jump");

    // Assert
    assert!(!result);
}

/// @doc: Verifies that action_just_pressed returns true when the bound key is just pressed.
#[test]
fn when_bound_key_is_just_pressed_then_action_just_pressed_returns_true() {
    // Arrange
    let mut state = InputState::default();
    state.press(KeyCode::Space);
    let mut map = ActionMap::default();
    map.bind("jump", vec![KeyCode::Space]);

    // Act
    let result = state.action_just_pressed(&map, "jump");

    // Assert
    assert!(result);
}

/// @doc: Verifies that action_just_pressed returns false for a held key after frame clear.
#[test]
fn when_bound_key_held_across_frame_clear_then_action_just_pressed_returns_false() {
    // Arrange
    let mut state = InputState::default();
    state.press(KeyCode::Space);
    state.clear_frame_state();
    let mut map = ActionMap::default();
    map.bind("jump", vec![KeyCode::Space]);

    // Act
    let result = state.action_just_pressed(&map, "jump");

    // Assert
    assert!(!result);
}

/// @doc: Verifies that action_just_pressed returns true when any one of multiple bound keys is just pressed.
#[test]
fn when_one_of_multiple_bound_keys_is_just_pressed_then_action_just_pressed_returns_true() {
    // Arrange
    let mut state = InputState::default();
    state.press(KeyCode::KeyD);
    let mut map = ActionMap::default();
    map.bind("move_right", vec![KeyCode::ArrowRight, KeyCode::KeyD]);

    // Act
    let result = state.action_just_pressed(&map, "move_right");

    // Assert
    assert!(result);
}

/// @doc: Verifies that action_pressed returns true when any one of multiple bound keys is pressed.
#[test]
fn when_one_of_multiple_bound_keys_is_pressed_then_action_pressed_returns_true() {
    // Arrange
    let mut state = InputState::default();
    state.press(KeyCode::KeyD);
    let mut map = ActionMap::default();
    map.bind("move_right", vec![KeyCode::ArrowRight, KeyCode::KeyD]);

    // Act
    let result = state.action_pressed(&map, "move_right");

    // Assert
    assert!(result);
}

/// @doc: Verifies that a held key remains pressed across frame clears.
#[test]
fn when_frame_cleared_then_held_key_stays_pressed() {
    // Arrange
    let mut state = InputState::default();
    state.press(KeyCode::ArrowLeft);

    // Act
    state.clear_frame_state();

    // Assert
    assert!(state.pressed(KeyCode::ArrowLeft));
}
