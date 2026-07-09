#![allow(clippy::unwrap_used)]

use glam::Vec2;

use engine_input::action_map::ActionMap;
use engine_input::mouse::MouseButton;
use engine_input::mouse::MouseState;

/// @doc: Verifies that a pressed mouse button is reported as currently pressed.
#[test]
fn when_button_pressed_then_pressed_returns_true() {
    // Arrange
    let mut state = MouseState::default();

    // Act
    state.press(MouseButton::Left);

    // Assert
    assert!(state.pressed(MouseButton::Left), "Left button should be reported as pressed");
}

/// @doc: Verifies that a freshly pressed mouse button is reported as just pressed.
#[test]
fn when_button_pressed_then_just_pressed_returns_true() {
    // Arrange
    let mut state = MouseState::default();

    // Act
    state.press(MouseButton::Left);

    // Assert
    assert!(state.just_pressed(MouseButton::Left), "Left button should be just-pressed on first press");
}

/// @doc: Verifies that a pressed mouse button is not reported as just released.
#[test]
fn when_button_pressed_then_just_released_returns_false() {
    // Arrange
    let mut state = MouseState::default();

    // Act
    state.press(MouseButton::Left);

    // Assert
    assert!(!state.just_released(MouseButton::Left), "just-released should be false on initial press");
}

/// @doc: Verifies that a released mouse button is no longer reported as pressed.
#[test]
fn when_button_released_after_press_then_pressed_returns_false() {
    // Arrange
    let mut state = MouseState::default();
    state.press(MouseButton::Right);

    // Act
    state.release(MouseButton::Right);

    // Assert
    assert!(!state.pressed(MouseButton::Right), "released button should not be reported as pressed");
}

/// @doc: Verifies that a released mouse button is reported as just released.
#[test]
fn when_button_released_after_press_then_just_released_returns_true() {
    // Arrange
    let mut state = MouseState::default();
    state.press(MouseButton::Right);

    // Act
    state.release(MouseButton::Right);

    // Assert
    assert!(state.just_released(MouseButton::Right), "released button should be reported as just-released");
}

/// @doc: Verifies that just-pressed state for a held button is cleared on frame boundary while held state persists.
#[test]
fn when_frame_cleared_then_just_pressed_is_false_for_held_button() {
    // Arrange
    let mut state = MouseState::default();
    state.press(MouseButton::Left);

    // Act
    state.clear_frame_state();

    // Assert
    assert!(!state.just_pressed(MouseButton::Left), "just-pressed should be false after frame clear");
    assert!(
        state.pressed(MouseButton::Left),
        "held button should remain pressed after frame clear"
    );
}

/// @doc: Verifies that just-released state for a mouse button is cleared on frame boundary.
#[test]
fn when_frame_cleared_then_just_released_is_false() {
    // Arrange
    let mut state = MouseState::default();
    state.press(MouseButton::Left);
    state.release(MouseButton::Left);

    // Act
    state.clear_frame_state();

    // Assert
    assert!(!state.just_released(MouseButton::Left), "just-released should be cleared after frame clear");
}

/// @doc: Verifies that the screen position is updated after cursor movement.
#[test]
fn when_cursor_moved_then_screen_pos_is_updated() {
    // Arrange
    let mut state = MouseState::default();

    // Act
    state.set_screen_pos(Vec2::new(100.0, 200.0));

    // Assert
    assert_eq!(state.screen_pos(), Vec2::new(100.0, 200.0), "screen_pos should reflect set_screen_pos value");
}

/// @doc: Verifies that scroll delta reflects accumulated scroll input.
#[test]
fn when_scroll_accumulated_then_scroll_delta_reflects_total() {
    // Arrange
    let mut state = MouseState::default();

    // Act
    state.add_scroll_delta(Vec2::new(0.0, 3.0));

    // Assert
    assert_eq!(state.scroll_delta(), Vec2::new(0.0, 3.0), "scroll_delta should reflect accumulated scroll");
}

/// @doc: Verifies that scroll delta persists across frame clears (not cleared until explicitly consumed).
#[test]
fn when_frame_cleared_then_scroll_delta_is_preserved() {
    // Arrange
    let mut state = MouseState::default();
    state.add_scroll_delta(Vec2::new(2.0, 5.0));

    // Act
    state.clear_frame_state();

    // Assert
    assert_eq!(state.scroll_delta(), Vec2::new(2.0, 5.0), "scroll_delta should persist across frame clear");
}

