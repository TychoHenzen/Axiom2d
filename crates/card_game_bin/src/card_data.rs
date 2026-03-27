use card_game::prelude::*;
use glam::Vec2;
use rand_chacha::ChaCha8Rng;

pub struct StarterCard {
    pub definition: CardDefinition,
    pub position: Vec2,
    pub face_up: bool,
    pub signature: CardSignature,
}

pub fn starter_deck(_rng: &mut ChaCha8Rng) -> Vec<StarterCard> {
    // Hand-crafted signatures that produce one of each rarity tier.
    // Rarity is derived from sum-of-abs-axes through a log-normalized score.
    let cards: Vec<(&str, CardType, Option<CardStats>, Vec<Keyword>, &str, Vec2, [f32; 8])> = vec![
        (
            "Fireball",
            CardType::Spell,
            None,
            vec![],
            "Deal 3 damage",
            Vec2::new(-120.0, 0.0),
            // Common: raw ≈ 0.8, normalized ≈ 0.25
            [0.1, -0.1, 0.1, -0.1, 0.1, -0.1, 0.1, -0.1],
        ),
        (
            "Shield",
            CardType::Artifact,
            None,
            vec![],
            "Block 2 damage",
            Vec2::new(-60.0, 30.0),
            // Uncommon: raw ≈ 1.6, normalized ≈ 0.43
            [0.2, -0.2, 0.2, -0.2, 0.2, -0.2, 0.2, -0.2],
        ),
        (
            "Heal",
            CardType::Creature,
            Some(CardStats {
                cost: 3,
                attack: 2,
                health: 4,
            }),
            vec![Keyword::Lifesteal],
            "Restore 4 HP",
            Vec2::new(0.0, 0.0),
            // Rare: raw ≈ 2.8, normalized ≈ 0.55
            [0.35, -0.35, 0.35, -0.35, 0.35, -0.35, 0.35, -0.35],
        ),
        (
            "Lightning",
            CardType::Spell,
            None,
            vec![],
            "Deal 5 damage",
            Vec2::new(60.0, -20.0),
            // Epic: raw ≈ 4.4, normalized ≈ 0.72
            [0.55, -0.55, 0.55, -0.55, 0.55, -0.55, 0.55, -0.55],
        ),
        (
            "Draw",
            CardType::Spell,
            None,
            vec![],
            "Draw 2 cards",
            Vec2::new(120.0, 10.0),
            // Legendary: raw ≈ 6.4, normalized ≈ 0.86
            [0.8, -0.8, 0.8, -0.8, 0.8, -0.8, 0.8, -0.8],
        ),
    ];

    cards
        .into_iter()
        .map(
            |(name, card_type, stats, keywords, text, position, axes)| {
                StarterCard {
                    definition: CardDefinition {
                        card_type,
                        name: name.to_owned(),
                        stats,
                        abilities: CardAbilities {
                            keywords,
                            text: text.to_owned(),
                        },
                        art: art_descriptor_default(card_type),
                    },
                    position,
                    face_up: true,
                    signature: CardSignature::new(axes),
                }
            },
        )
        .collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn when_starter_deck_built_then_every_card_has_non_default_signature() {
        // Arrange
        let mut rng = ChaCha8Rng::seed_from_u64(0);

        // Act
        let deck = starter_deck(&mut rng);

        // Assert
        assert_eq!(deck.len(), 5);
        for card in &deck {
            assert_ne!(
                card.signature,
                CardSignature::default(),
                "card '{}' should have a non-default signature",
                card.definition.name
            );
        }
    }

    #[test]
    fn when_starter_deck_built_then_all_cards_are_face_up() {
        // Arrange
        let mut rng = ChaCha8Rng::seed_from_u64(0);

        // Act
        let deck = starter_deck(&mut rng);

        // Assert
        for card in &deck {
            assert!(
                card.face_up,
                "card '{}' should be face up",
                card.definition.name
            );
        }
    }

    #[test]
    fn when_starter_deck_built_then_every_rarity_tier_is_represented() {
        // Arrange
        let mut rng = ChaCha8Rng::seed_from_u64(0);

        // Act
        let deck = starter_deck(&mut rng);

        // Assert
        let rarities: Vec<Rarity> = deck.iter().map(|c| c.signature.rarity()).collect();
        assert!(rarities.contains(&Rarity::Common), "missing Common: {rarities:?}");
        assert!(rarities.contains(&Rarity::Uncommon), "missing Uncommon: {rarities:?}");
        assert!(rarities.contains(&Rarity::Rare), "missing Rare: {rarities:?}");
        assert!(rarities.contains(&Rarity::Epic), "missing Epic: {rarities:?}");
        assert!(rarities.contains(&Rarity::Legendary), "missing Legendary: {rarities:?}");
    }
}
