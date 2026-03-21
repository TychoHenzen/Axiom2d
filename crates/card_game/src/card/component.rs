use bevy_ecs::prelude::Component;
use engine_core::prelude::TextureId;
use serde::{Deserialize, Serialize};

use super::signature::CardSignature;

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
