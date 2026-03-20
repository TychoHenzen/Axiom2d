use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

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
    fn when_two_hand_zones_have_different_indices_then_not_equal() {
        // Arrange
        let zone_0 = CardZone::Hand(0);
        let zone_1 = CardZone::Hand(1);

        // Act + Assert
        assert_ne!(zone_0, zone_1);
    }

    #[test]
    fn when_two_stash_zones_differ_in_any_coordinate_then_not_equal() {
        // Arrange
        let origin = CardZone::Stash {
            page: 0,
            col: 0,
            row: 0,
        };

        // Act + Assert
        assert_ne!(
            origin,
            CardZone::Stash {
                page: 1,
                col: 0,
                row: 0
            },
            "page differs"
        );
        assert_ne!(
            origin,
            CardZone::Stash {
                page: 0,
                col: 1,
                row: 0
            },
            "col differs"
        );
        assert_ne!(
            origin,
            CardZone::Stash {
                page: 0,
                col: 0,
                row: 1
            },
            "row differs"
        );
    }

    #[test]
    fn when_same_stash_coordinates_then_zones_are_equal() {
        // Arrange
        let a = CardZone::Stash {
            page: 2,
            col: 3,
            row: 4,
        };
        let b = CardZone::Stash {
            page: 2,
            col: 3,
            row: 4,
        };

        // Assert
        assert_eq!(a, b);
    }

    #[test]
    fn when_card_zone_serialized_to_ron_then_roundtrips() {
        // Arrange
        let zones = [
            CardZone::Table,
            CardZone::Hand(3),
            CardZone::Stash {
                page: 1,
                col: 4,
                row: 7,
            },
        ];

        for zone in zones {
            // Act
            let ron = ron::to_string(&zone).unwrap();
            let back: CardZone = ron::from_str(&ron).unwrap();

            // Assert
            assert_eq!(zone, back);
        }
    }
}
