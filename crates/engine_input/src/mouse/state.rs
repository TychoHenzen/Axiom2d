use std::collections::HashSet;

use bevy_ecs::prelude::Resource;
use glam::Vec2;

use crate::mouse_button::MouseButton;

#[derive(Resource, Debug, Clone, PartialEq, Default)]
pub struct MouseState {
    pressed: HashSet<MouseButton>,
    just_pressed: HashSet<MouseButton>,
    just_released: HashSet<MouseButton>,
    screen_pos: Vec2,
    world_pos: Vec2,
    scroll_delta: Vec2,
}

impl MouseState {
    pub fn pressed(&self, button: MouseButton) -> bool {
        self.pressed.contains(&button)
    }

    pub fn just_pressed(&self, button: MouseButton) -> bool {
        self.just_pressed.contains(&button)
    }

    pub fn just_released(&self, button: MouseButton) -> bool {
        self.just_released.contains(&button)
    }

    pub fn press(&mut self, button: MouseButton) {
        self.pressed.insert(button);
        self.just_pressed.insert(button);
    }

    pub fn release(&mut self, button: MouseButton) {
        self.pressed.remove(&button);
        self.just_released.insert(button);
    }

    pub fn clear_frame_state(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }

    pub fn clear_scroll_delta(&mut self) {
        self.scroll_delta = Vec2::ZERO;
    }

    pub fn screen_pos(&self) -> Vec2 {
        self.screen_pos
    }

    pub fn set_screen_pos(&mut self, pos: Vec2) {
        self.screen_pos = pos;
    }

    pub fn world_pos(&self) -> Vec2 {
        self.world_pos
    }

    pub fn set_world_pos(&mut self, pos: Vec2) {
        self.world_pos = pos;
    }

    pub fn scroll_delta(&self) -> Vec2 {
        self.scroll_delta
    }

    pub fn add_scroll_delta(&mut self, delta: Vec2) {
        self.scroll_delta += delta;
    }

    pub fn action_pressed(&self, map: &crate::action_map::ActionMap, action: &str) -> bool {
        map.mouse_bindings_for(action)
            .iter()
            .any(|btn| self.pressed(*btn))
    }

    pub fn action_just_pressed(&self, map: &crate::action_map::ActionMap, action: &str) -> bool {
        map.mouse_bindings_for(action)
            .iter()
            .any(|btn| self.just_pressed(*btn))
    }
}
