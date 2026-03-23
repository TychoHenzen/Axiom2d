use bevy_ecs::prelude::Component;
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
    Stash { page: u8, col: u8, row: u8 },
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn when_card_face_down_then_signature_is_all_zeros() {
        // Arrange
        let face = TextureId(1);
        let back = TextureId(2);

        // Act
        let card = Card::face_down(face, back);

        // Assert
        assert_eq!(card.signature, CardSignature::default());
    }

    #[test]
    fn when_card_constructed_face_down_then_face_up_is_false() {
        // Arrange
        let face = TextureId(1);
        let back = TextureId(2);

        // Act
        let card = Card::face_down(face, back);

        // Assert
        assert!(!card.face_up);
    }
}