/// @doc: Verifies that clear_scroll_delta resets the accumulated delta to zero.
#[test]
fn when_clear_scroll_delta_called_then_scroll_delta_is_zero() {
    // Arrange
    let mut state = MouseState::default();
    state.add_scroll_delta(Vec2::new(2.0, 5.0));

    // Act
    state.clear_scroll_delta();

    // Assert
    assert_eq!(state.scroll_delta(), Vec2::ZERO, "scroll_delta should be zero after clear_scroll_delta");
}

/// @doc: Verifies that multiple scroll events in one frame are summed in the delta.
#[test]
fn when_multiple_scroll_events_in_one_frame_then_delta_is_sum() {
    // Arrange
    let mut state = MouseState::default();

    // Act
    state.add_scroll_delta(Vec2::new(0.0, 1.0));
    state.add_scroll_delta(Vec2::new(0.0, 1.0));

    // Assert
    assert_eq!(state.scroll_delta(), Vec2::new(0.0, 2.0), "delta should accumulate across multiple scroll events");
}

/// @doc: Verifies that screen position persists across frame clears alongside button state.
#[test]
fn when_screen_pos_set_then_clear_frame_state_does_not_reset_it() {
    // Arrange
    let mut state = MouseState::default();
    state.set_screen_pos(Vec2::new(50.0, 75.0));

    // Act
    state.clear_frame_state();

    // Assert
    assert_eq!(state.screen_pos(), Vec2::new(50.0, 75.0), "screen_pos should not be reset by frame clear");
}

/// @doc: Verifies that world position can be set and read back from mouse state.
#[test]
fn when_world_pos_set_then_world_pos_is_readable() {
    // Arrange
    let mut state = MouseState::default();

    // Act
    state.set_world_pos(Vec2::new(300.0, -150.0));

    // Assert
    assert_eq!(state.world_pos(), Vec2::new(300.0, -150.0), "world_pos should reflect set_world_pos value");
}

/// @doc: Verifies that action_pressed returns true when the bound mouse button is pressed.
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
    assert!(result, "action_pressed should return true when bound button is pressed");
}

/// @doc: Verifies that action_pressed returns false when the bound mouse button is not pressed.
#[test]
fn when_action_bound_to_mouse_button_and_button_not_pressed_then_action_pressed_returns_false() {
    // Arrange
    let state = MouseState::default();
    let mut map = ActionMap::default();
    map.bind_mouse("fire", vec![MouseButton::Left]);

    // Act
    let result = state.action_pressed(&map, "fire");

    // Assert
    assert!(!result, "action_pressed should return false when no button is pressed");
}

/// @doc: Verifies that action_just_pressed returns true when the bound mouse button is just pressed.
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
    assert!(result, "action_just_pressed should return true for just-pressed bound button");
}

/// @doc: Verifies that action_just_pressed returns false for a held mouse button after frame clear.
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
    assert!(!result, "action_just_pressed should return false after frame clear even while held");
}

/// @doc: Verifies that action_pressed returns false for an unbound mouse action even when the button is pressed.
#[test]
fn when_unbound_mouse_action_queried_then_action_pressed_returns_false() {
    // Arrange
    let mut state = MouseState::default();
    state.press(MouseButton::Left);
    let map = ActionMap::default();

    // Act
    let result = state.action_pressed(&map, "fire");

    // Assert
    assert!(!result, "action_pressed should return false for unbound action");
}

/// @doc: Verifies that a zero scroll delta is a no-op on the accumulated delta.
#[test]
fn when_scroll_delta_zero_then_delta_is_unchanged() {
    // Arrange
    let mut state = MouseState::default();
    state.add_scroll_delta(Vec2::new(1.0, 2.0));

    // Act
    state.add_scroll_delta(Vec2::ZERO);

    // Assert
    assert_eq!(state.scroll_delta(), Vec2::new(1.0, 2.0), "zero delta should not change accumulated scroll");
}

/// @doc: Verifies that setting screen position to the zero vector (origin) is stored correctly.
#[test]
fn when_screen_pos_set_to_origin_then_screen_pos_is_zero() {
    // Arrange
    let mut state = MouseState::default();

    // Act
    state.set_screen_pos(Vec2::ZERO);

    // Assert
    assert_eq!(state.screen_pos(), Vec2::ZERO, "screen_pos at origin should be stored correctly");
}

