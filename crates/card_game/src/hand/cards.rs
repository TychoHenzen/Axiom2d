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
