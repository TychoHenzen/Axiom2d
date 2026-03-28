use card_game::prelude::*;
use glam::Vec2;
use rand_chacha::ChaCha8Rng;

pub struct StarterCard {
    pub definition: CardDefinition,
    pub position: Vec2,
    pub face_up: bool,
    pub signature: CardSignature,
}

/// Spacing constants for the 3×5 card grid layout.
const COL_SPACING: f32 = 80.0;
const ROW_SPACING: f32 = 130.0;
const START_X: f32 = -160.0;
const DORMANT_Y: f32 = ROW_SPACING;
const ACTIVE_Y: f32 = 0.0;
const INTENSE_Y: f32 = -ROW_SPACING;

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

pub fn starter_deck(_rng: &mut ChaCha8Rng) -> Vec<StarterCard> {
    let types = [
        CardType::Spell,
        CardType::Artifact,
        CardType::Creature,
        CardType::Spell,
        CardType::Artifact,
    ];

    // Each row: 5 cards across rarities. Signatures crafted to hit target tier + rarity.
    // Tier = max(abs(axes)), Rarity = ln(1 + sum(abs(axes))) / ln(9)
    //
    // Dormant: dominant < 0.3 (max rarity = Rare since sum capped at ~2.3)
    let dormant_sigs: [[f32; 8]; 5] = [
        // Common: raw=0.8, dominant=0.1
        [0.1, -0.1, 0.1, -0.1, 0.1, -0.1, 0.1, -0.1],
        // Uncommon: raw=1.6, dominant=0.2
        [0.2, -0.2, 0.2, -0.2, 0.2, -0.2, 0.2, -0.2],
        // Rare: raw=2.32, dominant=0.29
        [0.29, -0.29, 0.29, -0.29, 0.29, -0.29, 0.29, -0.29],
        // Uncommon variant: raw=1.2, dominant=0.25
        [0.25, -0.15, 0.2, -0.1, 0.15, -0.1, 0.15, -0.1],
        // Common variant: raw=0.56, dominant=0.15
        [0.15, -0.07, 0.1, -0.06, 0.05, -0.05, 0.04, -0.04],
    ];

    // Active: dominant 0.3–0.69
    let active_sigs: [[f32; 8]; 5] = [
        // Common: raw=0.5, dominant=0.5
        [0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        // Uncommon: raw=1.6, dominant=0.5
        [0.5, -0.2, 0.2, -0.2, 0.2, -0.15, 0.1, -0.05],
        // Rare: raw=2.8, dominant=0.5
        [0.5, -0.35, 0.35, -0.35, 0.35, -0.35, 0.3, -0.25],
        // Epic: raw=4.5, dominant=0.69
        [0.69, -0.65, 0.65, -0.6, 0.55, -0.5, 0.45, -0.41],
        // Legendary: raw=5.52, dominant=0.69
        [0.69, -0.69, 0.69, -0.69, 0.69, -0.69, 0.69, -0.69],
    ];

    // Intense: dominant >= 0.7
    let intense_sigs: [[f32; 8]; 5] = [
        // Common: raw=0.7, dominant=0.7
        [0.7, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        // Uncommon: raw=1.7, dominant=0.7
        [0.7, -0.2, 0.15, -0.15, 0.15, -0.1, 0.1, -0.15],
        // Rare: raw=2.8, dominant=0.7
        [0.7, -0.35, 0.35, -0.35, 0.35, -0.35, 0.2, -0.15],
        // Epic: raw=4.4, dominant=0.8
        [0.8, -0.55, 0.55, -0.55, 0.55, -0.55, 0.55, -0.3],
        // Legendary: raw=6.4, dominant=0.8
        [0.8, -0.8, 0.8, -0.8, 0.8, -0.8, 0.8, -0.8],
    ];

    let rows: [(&[[f32; 8]; 5], f32); 3] = [
        (&dormant_sigs, DORMANT_Y),
        (&active_sigs, ACTIVE_Y),
        (&intense_sigs, INTENSE_Y),
    ];

    let mut cards = Vec::with_capacity(15);
    for (sigs, y) in &rows {
        for (col, axes) in sigs.iter().enumerate() {
            let x = START_X + col as f32 * COL_SPACING;
            cards.push(StarterCard {
                definition: make_def(types[col]),
                position: Vec2::new(x, *y),
                face_up: true,
                signature: CardSignature::new(*axes),
            });
        }
    }
    cards
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use card_game::card::identity::signature_profile::{SignatureProfile, Tier};
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

    #[test]
    fn when_starter_deck_built_then_all_cards_are_face_up() {
        // Arrange
        let mut rng = ChaCha8Rng::seed_from_u64(0);

        // Act
        let deck = starter_deck(&mut rng);

        // Assert
        for card in &deck {
            assert!(card.face_up);
        }
    }

    #[test]
    fn when_starter_deck_built_then_dormant_row_has_all_dormant_tiers() {
        // Arrange
        let mut rng = ChaCha8Rng::seed_from_u64(0);

        // Act
        let deck = starter_deck(&mut rng);

        // Assert — first 5 cards are the dormant row
        for card in &deck[..5] {
            let profile = SignatureProfile::without_archetype(&card.signature);
            let tier = profile
                .dominant_axis
                .map_or(Tier::Dormant, |e| profile.tiers[e as usize]);
            assert_eq!(
                tier,
                Tier::Dormant,
                "dormant row card at {:?} has tier {tier:?}",
                card.position
            );
        }
    }

    #[test]
    fn when_starter_deck_built_then_active_row_has_all_active_tiers() {
        // Arrange
        let mut rng = ChaCha8Rng::seed_from_u64(0);

        // Act
        let deck = starter_deck(&mut rng);

        // Assert — cards 5..10 are the active row
        for card in &deck[5..10] {
            let profile = SignatureProfile::without_archetype(&card.signature);
            let tier = profile
                .dominant_axis
                .map_or(Tier::Active, |e| profile.tiers[e as usize]);
            assert_eq!(
                tier,
                Tier::Active,
                "active row card at {:?} has tier {tier:?}",
                card.position
            );
        }
    }

    #[test]
    fn when_starter_deck_built_then_intense_row_has_all_intense_tiers() {
        // Arrange
        let mut rng = ChaCha8Rng::seed_from_u64(0);

        // Act
        let deck = starter_deck(&mut rng);

        // Assert — cards 10..15 are the intense row
        for card in &deck[10..15] {
            let profile = SignatureProfile::without_archetype(&card.signature);
            let tier = profile
                .dominant_axis
                .map_or(Tier::Intense, |e| profile.tiers[e as usize]);
            assert_eq!(
                tier,
                Tier::Intense,
                "intense row card at {:?} has tier {tier:?}",
                card.position
            );
        }
    }
}
