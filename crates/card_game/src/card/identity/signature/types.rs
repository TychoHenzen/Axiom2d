use serde::{Deserialize, Serialize};

use super::algorithms::{compute_seed, geometric_level};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RarityTierConfig {
    pub rarity_advance_rate: f32,
    pub tier_advance_rate: f32,
}

impl Default for RarityTierConfig {
    fn default() -> Self {
        Self {
            rarity_advance_rate: 0.3,
            tier_advance_rate: 0.3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Element {
    Solidum,
    Febris,
    Ordinem,
    Lumines,
    Varias,
    Inertiae,
    Subsidium,
    Spatium,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Aspect {
    // Solidum
    Solid,
    Fragile,
    // Febris
    Heat,
    Cold,
    // Ordinem
    Order,
    Chaos,
    // Lumines
    Light,
    Dark,
    // Varias
    Change,
    Stasis,
    // Inertiae
    Force,
    Calm,
    // Subsidium
    Growth,
    Decay,
    // Spatium
    Expansion,
    Contraction,
}

impl Element {
    pub const ALL: [Self; 8] = [
        Self::Solidum,
        Self::Febris,
        Self::Ordinem,
        Self::Lumines,
        Self::Varias,
        Self::Inertiae,
        Self::Subsidium,
        Self::Spatium,
    ];

    fn index(self) -> usize {
        self as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CardSignature {
    axes: [f32; 8],
}

impl Default for CardSignature {
    fn default() -> Self {
        Self { axes: [0.0; 8] }
    }
}

impl CardSignature {
    pub fn new(values: [f32; 8]) -> Self {
        let mut axes = values;
        for v in &mut axes {
            *v = v.clamp(-1.0, 1.0);
        }
        Self { axes }
    }

    pub fn random(rng: &mut rand_chacha::ChaCha8Rng) -> Self {
        use rand::Rng;

        let mut axes = [0.0; 8];
        for v in &mut axes {
            *v = rng.gen_range(-1.0..=1.0);
        }
        Self { axes }
    }

    pub fn subtract(&self, other: &Self) -> Self {
        let mut result = [0.0; 8];
        for (i, val) in result.iter_mut().enumerate() {
            *val = self.axes[i] - other.axes[i];
        }
        Self::new(result)
    }

    pub fn distance_to(&self, other: &Self) -> f32 {
        self.axes
            .iter()
            .zip(other.axes.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f32>()
            .sqrt()
    }

    pub fn intensity(&self, element: Element) -> f32 {
        self.axes[element.index()].abs()
    }

    pub fn dominant_aspect(&self, element: Element) -> Aspect {
        let value = self.axes[element.index()];
        if value > 0.0 {
            match element {
                Element::Solidum => Aspect::Solid,
                Element::Febris => Aspect::Heat,
                Element::Ordinem => Aspect::Order,
                Element::Lumines => Aspect::Light,
                Element::Varias => Aspect::Change,
                Element::Inertiae => Aspect::Force,
                Element::Subsidium => Aspect::Growth,
                Element::Spatium => Aspect::Expansion,
            }
        } else {
            match element {
                Element::Solidum => Aspect::Fragile,
                Element::Febris => Aspect::Cold,
                Element::Ordinem => Aspect::Chaos,
                Element::Lumines => Aspect::Dark,
                Element::Varias => Aspect::Stasis,
                Element::Inertiae => Aspect::Calm,
                Element::Subsidium => Aspect::Decay,
                Element::Spatium => Aspect::Contraction,
            }
        }
    }

    pub fn axes(&self) -> [f32; 8] {
        self.axes
    }

    pub fn rarity(&self) -> Rarity {
        self.rarity_with_config(&RarityTierConfig::default())
    }

    pub fn card_tier(&self) -> crate::card::identity::signature_profile::Tier {
        self.card_tier_with_config(&RarityTierConfig::default())
    }

    pub fn card_tier_with_config(
        &self,
        config: &RarityTierConfig,
    ) -> crate::card::identity::signature_profile::Tier {
        use crate::card::identity::signature_profile::Tier;

        let seed = compute_seed(self);
        let value = (seed >> 32) as f32 / u32::MAX as f32;
        let level = geometric_level(value, config.tier_advance_rate, 3);
        match level {
            0 => Tier::Dormant,
            1 => Tier::Active,
            _ => Tier::Intense,
        }
    }

    pub fn rarity_with_config(&self, config: &RarityTierConfig) -> Rarity {
        let seed = compute_seed(self);
        let value = (seed & 0xFFFF_FFFF) as f32 / u32::MAX as f32;
        let level = geometric_level(value, config.rarity_advance_rate, 5);
        match level {
            0 => Rarity::Common,
            1 => Rarity::Uncommon,
            2 => Rarity::Rare,
            3 => Rarity::Epic,
            _ => Rarity::Legendary,
        }
    }
}

impl std::ops::Index<Element> for CardSignature {
    type Output = f32;

    fn index(&self, element: Element) -> &f32 {
        &self.axes[element.index()]
    }
}
