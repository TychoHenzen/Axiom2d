use bevy_ecs::prelude::Resource;

pub trait Event: Send + Sync + 'static {}

#[derive(Resource, Debug)]
pub struct EventBus<T: Event> {
    events: Vec<T>,
}

impl<T: Event> Default for EventBus<T> {
    fn default() -> Self {
        Self { events: Vec::new() }
    }
}

impl<T: Event> EventBus<T> {
    pub fn push(&mut self, event: T) {
        self.events.push(event);
    }

    pub fn drain(&mut self) -> std::vec::Drain<'_, T> {
        self.events.drain(..)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.events.iter_mut()
    }
}

impl<'a, T: Event> IntoIterator for &'a mut EventBus<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct TestEvent(u32);
    impl Event for TestEvent {}

    /// @doc: `EventBus` is the single in-frame communication channel between
    /// ECS systems that produce events and systems that consume them. If push
    /// silently discards the event or drain returns an empty iterator, producer
    /// systems become invisible to consumers — game logic depending on events
    /// (damage, card plays, collision callbacks) would silently do nothing.
    #[test]
    fn when_single_event_pushed_then_drain_yields_that_event() {
        // Arrange
        let mut bus = EventBus::<TestEvent>::default();

        // Act
        bus.push(TestEvent(42));
        let events: Vec<TestEvent> = bus.drain().collect();

        // Assert
        assert_eq!(events, vec![TestEvent(42)]);
    }

    /// @doc: Systems that consume events must see them in the order they were
    /// produced within a frame — a damage event before a death event, a card-play
    /// before a score update. If drain reorders events, downstream consumers
    /// react to effects before their causes, breaking gameplay invariants that
    /// depend on sequenced processing.
    #[test]
    fn when_multiple_events_pushed_then_drain_yields_events_in_insertion_order() {
        // Arrange
        let mut bus = EventBus::<TestEvent>::default();
        bus.push(TestEvent(1));
        bus.push(TestEvent(2));
        bus.push(TestEvent(3));

        // Act
        let events: Vec<TestEvent> = bus.drain().collect();

        // Assert
        assert_eq!(events, vec![TestEvent(1), TestEvent(2), TestEvent(3)]);
    }

    /// @doc: Each frame, consumer systems drain the event bus to process that
    /// frame's events. If drain didn't clear the buffer, events would accumulate
    /// and be re-processed every subsequent frame — a single collision would
    /// trigger damage every tick instead of once, and input events would repeat
    /// indefinitely.
    #[test]
    fn when_drained_then_subsequent_drain_returns_empty() {
        // Arrange
        let mut bus = EventBus::<TestEvent>::default();
        bus.push(TestEvent(1));
        let _first: Vec<TestEvent> = bus.drain().collect();

        // Act
        let second: Vec<TestEvent> = bus.drain().collect();

        // Assert
        assert!(second.is_empty());
    }

    /// @doc: Systems guard on `is_empty()` to skip expensive iteration when no
    /// events arrived in a frame. If `is_empty` reported wrong state — false
    /// when empty or true when populated — guard systems would either spin
    /// needlessly or silently drop events that should be processed.
    #[test]
    fn when_events_present_then_is_empty_false_and_after_drain_is_empty_true() {
        // Arrange
        let mut bus = EventBus::<TestEvent>::default();
        bus.push(TestEvent(1));

        // Act / Assert — before drain
        assert!(!bus.is_empty());

        // Act — drain
        let _: Vec<TestEvent> = bus.drain().collect();

        // Assert — after drain
        assert!(bus.is_empty());
    }

    /// @doc: Diagnostic and pre-allocation code relies on `len()` to know how
    /// many events are pending. An incorrect count could cause under-allocation
    /// of result buffers or misleading debug output showing phantom events after
    /// a drain has cleared the bus.
    #[test]
    fn when_events_pushed_then_len_matches_and_after_drain_len_is_zero() {
        // Arrange
        let mut bus = EventBus::<TestEvent>::default();
        bus.push(TestEvent(1));
        bus.push(TestEvent(2));
        bus.push(TestEvent(3));

        // Act / Assert — before drain
        assert_eq!(bus.len(), 3);

        // Act — drain
        let _: Vec<TestEvent> = bus.drain().collect();

        // Assert — after drain
        assert_eq!(bus.len(), 0);
    }

    /// @doc: The spatial audio system mutates events in place (stamping spatial
    /// gains) before a later system drains them. `iter_mut` provides mutable
    /// access without consuming the events — if it drained instead, the
    /// downstream `play_sound_system` would see an empty bus and never hear
    /// the spatially-processed sounds.
    #[test]
    fn when_iter_mut_called_then_events_mutated_in_place_without_draining() {
        // Arrange
        let mut bus = EventBus::<TestEvent>::default();
        bus.push(TestEvent(1));
        bus.push(TestEvent(2));

        // Act
        for event in &mut bus {
            event.0 += 10;
        }

        // Assert — events are mutated and still in the bus
        assert_eq!(bus.len(), 2);
        let events: Vec<TestEvent> = bus.drain().collect();
        assert_eq!(events, vec![TestEvent(11), TestEvent(12)]);
    }

    /// @doc: The `for event in &mut bus` pattern (via `IntoIterator`) is
    /// syntactic sugar for `iter_mut()` used by systems that annotate events
    /// in place. If this impl were missing, the idiomatic `for cmd in &mut *bus`
    /// pattern in `spatial_audio_system` would fail to compile.
    #[test]
    fn when_into_iterator_for_mut_ref_then_all_events_yielded_as_mutable() {
        // Arrange
        let mut bus = EventBus::<TestEvent>::default();
        bus.push(TestEvent(1));
        bus.push(TestEvent(2));
        bus.push(TestEvent(3));

        // Act
        for event in &mut bus {
            event.0 *= 2;
        }

        // Assert — all 3 events remain and are mutated
        assert_eq!(bus.len(), 3);
        let events: Vec<TestEvent> = bus.drain().collect();
        assert_eq!(events, vec![TestEvent(2), TestEvent(4), TestEvent(6)]);
    }

    /// @doc: The ECS stores each `EventBus<T>` as a separate `Resource` keyed by
    /// its concrete type. If two bus instances accidentally shared backing storage
    /// (e.g., via a static), pushing a collision event would pollute the UI event
    /// bus, causing systems to process events meant for a completely different
    /// subsystem.
    #[test]
    fn when_two_buses_used_then_push_to_one_does_not_affect_other() {
        // Arrange
        let mut bus_a = EventBus::<TestEvent>::default();
        let bus_b = EventBus::<TestEvent>::default();

        // Act
        bus_a.push(TestEvent(99));

        // Assert
        assert_eq!(bus_a.len(), 1);
        assert_eq!(bus_b.len(), 0);
        assert!(bus_b.is_empty());
    }
}
