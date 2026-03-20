use std::vec::Drain;

use bevy_ecs::prelude::Resource;

use crate::button_state::ButtonState;
use crate::key_code::KeyCode;

#[derive(Resource, Debug, Default)]
pub struct InputEventBuffer {
    events: Vec<(KeyCode, ButtonState)>,
}

impl InputEventBuffer {
    pub fn push(&mut self, key: KeyCode, state: ButtonState) {
        self.events.push((key, state));
    }

    pub fn drain(&mut self) -> Drain<'_, (KeyCode, ButtonState)> {
        self.events.drain(..)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_key_event_pushed_then_drain_returns_one_event() {
        // Arrange
        let mut buffer = InputEventBuffer::default();

        // Act
        buffer.push(KeyCode::ArrowRight, ButtonState::Pressed);

        // Assert
        assert_eq!(buffer.drain().count(), 1);
    }

    #[test]
    fn when_buffer_drained_then_returns_all_events_and_buffer_is_empty() {
        // Arrange
        let mut buffer = InputEventBuffer::default();
        buffer.push(KeyCode::ArrowLeft, ButtonState::Pressed);
        buffer.push(KeyCode::ArrowRight, ButtonState::Released);

        // Act
        let events: Vec<_> = buffer.drain().collect();

        // Assert
        assert_eq!(events.len(), 2);
        assert_eq!(events[0], (KeyCode::ArrowLeft, ButtonState::Pressed));
        assert_eq!(events[1], (KeyCode::ArrowRight, ButtonState::Released));
        assert_eq!(buffer.drain().count(), 0);
    }
}
