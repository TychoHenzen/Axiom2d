use bevy_ecs::prelude::Resource;
use winit::event::ElementState;
use winit::keyboard::KeyCode;

#[derive(Resource, Debug, Default)]
pub struct InputEventBuffer {
    events: Vec<(KeyCode, ElementState)>,
}

impl InputEventBuffer {
    pub fn push(&mut self, key: KeyCode, state: ElementState) {
        self.events.push((key, state));
    }

    pub fn drain(&mut self) -> Vec<(KeyCode, ElementState)> {
        self.events.drain(..).collect()
    }
}

#[cfg(test)]
mod tests {
    use winit::event::ElementState;
    use winit::keyboard::KeyCode;

    use super::*;

    #[test]
    fn when_key_event_pushed_then_drain_returns_one_event() {
        // Arrange
        let mut buffer = InputEventBuffer::default();

        // Act
        buffer.push(KeyCode::ArrowRight, ElementState::Pressed);

        // Assert
        assert_eq!(buffer.drain().len(), 1);
    }

    #[test]
    fn when_buffer_drained_then_returns_all_events_and_buffer_is_empty() {
        // Arrange
        let mut buffer = InputEventBuffer::default();
        buffer.push(KeyCode::ArrowLeft, ElementState::Pressed);
        buffer.push(KeyCode::ArrowRight, ElementState::Released);

        // Act
        let events = buffer.drain();

        // Assert
        assert_eq!(events.len(), 2);
        assert_eq!(events[0], (KeyCode::ArrowLeft, ElementState::Pressed));
        assert_eq!(events[1], (KeyCode::ArrowRight, ElementState::Released));
        assert!(buffer.drain().is_empty());
    }

}
