use bevy_ecs::prelude::Component;
use engine_core::prelude::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Keyword {
    Taunt,
    Rush,
    Lifesteal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Legendary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardType {
    Creature,
    Spell,
    Artifact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParticleType {
    Sparks,
    Smoke,
    Embers,
    Frost,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CardStats {
    pub cost: u32,
    pub attack: u32,
    pub health: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Gradient {
    pub top: Color,
    pub bottom: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ArtShape {
    Circle { radius: f32 },
    Rectangle { width: f32, height: f32 },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArtDescriptor {
    pub background: Gradient,
    pub shapes: Vec<ArtShape>,
    pub effect: Option<ParticleType>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CardAbilities {
    pub keywords: Vec<Keyword>,
    pub text: String,
}

#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CardDefinition {
    pub card_type: CardType,
    pub rarity: Rarity,
    pub name: String,
    pub stats: Option<CardStats>,
    pub abilities: CardAbilities,
    pub art: ArtDescriptor,
}

impl std::fmt::Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Taunt => write!(f, "Taunt"),
            Self::Rush => write!(f, "Rush"),
            Self::Lifesteal => write!(f, "Lifesteal"),
        }
    }
}

pub fn description_from_abilities(abilities: &CardAbilities) -> String {
    let keyword_part: String = abilities
        .keywords
        .iter()
        .map(|k| format!("{k}."))
        .collect::<Vec<_>>()
        .join(" ");

    if keyword_part.is_empty() {
        abilities.text.clone()
    } else if abilities.text.is_empty() {
        keyword_part
    } else {
        format!("{keyword_part} {}", abilities.text)
    }
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardLayout {
    pub has_name_region: bool,
    pub has_art_region: bool,
    pub has_stats_bar: bool,
    pub has_description_region: bool,
}

pub fn card_type_layout(card_type: CardType) -> CardLayout {
    CardLayout {
        has_name_region: true,
        has_art_region: true,
        has_stats_bar: matches!(card_type, CardType::Creature),
        has_description_region: true,
    }
}

pub fn rarity_border_color(rarity: Rarity) -> Color {
    match rarity {
        Rarity::Common => Color::new(0.6, 0.6, 0.6, 1.0),
        Rarity::Uncommon => Color::new(0.2, 0.8, 0.2, 1.0),
        Rarity::Rare => Color::new(0.2, 0.4, 1.0, 1.0),
        Rarity::Legendary => Color::new(1.0, 0.65, 0.0, 1.0),
    }
}

pub fn art_descriptor_default(card_type: CardType) -> ArtDescriptor {
    let background = match card_type {
        CardType::Creature => Gradient {
            top: Color::new(0.8, 0.2, 0.1, 1.0),
            bottom: Color::new(0.3, 0.05, 0.0, 1.0),
        },
        CardType::Spell => Gradient {
            top: Color::new(0.1, 0.3, 0.9, 1.0),
            bottom: Color::new(0.0, 0.1, 0.4, 1.0),
        },
        CardType::Artifact => Gradient {
            top: Color::new(0.7, 0.7, 0.7, 1.0),
            bottom: Color::new(0.3, 0.25, 0.2, 1.0),
        },
    };
    ArtDescriptor {
        background,
        shapes: vec![],
        effect: None,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn when_keyword_serialized_to_ron_then_each_variant_roundtrips() {
        // Arrange
        let keywords = [Keyword::Taunt, Keyword::Rush, Keyword::Lifesteal];

        for keyword in keywords {
            // Act
            let ron = ron::to_string(&keyword).unwrap();
            let back: Keyword = ron::from_str(&ron).unwrap();

            // Assert
            assert_eq!(keyword, back);
        }
    }

    #[test]
    fn when_rarity_serialized_to_ron_then_each_variant_roundtrips() {
        // Arrange
        let rarities = [
            Rarity::Common,
            Rarity::Uncommon,
            Rarity::Rare,
            Rarity::Legendary,
        ];

        for rarity in rarities {
            // Act
            let ron = ron::to_string(&rarity).unwrap();
            let back: Rarity = ron::from_str(&ron).unwrap();

            // Assert
            assert_eq!(rarity, back);
        }
    }

    #[test]
    fn when_card_type_serialized_to_ron_then_each_variant_roundtrips() {
        // Arrange
        let card_types = [CardType::Creature, CardType::Spell, CardType::Artifact];

        for card_type in card_types {
            // Act
            let ron = ron::to_string(&card_type).unwrap();
            let back: CardType = ron::from_str(&ron).unwrap();

            // Assert
            assert_eq!(card_type, back);
        }
    }

    #[test]
    fn when_particle_type_serialized_to_ron_then_each_variant_roundtrips() {
        // Arrange
        let particle_types = [
            ParticleType::Sparks,
            ParticleType::Smoke,
            ParticleType::Embers,
            ParticleType::Frost,
        ];

        for particle_type in particle_types {
            // Act
            let ron = ron::to_string(&particle_type).unwrap();
            let back: ParticleType = ron::from_str(&ron).unwrap();

            // Assert
            assert_eq!(particle_type, back);
        }
    }

    #[test]
    fn when_card_stats_serialized_to_ron_then_cost_attack_health_roundtrip() {
        // Arrange
        let stats = CardStats {
            cost: 3,
            attack: 2,
            health: 5,
        };

        // Act
        let ron = ron::to_string(&stats).unwrap();
        let back: CardStats = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(stats, back);
    }

    #[test]
    fn when_gradient_serialized_to_ron_then_top_and_bottom_colors_roundtrip() {
        // Arrange
        let gradient = Gradient {
            top: Color::RED,
            bottom: Color::BLUE,
        };

        // Act
        let ron = ron::to_string(&gradient).unwrap();
        let back: Gradient = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(gradient, back);
    }

    #[test]
    fn when_art_descriptor_with_no_effect_serialized_to_ron_then_roundtrips() {
        // Arrange
        let art = ArtDescriptor {
            background: Gradient {
                top: Color::WHITE,
                bottom: Color::BLACK,
            },
            shapes: vec![],
            effect: None,
        };

        // Act
        let ron = ron::to_string(&art).unwrap();
        let back: ArtDescriptor = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(art, back);
    }

    #[test]
    fn when_art_descriptor_with_particle_effect_serialized_to_ron_then_effect_roundtrips() {
        // Arrange
        let art = ArtDescriptor {
            background: Gradient {
                top: Color::RED,
                bottom: Color::GREEN,
            },
            shapes: vec![],
            effect: Some(ParticleType::Sparks),
        };

        // Act
        let ron = ron::to_string(&art).unwrap();
        let back: ArtDescriptor = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(art, back);
    }

    #[test]
    fn when_card_abilities_with_no_keywords_serialized_to_ron_then_roundtrips() {
        // Arrange
        let abilities = CardAbilities {
            keywords: vec![],
            text: "Deal 3 damage to all enemies.".to_owned(),
        };

        // Act
        let ron = ron::to_string(&abilities).unwrap();
        let back: CardAbilities = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(abilities, back);
    }

    #[test]
    fn when_card_abilities_with_multiple_keywords_serialized_to_ron_then_roundtrips() {
        // Arrange
        let abilities = CardAbilities {
            keywords: vec![Keyword::Taunt, Keyword::Lifesteal],
            text: String::new(),
        };

        // Act
        let ron = ron::to_string(&abilities).unwrap();
        let back: CardAbilities = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(abilities, back);
    }

    #[test]
    fn when_creature_card_definition_serialized_to_ron_then_roundtrips() {
        // Arrange
        let def = CardDefinition {
            card_type: CardType::Creature,
            rarity: Rarity::Rare,
            name: "Fire Elemental".to_owned(),
            stats: Some(CardStats {
                cost: 5,
                attack: 6,
                health: 3,
            }),
            abilities: CardAbilities {
                keywords: vec![Keyword::Rush],
                text: "Deal 2 damage on entry.".to_owned(),
            },
            art: ArtDescriptor {
                background: Gradient {
                    top: Color::RED,
                    bottom: Color::BLACK,
                },
                shapes: vec![],
                effect: Some(ParticleType::Embers),
            },
        };

        // Act
        let ron = ron::to_string(&def).unwrap();
        let back: CardDefinition = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(def, back);
    }

    #[test]
    fn when_spell_card_definition_serialized_to_ron_then_stats_none_roundtrips() {
        // Arrange
        let def = CardDefinition {
            card_type: CardType::Spell,
            rarity: Rarity::Common,
            name: "Lightning Bolt".to_owned(),
            stats: None,
            abilities: CardAbilities {
                keywords: vec![],
                text: "Deal 3 damage.".to_owned(),
            },
            art: ArtDescriptor {
                background: Gradient {
                    top: Color::BLUE,
                    bottom: Color::WHITE,
                },
                shapes: vec![],
                effect: None,
            },
        };

        // Act
        let ron = ron::to_string(&def).unwrap();
        let back: CardDefinition = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(def, back);
        assert!(back.stats.is_none());
    }

    #[test]
    fn when_description_from_abilities_called_with_no_keywords_then_returns_freeform_text() {
        // Arrange
        let abilities = CardAbilities {
            keywords: vec![],
            text: "Draw two cards.".to_owned(),
        };

        // Act
        let desc = description_from_abilities(&abilities);

        // Assert
        assert_eq!(desc, "Draw two cards.");
    }

    #[test]
    fn when_description_from_abilities_called_with_keywords_only_then_returns_keyword_names() {
        // Arrange
        let abilities = CardAbilities {
            keywords: vec![Keyword::Taunt, Keyword::Rush],
            text: String::new(),
        };

        // Act
        let desc = description_from_abilities(&abilities);

        // Assert
        assert_eq!(desc, "Taunt. Rush.");
    }

    #[test]
    fn when_description_from_abilities_called_with_keywords_and_text_then_keywords_precede_freeform()
     {
        // Arrange
        let abilities = CardAbilities {
            keywords: vec![Keyword::Lifesteal],
            text: "Restore health equal to damage dealt.".to_owned(),
        };

        // Act
        let desc = description_from_abilities(&abilities);

        // Assert
        let keyword_pos = desc
            .find("Lifesteal")
            .expect("Lifesteal not found in output");
        let text_pos = desc
            .find("Restore health")
            .expect("freeform text not found in output");
        assert!(
            keyword_pos < text_pos,
            "expected 'Lifesteal' before 'Restore health', got: {desc:?}"
        );
        assert!(
            desc.contains('\n') || desc.contains(". ") || desc.contains(' '),
            "expected a separator between keyword and freeform text, got: {desc:?}"
        );
    }

    #[test]
    fn when_description_from_abilities_called_with_single_keyword_then_keyword_name_appears() {
        // Arrange
        let abilities = CardAbilities {
            keywords: vec![Keyword::Taunt],
            text: String::new(),
        };

        // Act
        let desc = description_from_abilities(&abilities);

        // Assert
        assert_eq!(desc, "Taunt.");
    }

    #[test]
    fn when_rarity_border_color_called_then_all_four_rarities_return_different_colors() {
        // Arrange
        let colors: Vec<Color> = [
            Rarity::Common,
            Rarity::Uncommon,
            Rarity::Rare,
            Rarity::Legendary,
        ]
        .iter()
        .map(|r| rarity_border_color(*r))
        .collect();

        // Assert
        for i in 0..colors.len() {
            for j in (i + 1)..colors.len() {
                assert_ne!(colors[i], colors[j], "rarities {i} and {j} share a color");
            }
        }
    }

    #[test]
    fn when_rarity_border_color_called_for_legendary_then_color_is_not_white_or_transparent() {
        // Act
        let color = rarity_border_color(Rarity::Legendary);

        // Assert
        assert_ne!(color, Color::WHITE);
        assert_ne!(color, Color::TRANSPARENT);
    }

    #[test]
    fn when_card_type_layout_called_for_creature_then_layout_includes_stats_bar() {
        // Act
        let layout = card_type_layout(CardType::Creature);

        // Assert
        assert!(layout.has_stats_bar);
    }

    #[test]
    fn when_card_type_layout_called_for_spell_then_layout_does_not_include_stats_bar() {
        // Act
        let layout = card_type_layout(CardType::Spell);

        // Assert
        assert!(!layout.has_stats_bar);
    }

    #[test]
    fn when_card_type_layout_called_for_artifact_then_layout_does_not_include_stats_bar() {
        // Act
        let layout = card_type_layout(CardType::Artifact);

        // Assert
        assert!(!layout.has_stats_bar);
    }

    #[test]
    fn when_card_type_layout_called_then_all_types_include_art_region_and_name_region() {
        // Arrange
        let types = [CardType::Creature, CardType::Spell, CardType::Artifact];

        for card_type in types {
            // Act
            let layout = card_type_layout(card_type);

            // Assert
            assert!(layout.has_art_region, "{card_type:?} missing art region");
            assert!(layout.has_name_region, "{card_type:?} missing name region");
        }
    }

    #[test]
    fn when_art_descriptor_default_called_for_creature_then_gradient_is_not_flat() {
        // Act
        let art = art_descriptor_default(CardType::Creature);

        // Assert
        assert_ne!(art.background.top, art.background.bottom);
    }

    #[test]
    fn when_art_descriptor_default_called_for_spell_then_gradient_differs_from_creature() {
        // Act
        let creature = art_descriptor_default(CardType::Creature);
        let spell = art_descriptor_default(CardType::Spell);

        // Assert
        assert!(
            creature.background.top != spell.background.top
                || creature.background.bottom != spell.background.bottom,
            "spell and creature should have distinct default gradients"
        );
    }

    #[test]
    fn when_art_descriptor_default_called_for_artifact_then_gradient_differs_from_creature() {
        // Act
        let creature = art_descriptor_default(CardType::Creature);
        let artifact = art_descriptor_default(CardType::Artifact);

        // Assert
        assert!(
            creature.background.top != artifact.background.top
                || creature.background.bottom != artifact.background.bottom,
            "artifact and creature should have distinct default gradients"
        );
    }

    #[test]
    fn when_card_definition_is_creature_with_none_stats_then_stats_is_none() {
        // Arrange
        let def = CardDefinition {
            card_type: CardType::Creature,
            rarity: Rarity::Rare,
            name: "Token".to_owned(),
            stats: None,
            abilities: CardAbilities {
                keywords: vec![],
                text: String::new(),
            },
            art: art_descriptor_default(CardType::Creature),
        };

        // Assert
        assert!(def.stats.is_none());
    }

    #[test]
    fn when_card_definition_is_spell_with_some_stats_then_stats_is_some() {
        // Arrange
        let def = CardDefinition {
            card_type: CardType::Spell,
            rarity: Rarity::Common,
            name: "Fireball".to_owned(),
            stats: Some(CardStats {
                cost: 4,
                attack: 0,
                health: 0,
            }),
            abilities: CardAbilities {
                keywords: vec![],
                text: "Deal 6 damage.".to_owned(),
            },
            art: art_descriptor_default(CardType::Spell),
        };

        // Assert
        assert!(def.stats.is_some());
    }

    proptest::proptest! {
        #[test]
        fn when_any_card_definition_serialized_to_ron_then_roundtrips(
            card_type_idx in 0_u8..3,
            rarity_idx in 0_u8..4,
            keyword_count in 0_usize..4,
            keyword_bits in 0_u8..8,
            cost in proptest::num::u32::ANY,
            attack in proptest::num::u32::ANY,
            health in proptest::num::u32::ANY,
            has_stats in proptest::bool::ANY,
            r1 in 0.0_f32..=1.0,
            g1 in 0.0_f32..=1.0,
            b1 in 0.0_f32..=1.0,
            r2 in 0.0_f32..=1.0,
            g2 in 0.0_f32..=1.0,
            b2 in 0.0_f32..=1.0,
            particle_idx in 0_u8..5,
        ) {
            // Arrange
            let card_type = match card_type_idx {
                0 => CardType::Creature,
                1 => CardType::Spell,
                _ => CardType::Artifact,
            };
            let rarity = match rarity_idx {
                0 => Rarity::Common,
                1 => Rarity::Uncommon,
                2 => Rarity::Rare,
                _ => Rarity::Legendary,
            };
            let all_keywords = [Keyword::Taunt, Keyword::Rush, Keyword::Lifesteal];
            let keywords: Vec<Keyword> = (0..keyword_count.min(3))
                .filter(|i| keyword_bits & (1 << i) != 0)
                .map(|i| all_keywords[i])
                .collect();
            let effect = match particle_idx {
                0 => None,
                1 => Some(ParticleType::Sparks),
                2 => Some(ParticleType::Smoke),
                3 => Some(ParticleType::Embers),
                _ => Some(ParticleType::Frost),
            };
            let def = CardDefinition {
                card_type,
                rarity,
                name: "Test Card".to_owned(),
                stats: if has_stats {
                    Some(CardStats { cost, attack, health })
                } else {
                    None
                },
                abilities: CardAbilities {
                    keywords,
                    text: "Some effect.".to_owned(),
                },
                art: ArtDescriptor {
                    background: Gradient {
                        top: Color::new(r1, g1, b1, 1.0),
                        bottom: Color::new(r2, g2, b2, 1.0),
                    },
                    shapes: vec![],
                    effect,
                },
            };

            // Act
            let ron_str = ron::to_string(&def).unwrap();
            let back: CardDefinition = ron::from_str(&ron_str).unwrap();

            // Assert
            assert_eq!(def, back);
        }
    }
}
