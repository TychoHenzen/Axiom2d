use std::vec::Drain;

use bevy_ecs::prelude::Resource;

use crate::button_state::ButtonState;
use crate::mouse_button::MouseButton;

#[derive(Resource, Debug, Default)]
pub struct MouseEventBuffer {
    events: Vec<(MouseButton, ButtonState)>,
}

impl MouseEventBuffer {
    pub fn push(&mut self, button: MouseButton, state: ButtonState) {
        self.events.push((button, state));
    }

    pub fn drain(&mut self) -> Drain<'_, (MouseButton, ButtonState)> {
        self.events.drain(..)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_button_event_pushed_then_drain_returns_that_event() {
        // Arrange
        let mut buffer = MouseEventBuffer::default();

        // Act
        buffer.push(MouseButton::Left, ButtonState::Pressed);
        let events: Vec<_> = buffer.drain().collect();

        // Assert
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], (MouseButton::Left, ButtonState::Pressed));
    }

    #[test]
    fn when_buffer_drained_then_buffer_is_empty_on_second_drain() {
        // Arrange
        let mut buffer = MouseEventBuffer::default();
        buffer.push(MouseButton::Left, ButtonState::Pressed);
        buffer.push(MouseButton::Right, ButtonState::Released);

        // Act
        let _: Vec<_> = buffer.drain().collect();

        // Assert
        assert_eq!(buffer.drain().count(), 0);
    }
}
