use std::vec::Drain;

use bevy_ecs::prelude::Resource;
use winit::event::ElementState;

use super::state::MouseButton;

#[derive(Resource, Debug, Default)]
pub struct MouseEventBuffer {
    events: Vec<(MouseButton, ElementState)>,
}

impl MouseEventBuffer {
    pub fn push(&mut self, button: MouseButton, state: ElementState) {
        self.events.push((button, state));
    }

    pub fn drain(&mut self) -> Drain<'_, (MouseButton, ElementState)> {
        self.events.drain(..)
    }
}

#[cfg(test)]
mod tests {
    use winit::event::{ElementState, MouseButton};

    use super::*;

    #[test]
    fn when_button_event_pushed_then_drain_returns_that_event() {
        // Arrange
        let mut buffer = MouseEventBuffer::default();

        // Act
        buffer.push(MouseButton::Left, ElementState::Pressed);
        let events: Vec<_> = buffer.drain().collect();

        // Assert
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], (MouseButton::Left, ElementState::Pressed));
    }

    #[test]
    fn when_buffer_drained_then_buffer_is_empty_on_second_drain() {
        // Arrange
        let mut buffer = MouseEventBuffer::default();
        buffer.push(MouseButton::Left, ElementState::Pressed);
        buffer.push(MouseButton::Right, ElementState::Released);

        // Act
        let _: Vec<_> = buffer.drain().collect();

        // Assert
        assert_eq!(buffer.drain().count(), 0);
    }
}
