// EVOLVE-BLOCK-START
use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

use super::base_type::BaseCardType;
use super::signature::{CardSignature, Element};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModifierType {
    Power,
    Cost,
    Duration,
    Range,
    Healing,
    Speed,
    Defense,
    Special,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidualModifier {
    pub source_element: Element,
    pub modifier_type: ModifierType,
    pub intensity: f32,
    pub use_positive: bool,
}

impl ResidualModifier {
    pub fn calculate_effect(&self, residual: &CardSignature) -> f32 {
        let value = residual[self.source_element];
        if self.use_positive && value <= 0.0 {
            return 0.0;
        }
        if !self.use_positive && value >= 0.0 {
            return 0.0;
        }
        value * self.intensity
    }
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct ResidualStats {
    pub power: f32,
    pub cost: f32,
    pub duration: f32,
    pub range: f32,
    pub healing: f32,
    pub speed: f32,
    pub defense: f32,
    pub special: f32,
}

impl ResidualStats {
    pub fn from_card(signature: &CardSignature, base_type: &BaseCardType) -> Self {
        let residual = signature.subtract(&base_type.base_signature);
        Self::compute(&residual, &base_type.modifiers)
    }

    pub fn compute(residual: &CardSignature, modifiers: &[ResidualModifier]) -> Self {
        let mut stats = Self {
            power: 0.0,
            cost: 0.0,
            duration: 0.0,
            range: 0.0,
            healing: 0.0,
            speed: 0.0,
            defense: 0.0,
            special: 0.0,
        };
        for modifier in modifiers {
            let effect = modifier.calculate_effect(residual);
            match modifier.modifier_type {
                ModifierType::Power => stats.power += effect,
                ModifierType::Cost => stats.cost += effect,
                ModifierType::Duration => stats.duration += effect,
                ModifierType::Range => stats.range += effect,
                ModifierType::Healing => stats.healing += effect,
                ModifierType::Speed => stats.speed += effect,
                ModifierType::Defense => stats.defense += effect,
                ModifierType::Special => stats.special += effect,
            }
        }
        stats
    }
}
// EVOLVE-BLOCK-END
