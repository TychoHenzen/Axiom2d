use bevy_ecs::prelude::{Entity, Resource};
use glam::Vec2;

use crate::card::component::CardZone;

/// Shared drag info for device entities (reader, screen, combiner, booster).
///
/// All devices track the same two fields when dragged: the entity being dragged
/// and the grab offset from the entity's origin to the cursor.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeviceDragInfo {
    pub entity: Entity,
    pub grab_offset: Vec2,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DragInfo {
    pub entity: Entity,
    pub local_grab_offset: Vec2,
    pub origin_zone: CardZone,
    pub stash_cursor_follow: bool,
    pub origin_position: Vec2,
}

#[derive(Resource, Debug, Clone, PartialEq, Default)]
pub struct DragState {
    pub dragging: Option<DragInfo>,
}