/// @doc: Verifies that releasing a mouse button that was never pressed still sets just_released (by current impl).
#[test]
fn when_unpressed_button_released_then_just_released_is_set() {
    // Arrange
    let mut state = MouseState::default();

    // Act
    state.release(MouseButton::Middle);

    // Assert
    assert!(!state.pressed(MouseButton::Middle), "never-pressed button should not be reported as pressed");
    assert!(state.just_released(MouseButton::Middle), "just_released set even for never-pressed button");
}

/// @doc: Verifies that pressing and releasing two different buttons are tracked independently.
#[test]
fn when_two_different_buttons_used_independently_then_states_are_isolated() {
    // Arrange
    let mut state = MouseState::default();

    // Act
    state.press(MouseButton::Left);
    state.press(MouseButton::Right);
    state.release(MouseButton::Left);

    // Assert
    assert!(state.pressed(MouseButton::Right), "Right should still be pressed");
    assert!(!state.pressed(MouseButton::Left), "Left should be released");
    assert!(state.just_released(MouseButton::Left), "Left should be just_released");
}

/// @doc: Verifies that world position is preserved across frame clears (only button state resets).
#[test]
fn when_world_pos_set_then_clear_frame_state_does_not_reset_world_pos() {
    // Arrange
    let mut state = MouseState::default();
    state.set_world_pos(Vec2::new(42.0, 99.0));
    state.press(MouseButton::Left);

    // Act
    state.clear_frame_state();

    // Assert
    assert_eq!(state.world_pos(), Vec2::new(42.0, 99.0), "world_pos should persist across frame clear");
    assert!(!state.just_pressed(MouseButton::Left), "just_pressed should reset");
    assert!(state.pressed(MouseButton::Left), "pressed should persist");
}

/// @doc: Verifies that negative scroll delta values are tracked correctly by the accumulator.
#[test]
fn when_negative_scroll_delta_accumulated_then_delta_is_negative() {
    // Arrange
    let mut state = MouseState::default();

    // Act
    state.add_scroll_delta(Vec2::new(-10.0, -20.0));

    // Assert
    assert_eq!(state.scroll_delta(), Vec2::new(-10.0, -20.0), "negative scroll delta should be tracked correctly");
}

/// @doc: Verifies that combined scroll with horizontal and vertical components accumulates correctly.
#[test]
fn when_scroll_with_horizontal_and_vertical_then_delta_accumulates_both_axes() {
    // Arrange
    let mut state = MouseState::default();

    // Act
    state.add_scroll_delta(Vec2::new(5.0, -8.0));

    // Assert
    assert_eq!(state.scroll_delta(), Vec2::new(5.0, -8.0), "delta should accumulate both axes correctly");
}

/// @doc: Verifies that pressing the same button again in a new frame re-triggers just_pressed.
#[test]
fn when_button_pressed_cleared_then_pressed_again_then_just_pressed_is_true_again() {
    // Arrange
    let mut state = MouseState::default();
    state.press(MouseButton::Left);
    state.clear_frame_state();

    // Act: press again in next frame (simulating repeated OS press events for held key)
    state.press(MouseButton::Left);

    // Assert
    assert!(state.pressed(MouseButton::Left), "button should remain pressed");
    assert!(state.just_pressed(MouseButton::Left), "just_pressed should re-trigger on repeated press in new frame");
}

/// @doc: Verifies that rapid press-release-press cycles produce correct per-frame state transitions.
#[test]
fn when_rapid_press_release_repress_then_states_update_correctly() {
    // Arrange
    let mut state = MouseState::default();

    // Press once
    state.press(MouseButton::Left);
    assert!(state.just_pressed(MouseButton::Left), "first press: just_pressed should be true");
    assert!(state.pressed(MouseButton::Left), "first press: pressed should be true");

    state.clear_frame_state();

    // Release
    state.release(MouseButton::Left);
    assert!(!state.pressed(MouseButton::Left), "release: pressed should be false");
    assert!(state.just_released(MouseButton::Left), "release: just_released should be true");

    state.clear_frame_state();

    // Press again (second cycle)
    state.press(MouseButton::Left);
    assert!(state.just_pressed(MouseButton::Left), "re-press: just_pressed should be true again");
    assert!(state.pressed(MouseButton::Left), "re-press: pressed should be true");
}

/// @doc: Verifies that negative screen coordinates are stored correctly.
#[test]
fn when_negative_screen_pos_set_then_screen_pos_stored_correctly() {
    // Arrange
    let mut state = MouseState::default();

    // Act
    state.set_screen_pos(Vec2::new(-100.0, -200.0));

    // Assert
    assert_eq!(state.screen_pos(), Vec2::new(-100.0, -200.0), "negative screen_pos should be stored correctly");
}
