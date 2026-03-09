use std::collections::HashSet;

use bevy_ecs::prelude::Resource;
use winit::keyboard::KeyCode;

use crate::action_map::ActionMap;

#[derive(Resource, Debug, Clone, PartialEq, Default)]
pub struct InputState {
    pressed: HashSet<KeyCode>,
    just_pressed: HashSet<KeyCode>,
    just_released: HashSet<KeyCode>,
}

impl InputState {
    pub fn pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key)
    }

    pub fn just_pressed(&self, key: KeyCode) -> bool {
        self.just_pressed.contains(&key)
    }

    pub fn just_released(&self, key: KeyCode) -> bool {
        self.just_released.contains(&key)
    }

    pub fn press(&mut self, key: KeyCode) {
        self.pressed.insert(key);
        self.just_pressed.insert(key);
    }

    pub fn release(&mut self, key: KeyCode) {
        self.pressed.remove(&key);
        self.just_released.insert(key);
    }

    pub fn action_pressed(&self, map: &ActionMap, action: &str) -> bool {
        map.bindings_for(action).iter().any(|key| self.pressed(*key))
    }

    pub fn action_just_pressed(&self, map: &ActionMap, action: &str) -> bool {
        map.bindings_for(action)
            .iter()
            .any(|key| self.just_pressed(*key))
    }

    pub fn clear_frame_state(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_input_state_default_then_no_keys_are_pressed() {
        // Arrange
        let state = InputState::default();

        // Act
        let result = state.pressed(KeyCode::Space);

        // Assert
        assert!(!result);
    }

    #[test]
    fn when_key_pressed_then_pressed_returns_true() {
        // Arrange
        let mut state = InputState::default();

        // Act
        state.press(KeyCode::ArrowRight);

        // Assert
        assert!(state.pressed(KeyCode::ArrowRight));
    }

    #[test]
    fn when_key_pressed_then_just_pressed_returns_true() {
        // Arrange
        let mut state = InputState::default();

        // Act
        state.press(KeyCode::ArrowRight);

        // Assert
        assert!(state.just_pressed(KeyCode::ArrowRight));
    }

    #[test]
    fn when_key_pressed_then_just_released_returns_false() {
        // Arrange
        let mut state = InputState::default();

        // Act
        state.press(KeyCode::ArrowRight);

        // Assert
        assert!(!state.just_released(KeyCode::ArrowRight));
    }

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
}
