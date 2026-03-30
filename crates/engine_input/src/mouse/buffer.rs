use crate::button_state::ButtonState;
use crate::mouse_button::MouseButton;
use engine_core::prelude::Event;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseInputEvent {
    pub button: MouseButton,
    pub state: ButtonState,
}

impl Event for MouseInputEvent {}
