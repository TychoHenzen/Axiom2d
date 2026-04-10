// EVOLVE-BLOCK-START
use bevy_ecs::prelude::Entity;
use engine_core::prelude::Event;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiEvent {
    Clicked(Entity),
    HoverEnter(Entity),
    HoverExit(Entity),
    FocusGained(Entity),
    FocusLost(Entity),
}

impl Event for UiEvent {}
// EVOLVE-BLOCK-END
