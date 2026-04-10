// EVOLVE-BLOCK-START
use std::collections::HashSet;

use bevy_ecs::prelude::Resource;

use crate::action_map::ActionMap;
use crate::key_code::KeyCode;

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
        map.bindings_for(action)
            .iter()
            .any(|key| self.pressed(*key))
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
// EVOLVE-BLOCK-END
