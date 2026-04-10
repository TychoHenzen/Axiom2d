// EVOLVE-BLOCK-START
use crate::button_state::ButtonState;
use crate::mouse_button::MouseButton;
use engine_core::prelude::Event;
use glam::Vec2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseInputEvent {
    Button {
        button: MouseButton,
        state: ButtonState,
    },
    Move {
        screen_pos: Vec2,
    },
    Scroll {
        delta: Vec2,
    },
}

impl Event for MouseInputEvent {}
// EVOLVE-BLOCK-END
