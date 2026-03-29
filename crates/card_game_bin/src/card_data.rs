use card_game::prelude::*;
use glam::Vec2;
use rand_chacha::ChaCha8Rng;

pub struct StarterCard {
    pub definition: CardDefinition,
    pub position: Vec2,
    pub face_up: bool,
    pub signature: CardSignature,
}

/// Spacing constants for the 5×3 card grid layout.
const COL_SPACING: f32 = 80.0;
const ROW_SPACING: f32 = 130.0;
const START_X: f32 = -160.0;
const DORMANT_Y: f32 = ROW_SPACING;
const ACTIVE_Y: f32 = 0.0;
const INTENSE_Y: f32 = -ROW_SPACING;

const RARITIES: [Rarity; 5] = [
    Rarity::Common,
    Rarity::Uncommon,
    Rarity::Rare,
    Rarity::Epic,
    Rarity::Legendary,
];

use card_game::card::identity::signature_profile::Tier;

const TIERS: [Tier; 3] = [Tier::Dormant, Tier::Active, Tier::Intense];
const TIER_YS: [f32; 3] = [DORMANT_Y, ACTIVE_Y, INTENSE_Y];

fn make_def(card_type: CardType) -> CardDefinition {
    CardDefinition {
        card_type,
        name: String::new(), // procedural name from signature
        stats: None,
        abilities: CardAbilities {
            keywords: vec![],
            text: String::new(),
        },
        art: art_descriptor_default(card_type),
    }
}

/// Search for a signature whose hash-based rarity and card-level tier match the targets.
fn find_signature_for(rng: &mut ChaCha8Rng, rarity: Rarity, tier: Tier) -> CardSignature {
    for _ in 0..100_000 {
        let sig = CardSignature::random(rng);
        if sig.rarity() == rarity && sig.card_tier() == tier {
            return sig;
        }
    }
    // Fallback: accept rarity match alone (extremely rare tier combos)
    loop {
        let sig = CardSignature::random(rng);
        if sig.rarity() == rarity {
            return sig;
        }
    }
}

pub fn starter_deck(rng: &mut ChaCha8Rng) -> Vec<StarterCard> {
    let types = [
        CardType::Spell,
        CardType::Artifact,
        CardType::Creature,
        CardType::Spell,
        CardType::Artifact,
    ];

    let mut cards = Vec::with_capacity(15);
    for (row, (&tier, &y)) in TIERS.iter().zip(TIER_YS.iter()).enumerate() {
        for (col, &rarity) in RARITIES.iter().enumerate() {
            let x = START_X + col as f32 * COL_SPACING;
            let signature = find_signature_for(rng, rarity, tier);
            cards.push(StarterCard {
                definition: make_def(types[col]),
                position: Vec2::new(x, y),
                face_up: row != 0 || col != 0, // first card face-down for variety
                signature,
            });
        }
    }
    cards
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn when_starter_deck_built_then_fifteen_cards() {
        // Arrange
        let mut rng = ChaCha8Rng::seed_from_u64(0);

        // Act
        let deck = starter_deck(&mut rng);

        // Assert
        assert_eq!(deck.len(), 15);
    }

    /// @doc: The starter deck must cover all 15 rarity×tier combinations so
    /// every visual treatment is visible on the table at game start.
    #[test]
    fn when_starter_deck_built_then_all_rarity_tier_combinations_present() {
        // Arrange
        let mut rng = ChaCha8Rng::seed_from_u64(0);

        // Act
        let deck = starter_deck(&mut rng);

        // Assert
        let mut pairs: Vec<(Rarity, Tier)> = deck
            .iter()
            .map(|c| (c.signature.rarity(), c.signature.card_tier()))
            .collect();
        pairs.sort_by_key(|(r, t)| (*r as u8, *t as u8));
        pairs.dedup();
        assert_eq!(
            pairs.len(),
            15,
            "expected 15 unique rarity×tier combinations, got {}: {pairs:?}",
            pairs.len()
        );
    }

    #[test]
    fn when_starter_deck_built_then_dormant_row_has_correct_card_tier() {
        // Arrange
        let mut rng = ChaCha8Rng::seed_from_u64(0);

        // Act
        let deck = starter_deck(&mut rng);

        // Assert — first 5 cards are the dormant row
        for card in &deck[..5] {
            assert_eq!(
                card.signature.card_tier(),
                Tier::Dormant,
                "dormant row card at {:?} has wrong card_tier",
                card.position
            );
        }
    }

    #[test]
    fn when_starter_deck_built_then_active_row_has_correct_card_tier() {
        // Arrange
        let mut rng = ChaCha8Rng::seed_from_u64(0);

        // Act
        let deck = starter_deck(&mut rng);

        // Assert — cards 5..10 are the active row
        for card in &deck[5..10] {
            assert_eq!(
                card.signature.card_tier(),
                Tier::Active,
                "active row card at {:?} has wrong card_tier",
                card.position
            );
        }
    }

    #[test]
    fn when_starter_deck_built_then_intense_row_has_correct_card_tier() {
        // Arrange
        let mut rng = ChaCha8Rng::seed_from_u64(0);

        // Act
        let deck = starter_deck(&mut rng);

        // Assert — cards 10..15 are the intense row
        for card in &deck[10..15] {
            assert_eq!(
                card.signature.card_tier(),
                Tier::Intense,
                "intense row card at {:?} has wrong card_tier",
                card.position
            );
        }
    }
}
