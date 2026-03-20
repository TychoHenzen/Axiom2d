use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CardLabel {
    pub name: String,
    pub description: String,
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn when_card_label_serialized_to_ron_then_roundtrip_preserves_name_and_description() {
        // Arrange
        let label = CardLabel {
            name: "Fireball".to_owned(),
            description: "Deal 3 damage".to_owned(),
        };

        // Act
        let ron = ron::to_string(&label).unwrap();
        let back: CardLabel = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(label, back);
    }
}
