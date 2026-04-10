// EVOLVE-BLOCK-START
use bevy_ecs::prelude::Entity;
use engine_core::prelude::Event;

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

impl Event for CollisionEvent {}
// EVOLVE-BLOCK-END
