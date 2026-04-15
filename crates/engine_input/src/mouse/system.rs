use bevy_ecs::prelude::ResMut;
use engine_core::prelude::EventBus;

use super::buffer::MouseInputEvent;
use super::state::MouseState;
use crate::button_state::ButtonState;

pub fn mouse_input_system(
    mut bus: ResMut<EventBus<MouseInputEvent>>,
    mut state: ResMut<MouseState>,
) {
    state.clear_frame_state();
    for event in bus.drain() {
        match event {
            MouseInputEvent::Button { button, state: bs } => match bs {
                ButtonState::Pressed => state.press(button),
                ButtonState::Released => state.release(button),
            },
            MouseInputEvent::Move { screen_pos } => state.set_screen_pos(screen_pos),
            MouseInputEvent::Scroll { delta } => state.add_scroll_delta(delta),
        }
    }
}

pub fn scroll_clear_system(mut state: ResMut<MouseState>) {
    state.clear_scroll_delta();
}
