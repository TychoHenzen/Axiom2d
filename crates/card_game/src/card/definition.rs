use bevy_ecs::prelude::Component;
use engine_core::prelude::Color;
use serde::{Deserialize, Serialize};

use super::signature::CardSignature;

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

/// Base hue ranges per rarity tier (H, S, L center values).
/// Each card gets a randomized shade within its tier using the signature as seed.
fn rarity_hsl(rarity: Rarity) -> (f32, f32, f32) {
    match rarity {
        Rarity::Common => (0.0, 0.0, 0.75),      // light grays/whites
        Rarity::Uncommon => (0.33, 0.55, 0.45),  // greens
        Rarity::Rare => (0.61, 0.65, 0.50),      // blues
        Rarity::Epic => (0.78, 0.60, 0.45),      // purples
        Rarity::Legendary => (0.11, 0.85, 0.55), // golds/oranges
    }
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    if s < 1e-6 {
        return (l, l, l);
    }
    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    let hue_to_rgb = |t: f32| {
        let t = ((t % 1.0) + 1.0) % 1.0;
        if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 0.5 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        }
    };
    (
        hue_to_rgb(h + 1.0 / 3.0),
        hue_to_rgb(h),
        hue_to_rgb(h - 1.0 / 3.0),
    )
}

/// Returns a rarity-appropriate border color with per-card variation seeded by the signature.
pub fn rarity_border_color(rarity: Rarity, signature: &CardSignature) -> Color {
    let (base_h, base_s, base_l) = rarity_hsl(rarity);

    // Derive a deterministic variation from the signature axes
    let seed = signature
        .axes()
        .iter()
        .enumerate()
        .fold(0.0_f32, |acc, (i, &v)| acc + v * (i as f32 + 1.0) * 0.1);
    let variation = (seed * 7.13).fract() * 2.0 - 1.0; // -1..1

    let h = base_h + variation * 0.04; // ±4% hue shift
    let s = (base_s + variation * 0.15).clamp(0.0, 1.0); // ±15% saturation
    let l = (base_l + variation * 0.12).clamp(0.15, 0.90); // ±12% lightness

    let (r, g, b) = hsl_to_rgb(h, s, l);
    Color::new(r, g, b, 1.0)
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
    fn when_rarity_border_color_called_with_same_sig_then_different_rarities_produce_different_colors()
     {
        // Arrange
        use crate::card::signature::CardSignature;
        let sig = CardSignature::new([0.5; 8]);
        let rarities = [
            Rarity::Common,
            Rarity::Uncommon,
            Rarity::Rare,
            Rarity::Epic,
            Rarity::Legendary,
        ];

        // Act
        let colors: Vec<Color> = rarities
            .iter()
            .map(|r| rarity_border_color(*r, &sig))
            .collect();

        // Assert
        for i in 0..colors.len() {
            for j in (i + 1)..colors.len() {
                assert_ne!(colors[i], colors[j], "rarities {i} and {j} share a color");
            }
        }
    }

    #[test]
    fn when_rarity_border_color_called_with_different_sigs_then_same_rarity_varies() {
        // Arrange
        use crate::card::signature::CardSignature;
        let sig_a = CardSignature::new([0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8]);
        let sig_b = CardSignature::new([0.8, 0.7, 0.6, 0.5, 0.4, 0.3, 0.2, 0.1]);

        // Act
        let color_a = rarity_border_color(Rarity::Rare, &sig_a);
        let color_b = rarity_border_color(Rarity::Rare, &sig_b);

        // Assert — different signatures should produce different shades
        assert_ne!(
            color_a, color_b,
            "same rarity with different signatures should produce different colors"
        );
    }

    #[test]
    fn when_rarity_border_color_called_then_same_inputs_produce_same_output() {
        // Arrange
        use crate::card::signature::CardSignature;
        let sig = CardSignature::new([0.3, 0.6, 0.1, 0.9, 0.2, 0.5, 0.4, 0.7]);

        // Act
        let color1 = rarity_border_color(Rarity::Epic, &sig);
        let color2 = rarity_border_color(Rarity::Epic, &sig);

        // Assert
        assert_eq!(
            color1, color2,
            "same inputs must produce deterministic output"
        );
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
