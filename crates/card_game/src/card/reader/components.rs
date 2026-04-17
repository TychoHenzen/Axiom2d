use bevy_ecs::prelude::{Component, Entity, Resource};
use glam::Vec2;

use crate::card::interaction::drag_state::DeviceDragInfo;

pub const READER_CARD_SCALE: f32 = 0.6;
pub const READER_COLLISION_GROUP: u32 = 0b0010;
pub const READER_COLLISION_FILTER: u32 = 0b0001;

#[derive(Component, Debug, Clone)]
pub struct CardReader {
    pub loaded: Option<Entity>,
    pub half_extents: Vec2,
    pub jack_entity: Entity,
}

pub fn card_overlaps_reader(card_pos: Vec2, reader_pos: Vec2, reader_half: Vec2) -> bool {
    let delta = (card_pos - reader_pos).abs();
    delta.x <= reader_half.x && delta.y <= reader_half.y
}

#[derive(Resource, Debug, Default)]
pub struct ReaderDragState {
    pub dragging: Option<DeviceDragInfo>,
}
