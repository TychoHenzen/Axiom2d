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
