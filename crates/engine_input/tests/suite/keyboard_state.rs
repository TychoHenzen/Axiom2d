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
    assert!(
        state.pressed(KeyCode::ArrowRight),
        "pressed should return true after press() is called"
    );
}

/// @doc: Verifies that a freshly pressed key is reported as just pressed.
#[test]
fn when_key_pressed_then_just_pressed_returns_true() {
    // Arrange
    let mut state = InputState::default();

    // Act
    state.press(KeyCode::ArrowRight);

    // Assert
    assert!(
        state.just_pressed(KeyCode::ArrowRight),
        "just_pressed should return true after press() is called"
    );
}

/// @doc: Verifies that a pressed key is not reported as just released.
#[test]
fn when_key_pressed_then_just_released_returns_false() {
    // Arrange
    let mut state = InputState::default();

    // Act
    state.press(KeyCode::ArrowRight);

    // Assert
    assert!(
        !state.just_released(KeyCode::ArrowRight),
        "just_released should be false for a key that was pressed, not released"
    );
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
    assert!(
        !state.pressed(KeyCode::Space),
        "pressed should return false after release() is called"
    );
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
    assert!(
        state.just_released(KeyCode::Space),
        "just_released should return true after release() is called"
    );
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
    assert!(
        !state.just_pressed(KeyCode::ArrowUp),
        "just_pressed should be false after clear_frame_state() for a held key"
    );
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
    assert!(
        !state.just_released(KeyCode::Space),
        "just_released should be false after clear_frame_state()"
    );
}

/// @doc: Verifies that `action_pressed` returns false when the action is not bound in the map.
#[test]
fn when_action_not_in_map_then_action_pressed_returns_false() {
    // Arrange
    let mut state = InputState::default();
    state.press(KeyCode::Space);
    let map = ActionMap::default();

    // Act
    let result = state.action_pressed(&map, "jump");

    // Assert
    assert!(
        !result,
        "action_pressed should return false when the action is not bound in the map"
    );
}

/// @doc: Verifies that `action_pressed` returns false when the bound key is not pressed.
#[test]
fn when_bound_key_is_not_pressed_then_action_pressed_returns_false() {
    // Arrange
    let state = InputState::default();
    let mut map = ActionMap::default();
    map.bind("jump", vec![KeyCode::Space]);

    // Act
    let result = state.action_pressed(&map, "jump");

    // Assert
    assert!(
        !result,
        "action_pressed should return false when the bound key is not pressed"
    );
}

/// @doc: Verifies that `action_pressed` returns true when the bound key is pressed.
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
    assert!(
        result,
        "action_pressed should return true when the bound key is pressed"
    );
}

/// @doc: Verifies that `action_just_pressed` returns false when the action is not bound in the map.
#[test]
fn when_action_not_in_map_then_action_just_pressed_returns_false() {
    // Arrange
    let mut state = InputState::default();
    state.press(KeyCode::Space);
    let map = ActionMap::default();

    // Act
    let result = state.action_just_pressed(&map, "jump");

    // Assert
    assert!(
        !result,
        "action_just_pressed should return false when the action is not bound"
    );
}

/// @doc: Verifies that `action_just_pressed` returns true when the bound key is just pressed.
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
    assert!(
        result,
        "action_just_pressed should return true when the bound key is just pressed"
    );
}

/// @doc: Verifies that `action_just_pressed` returns false for a held key after frame clear.
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
    assert!(
        !result,
        "action_just_pressed should return false for a held key after frame clear"
    );
}

/// @doc: Verifies that `action_just_pressed` returns true when any one of multiple bound keys is just pressed.
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
    assert!(
        result,
        "action_just_pressed should return true when one of multiple bound keys is just pressed"
    );
}

/// @doc: Verifies that `action_pressed` returns true when any one of multiple bound keys is pressed.
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
    assert!(
        result,
        "action_pressed should return true when one of multiple bound keys is pressed"
    );
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
    assert!(
        state.pressed(KeyCode::ArrowLeft),
        "held key should remain pressed after clear_frame_state()"
    );
}

/// @doc: Verifies that clearing frame state on a fresh default `InputState` is a no-op.
#[test]
fn when_frame_cleared_on_empty_state_then_state_remains_empty() {
    // Arrange
    let mut state = InputState::default();

    // Act
    state.clear_frame_state();

    // Assert
    assert!(!state.pressed(KeyCode::Space));
    assert!(!state.just_pressed(KeyCode::Space));
    assert!(!state.just_released(KeyCode::Space));
}

/// @doc: Verifies that pressing the same key twice is idempotent (pressed and `just_pressed` remain true).
#[test]
fn when_same_key_pressed_twice_then_state_is_still_pressed() {
    // Arrange
    let mut state = InputState::default();

    // Act
    state.press(KeyCode::ArrowRight);
    state.press(KeyCode::ArrowRight);

    // Assert
    assert!(
        state.pressed(KeyCode::ArrowRight),
        "key should remain pressed after duplicate press"
    );
    assert!(
        state.just_pressed(KeyCode::ArrowRight),
        "just_pressed should remain true after duplicate press"
    );
}

/// @doc: Verifies that `just_released` returns false for a key that was only pressed (not released).
#[test]
fn when_key_is_held_then_just_released_returns_false() {
    // Arrange
    let mut state = InputState::default();
    state.press(KeyCode::Space);

    // Act
    let released = state.just_released(KeyCode::Space);

    // Assert
    assert!(
        !released,
        "just_released should return false for a held key that was not released"
    );
}

/// @doc: Verifies that `just_released` transitions from true to false after frame clear.
#[test]
fn when_key_released_and_frame_cleared_then_just_released_returns_false() {
    // Arrange
    let mut state = InputState::default();
    state.press(KeyCode::Space);
    state.release(KeyCode::Space);
    assert!(
        state.just_released(KeyCode::Space),
        "sanity: just_released true after release"
    );

    // Act
    state.clear_frame_state();

    // Assert
    assert!(
        !state.just_released(KeyCode::Space),
        "just_released should be false after frame clear"
    );
}

/// @doc: Verifies that `just_released` is true when a key is released without ever having been pressed.
#[test]
fn when_never_pressed_key_is_released_then_just_released_is_set() {
    // Arrange
    let mut state = InputState::default();

    // Act
    state.release(KeyCode::KeyF);

    // Assert
    assert!(
        state.just_released(KeyCode::KeyF),
        "release on never-pressed key sets just_released by current impl"
    );
}

/// @doc: Verifies that releasing a key that was never pressed marks it as `just_released` but pressed stays false.
#[test]
fn when_unpressed_key_released_then_pressed_stays_false_and_just_released_is_set() {
    // Arrange
    let mut state = InputState::default();

    // Act
    state.release(KeyCode::KeyE);

    // Assert
    assert!(!state.pressed(KeyCode::KeyE));
    assert!(
        !state.just_pressed(KeyCode::KeyE),
        "release should not produce just_pressed"
    );
    assert!(
        state.just_released(KeyCode::KeyE),
        "release() on unpressed key sets just_released by current implementation"
    );
}
