use std::collections::HashMap;

use bevy_ecs::prelude::Resource;

use crate::key_code::KeyCode;
use crate::mouse_button::MouseButton;

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
