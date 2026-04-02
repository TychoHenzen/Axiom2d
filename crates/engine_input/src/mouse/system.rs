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
    for MouseInputEvent { button, state: bs } in bus.drain() {
        match bs {
            ButtonState::Pressed => state.press(button),
            ButtonState::Released => state.release(button),
        }
    }
}

pub fn scroll_clear_system(mut state: ResMut<MouseState>) {
    state.clear_scroll_delta();
}
