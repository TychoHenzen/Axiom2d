use card_game::prelude::*;
use glam::Vec2;

pub struct StarterCard {
    pub definition: CardDefinition,
    pub position: Vec2,
    pub face_up: bool,
}

pub fn starter_deck() -> Vec<StarterCard> {
    vec![
        StarterCard {
            definition: CardDefinition {
                card_type: CardType::Spell,
                rarity: Rarity::Rare,
                name: "Fireball".to_owned(),
                stats: None,
                abilities: CardAbilities {
                    keywords: vec![],
                    text: "Deal 3 damage".to_owned(),
                },
                art: art_descriptor_default(CardType::Spell),
            },
            position: Vec2::new(-120.0, 0.0),
            face_up: false,
        },
        StarterCard {
            definition: CardDefinition {
                card_type: CardType::Artifact,
                rarity: Rarity::Common,
                name: "Shield".to_owned(),
                stats: None,
                abilities: CardAbilities {
                    keywords: vec![],
                    text: "Block 2 damage".to_owned(),
                },
                art: art_descriptor_default(CardType::Artifact),
            },
            position: Vec2::new(-60.0, 30.0),
            face_up: false,
        },
        StarterCard {
            definition: CardDefinition {
                card_type: CardType::Creature,
                rarity: Rarity::Legendary,
                name: "Heal".to_owned(),
                stats: Some(CardStats {
                    cost: 3,
                    attack: 2,
                    health: 4,
                }),
                abilities: CardAbilities {
                    keywords: vec![Keyword::Lifesteal],
                    text: "Restore 4 HP".to_owned(),
                },
                art: art_descriptor_default(CardType::Creature),
            },
            position: Vec2::new(0.0, 0.0),
            face_up: true,
        },
        StarterCard {
            definition: CardDefinition {
                card_type: CardType::Spell,
                rarity: Rarity::Uncommon,
                name: "Lightning".to_owned(),
                stats: None,
                abilities: CardAbilities {
                    keywords: vec![],
                    text: "Deal 5 damage".to_owned(),
                },
                art: art_descriptor_default(CardType::Spell),
            },
            position: Vec2::new(60.0, -20.0),
            face_up: false,
        },
        StarterCard {
            definition: CardDefinition {
                card_type: CardType::Spell,
                rarity: Rarity::Common,
                name: "Draw".to_owned(),
                stats: None,
                abilities: CardAbilities {
                    keywords: vec![],
                    text: "Draw 2 cards".to_owned(),
                },
                art: art_descriptor_default(CardType::Spell),
            },
            position: Vec2::new(120.0, 10.0),
            face_up: false,
        },
    ]
}
