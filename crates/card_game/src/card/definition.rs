use bevy_ecs::prelude::Component;
use engine_core::prelude::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Keyword {
    Taunt,
    Rush,
    Lifesteal,
}

pub use super::signature::Rarity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardType {
    Creature,
    Spell,
    Artifact,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArtDescriptor {
    pub background: Gradient,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CardAbilities {
    pub keywords: Vec<Keyword>,
    pub text: String,
}

#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CardDefinition {
    pub card_type: CardType,
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

pub fn rarity_border_color(rarity: Rarity) -> Color {
    match rarity {
        Rarity::Common => Color::new(0.6, 0.6, 0.6, 1.0),
        Rarity::Uncommon => Color::new(0.2, 0.8, 0.2, 1.0),
        Rarity::Rare => Color::new(0.2, 0.4, 1.0, 1.0),
        Rarity::Epic => Color::new(0.6, 0.2, 0.9, 1.0),
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
    ArtDescriptor { background }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

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
    fn when_rarity_border_color_called_then_all_five_rarities_return_different_colors() {
        // Arrange
        let colors: Vec<Color> = [
            Rarity::Common,
            Rarity::Uncommon,
            Rarity::Rare,
            Rarity::Epic,
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
}
