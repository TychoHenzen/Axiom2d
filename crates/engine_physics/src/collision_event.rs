use std::vec::Drain;

use bevy_ecs::prelude::{Entity, Resource};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionKind {
    Started,
    Stopped,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollisionEvent {
    pub entity_a: Entity,
    pub entity_b: Entity,
    pub kind: CollisionKind,
}

#[derive(Resource, Debug, Default)]
pub struct CollisionEventBuffer {
    events: Vec<CollisionEvent>,
}

impl CollisionEventBuffer {
    pub fn push(&mut self, event: CollisionEvent) {
        self.events.push(event);
    }

    pub fn drain(&mut self) -> Drain<'_, CollisionEvent> {
        self.events.drain(..)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_helpers::spawn_entities;

    #[test]
    fn when_empty_buffer_drained_then_returns_empty_iterator() {
        // Arrange
        let mut buffer = CollisionEventBuffer::default();

        // Act
        let events: Vec<_> = buffer.drain().collect();

        // Assert
        assert!(events.is_empty());
    }

    #[test]
    fn when_event_pushed_and_drained_then_yields_that_event() {
        // Arrange
        let entities = spawn_entities(2);
        let mut buffer = CollisionEventBuffer::default();
        let event = CollisionEvent {
            entity_a: entities[0],
            entity_b: entities[1],
            kind: CollisionKind::Started,
        };
        buffer.push(event);

        // Act
        let events: Vec<_> = buffer.drain().collect();

        // Assert
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], event);
    }

    #[test]
    fn when_drained_twice_then_second_drain_returns_empty() {
        // Arrange
        let entities = spawn_entities(2);
        let mut buffer = CollisionEventBuffer::default();
        buffer.push(CollisionEvent {
            entity_a: entities[0],
            entity_b: entities[1],
            kind: CollisionKind::Started,
        });
        buffer.push(CollisionEvent {
            entity_a: entities[1],
            entity_b: entities[0],
            kind: CollisionKind::Stopped,
        });

        // Act
        let _ = buffer.drain().count();
        let second: Vec<_> = buffer.drain().collect();

        // Assert
        assert!(second.is_empty());
    }
}
