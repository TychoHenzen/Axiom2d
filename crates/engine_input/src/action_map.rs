use std::collections::HashMap;

use bevy_ecs::prelude::Resource;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

#[derive(Resource, Debug, Clone, Default)]
pub struct ActionMap {
    bindings: HashMap<String, Vec<KeyCode>>,
    mouse_bindings: HashMap<String, Vec<MouseButton>>,
}

impl ActionMap {
    pub fn bind(&mut self, action: &str, keys: Vec<KeyCode>) {
        self.bindings.insert(action.to_string(), keys);
    }

    pub fn bindings_for(&self, action: &str) -> &[KeyCode] {
        self.bindings.get(action).map_or(&[], Vec::as_slice)
    }

    pub fn bind_mouse(&mut self, action: &str, buttons: Vec<MouseButton>) {
        self.mouse_bindings.insert(action.to_string(), buttons);
    }

    pub fn mouse_bindings_for(&self, action: &str) -> &[MouseButton] {
        self.mouse_bindings.get(action).map_or(&[], Vec::as_slice)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_multiple_keys_bound_to_same_action_then_all_keys_returned() {
        // Arrange
        let mut map = ActionMap::default();

        // Act
        map.bind("move_right", vec![KeyCode::ArrowRight, KeyCode::KeyD]);

        // Assert
        assert_eq!(
            map.bindings_for("move_right"),
            &[KeyCode::ArrowRight, KeyCode::KeyD]
        );
    }

    #[test]
    fn when_single_key_bound_to_action_then_bindings_for_returns_that_key() {
        // Arrange
        let mut map = ActionMap::default();

        // Act
        map.bind("jump", vec![KeyCode::Space]);

        // Assert
        assert_eq!(map.bindings_for("jump"), &[KeyCode::Space]);
    }
}
