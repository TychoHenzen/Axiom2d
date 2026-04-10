// EVOLVE-BLOCK-START
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
// EVOLVE-BLOCK-END
