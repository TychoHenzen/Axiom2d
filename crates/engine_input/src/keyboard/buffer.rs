// EVOLVE-BLOCK-START
use crate::button_state::ButtonState;
use crate::key_code::KeyCode;
use engine_core::prelude::Event;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyInputEvent {
    pub key: KeyCode,
    pub state: ButtonState,
}

impl Event for KeyInputEvent {}
// EVOLVE-BLOCK-END
