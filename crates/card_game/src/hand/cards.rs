use bevy_ecs::prelude::{Entity, Resource};

#[derive(Debug)]
pub struct HandFull;

impl std::fmt::Display for HandFull {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("hand is full")
    }
}

impl std::error::Error for HandFull {}

#[derive(Resource)]
pub struct Hand {
    cards: Vec<Entity>,
    max_size: usize,
}

impl Hand {
    pub fn new(max_size: usize) -> Self {
        Self {
            cards: Vec::with_capacity(max_size),
            max_size,
        }
    }

    pub fn is_full(&self) -> bool {
        self.cards.len() >= self.max_size
    }

    pub fn add(&mut self, entity: Entity) -> Result<usize, HandFull> {
        if self.is_full() {
            return Err(HandFull);
        }
        let index = self.cards.len();
        self.cards.push(entity);
        Ok(index)
    }

    pub fn remove(&mut self, entity: Entity) -> Option<usize> {
        let index = self.cards.iter().position(|&e| e == entity)?;
        self.cards.remove(index);
        Some(index)
    }

    pub fn cards(&self) -> &[Entity] {
        &self.cards
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use bevy_ecs::prelude::World;

    #[test]
    fn when_adding_first_card_then_returns_index_zero() {
        // Arrange
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let mut hand = Hand::new(5);

        // Act
        let result = hand.add(entity);

        // Assert
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn when_adding_second_card_then_returns_index_one() {
        // Arrange
        let mut world = World::new();
        let first = world.spawn_empty().id();
        let second = world.spawn_empty().id();
        let mut hand = Hand::new(5);
        hand.add(first).unwrap();

        // Act
        let result = hand.add(second);

        // Assert
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn when_adding_card_to_full_hand_then_returns_hand_full_error() {
        // Arrange
        let mut world = World::new();
        let first = world.spawn_empty().id();
        let second = world.spawn_empty().id();
        let overflow = world.spawn_empty().id();
        let mut hand = Hand::new(2);
        hand.add(first).unwrap();
        hand.add(second).unwrap();

        // Act
        let result = hand.add(overflow);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn when_hand_is_full_then_is_full_returns_true() {
        // Arrange
        let mut world = World::new();
        let entity = world.spawn_empty().id();
        let mut hand = Hand::new(1);
        hand.add(entity).unwrap();

        // Act / Assert
        assert!(hand.is_full());
    }

    #[test]
    fn when_hand_is_not_full_then_is_full_returns_false() {
        // Arrange
        let hand = Hand::new(3);

        // Act / Assert
        assert!(!hand.is_full());
    }

    #[test]
    fn when_removing_a_present_card_then_returns_its_former_index() {
        // Arrange
        let mut world = World::new();
        let a = world.spawn_empty().id();
        let b = world.spawn_empty().id();
        let c = world.spawn_empty().id();
        let mut hand = Hand::new(5);
        hand.add(a).unwrap();
        hand.add(b).unwrap();
        hand.add(c).unwrap();

        // Act
        let result = hand.remove(b);

        // Assert
        assert_eq!(result, Some(1));
    }

    #[test]
    fn when_removing_an_unknown_entity_then_returns_none() {
        // Arrange
        let mut world = World::new();
        let known = world.spawn_empty().id();
        let unknown = world.spawn_empty().id();
        let mut hand = Hand::new(5);
        hand.add(known).unwrap();

        // Act
        let result = hand.remove(unknown);

        // Assert
        assert_eq!(result, None);
    }

    #[test]
    fn when_removing_a_card_then_remaining_cards_preserve_relative_order() {
        // Arrange
        let mut world = World::new();
        let a = world.spawn_empty().id();
        let b = world.spawn_empty().id();
        let c = world.spawn_empty().id();
        let mut hand = Hand::new(5);
        hand.add(a).unwrap();
        hand.add(b).unwrap();
        hand.add(c).unwrap();

        // Act
        hand.remove(a);

        // Assert
        assert_eq!(hand.cards(), &[b, c]);
    }

    #[test]
    fn when_removing_a_card_then_len_decrements_by_one() {
        // Arrange
        let mut world = World::new();
        let a = world.spawn_empty().id();
        let b = world.spawn_empty().id();
        let mut hand = Hand::new(5);
        hand.add(a).unwrap();
        hand.add(b).unwrap();

        // Act
        hand.remove(a);

        // Assert
        assert_eq!(hand.len(), 1);
    }

    #[test]
    fn when_cards_added_in_order_then_cards_slice_reflects_insertion_order() {
        // Arrange
        let mut world = World::new();
        let a = world.spawn_empty().id();
        let b = world.spawn_empty().id();
        let c = world.spawn_empty().id();
        let mut hand = Hand::new(3);
        hand.add(a).unwrap();
        hand.add(b).unwrap();
        hand.add(c).unwrap();

        // Act / Assert
        assert_eq!(hand.cards(), &[a, b, c]);
    }
}
