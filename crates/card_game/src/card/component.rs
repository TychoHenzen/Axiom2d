// EVOLVE-BLOCK-START
use bevy_ecs::prelude::{Component, Entity};
use engine_core::prelude::TextureId;
use serde::{Deserialize, Serialize};

use super::identity::signature::CardSignature;

#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Card {
    pub face_texture: TextureId,
    pub back_texture: TextureId,
    pub face_up: bool,
    pub signature: CardSignature,
}

impl Card {
    pub fn face_down(face_texture: TextureId, back_texture: TextureId) -> Self {
        Self {
            face_texture,
            back_texture,
            face_up: false,
            signature: CardSignature::default(),
        }
    }
}

/// Marker for cards in stash item-form (rendered as stash grid slots rather
/// than full table cards).
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct CardItemForm;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CardFaceSide {
    Front,
    Back,
}

#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CardLabel {
    pub name: String,
    pub description: String,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CardZone {
    Table,
    Hand(usize),
    Stash {
        page: u8,
        col: u8,
        row: u8,
    },
    #[serde(skip)]
    Reader(Entity),
}
// EVOLVE-BLOCK-END
