// EVOLVE-BLOCK-START
use bevy_ecs::prelude::ResMut;
use engine_core::prelude::EventBus;

use super::buffer::KeyInputEvent;
use super::state::InputState;
use crate::button_state::ButtonState;

pub fn input_system(mut bus: ResMut<EventBus<KeyInputEvent>>, mut state: ResMut<InputState>) {
    state.clear_frame_state();
    for KeyInputEvent { key, state: bs } in bus.drain() {
        match bs {
            ButtonState::Pressed => state.press(key),
            ButtonState::Released => state.release(key),
        }
    }
}
// EVOLVE-BLOCK-END
