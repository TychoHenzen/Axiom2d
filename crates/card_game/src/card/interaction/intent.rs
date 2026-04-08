use bevy_ecs::prelude::Entity;
use engine_core::event_bus::Event;
use engine_physics::prelude::Collider;
use glam::Vec2;

use crate::card::component::CardZone;

#[derive(Debug, Clone, PartialEq)]
pub enum InteractionIntent {
    PickCard {
        entity: Entity,
        zone: CardZone,
        collider: Collider,
        grab_offset: Vec2,
    },
    PickFromStash {
        entity: Entity,
        page: u8,
        col: u8,
        row: u8,
    },
    ReleaseOnTable {
        entity: Entity,
        snap_back: bool,
    },
    ReleaseOnHand {
        entity: Entity,
        face_up: bool,
        origin_position: Vec2,
    },
    ReleaseOnStash {
        entity: Entity,
        page: u8,
        col: u8,
        row: u8,
        current_position: Vec2,
    },
    OpenBoosterPack {
        entity: Entity,
    },
}

impl Event for InteractionIntent {}
