use std::vec::Drain;

use bevy_ecs::prelude::{Entity, Resource};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiEvent {
    Clicked(Entity),
    HoverEnter(Entity),
    HoverExit(Entity),
    FocusGained(Entity),
    FocusLost(Entity),
}

#[derive(Resource, Debug, Default)]
pub struct UiEventBuffer {
    events: Vec<UiEvent>,
}

impl UiEventBuffer {
    pub fn push(&mut self, event: UiEvent) {
        self.events.push(event);
    }

    pub fn drain(&mut self) -> Drain<'_, UiEvent> {
        self.events.drain(..)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use bevy_ecs::world::World;

    use super::*;

    #[test]
    fn when_clicked_event_pushed_then_drain_yields_exact_event_and_buffer_is_empty() {
        // Arrange
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let mut buffer = UiEventBuffer::default();
        buffer.push(UiEvent::Clicked(entity));

        // Act
        let drained: Vec<UiEvent> = buffer.drain().collect();

        // Assert
        assert_eq!(drained, vec![UiEvent::Clicked(entity)]);
        assert_eq!(buffer.drain().count(), 0);
    }

    #[test]
    fn when_drained_twice_then_second_drain_is_empty() {
        // Arrange
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let mut buffer = UiEventBuffer::default();
        buffer.push(UiEvent::HoverEnter(entity));
        let _ = buffer.drain().count();

        // Act
        let second: Vec<UiEvent> = buffer.drain().collect();

        // Assert
        assert!(second.is_empty());
    }
}
