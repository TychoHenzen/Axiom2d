use bevy_ecs::prelude::{Entity, Resource};
use glam::Vec2;

use crate::card_zone::CardZone;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DragInfo {
    pub entity: Entity,
    pub local_grab_offset: Vec2,
    pub origin_zone: CardZone,
}

#[derive(Resource, Debug, Clone, PartialEq, Default)]
pub struct DragState {
    pub dragging: Option<DragInfo>,
}
