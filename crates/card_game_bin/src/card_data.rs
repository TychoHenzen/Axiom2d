use card_game::prelude::*;
use glam::Vec2;
use rand_chacha::ChaCha8Rng;

pub struct StarterCard {
    pub definition: CardDefinition,
    pub position: Vec2,
    pub face_up: bool,
    pub signature: CardSignature,
}

pub fn starter_deck(rng: &mut ChaCha8Rng) -> Vec<StarterCard> {
    let cards = vec![
        (
            "Fireball",
            CardType::Spell,
            None,
            vec![],
            "Deal 3 damage",
            Vec2::new(-120.0, 0.0),
            false,
        ),
        (
            "Shield",
            CardType::Artifact,
            None,
            vec![],
            "Block 2 damage",
            Vec2::new(-60.0, 30.0),
            false,
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
            true,
        ),
        (
            "Lightning",
            CardType::Spell,
            None,
            vec![],
            "Deal 5 damage",
            Vec2::new(60.0, -20.0),
            false,
        ),
        (
            "Draw",
            CardType::Spell,
            None,
            vec![],
            "Draw 2 cards",
            Vec2::new(120.0, 10.0),
            false,
        ),
    ];

    cards
        .into_iter()
        .map(
            |(name, card_type, stats, keywords, text, position, face_up)| {
                let signature = CardSignature::random(rng);
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
                    face_up,
                    signature,
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
    fn when_same_seed_then_identical_signatures() {
        // Arrange
        let mut rng1 = ChaCha8Rng::seed_from_u64(42);
        let mut rng2 = ChaCha8Rng::seed_from_u64(42);

        // Act
        let deck1 = starter_deck(&mut rng1);
        let deck2 = starter_deck(&mut rng2);

        // Assert
        for (a, b) in deck1.iter().zip(deck2.iter()) {
            assert_eq!(a.signature, b.signature);
        }
    }

    #[test]
    fn when_different_seeds_then_at_least_one_signature_differs() {
        // Arrange
        let mut rng1 = ChaCha8Rng::seed_from_u64(0);
        let mut rng2 = ChaCha8Rng::seed_from_u64(1);

        // Act
        let deck1 = starter_deck(&mut rng1);
        let deck2 = starter_deck(&mut rng2);

        // Assert
        let any_different = deck1
            .iter()
            .zip(deck2.iter())
            .any(|(a, b)| a.signature != b.signature);
        assert!(
            any_different,
            "different seeds should produce different signatures"
        );
    }
}
