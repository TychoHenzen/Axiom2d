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
