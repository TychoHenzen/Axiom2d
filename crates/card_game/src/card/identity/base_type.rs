use bevy_ecs::prelude::Resource;
use serde::{Deserialize, Serialize};

use super::residual::{ModifierType, ResidualModifier};
use super::signature::{CardSignature, Element};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardCategory {
    Equipment,
    Skill,
    Playstyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseCardType {
    pub name: String,
    pub base_signature: CardSignature,
    pub match_radius: f32,
    pub category: CardCategory,
    pub modifiers: Vec<ResidualModifier>,
}

impl BaseCardType {
    pub fn can_match(&self, signature: &CardSignature) -> bool {
        self.base_signature.distance_to(signature) <= self.match_radius
    }

    pub fn match_weight(&self, signature: &CardSignature) -> f32 {
        let distance = self.base_signature.distance_to(signature);
        (1.0 - distance / self.match_radius).max(0.0)
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct BaseCardTypeRegistry {
    types: Vec<BaseCardType>,
}

impl BaseCardTypeRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, base_type: BaseCardType) {
        self.types.push(base_type);
    }

    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }

    #[must_use]
    pub fn best_match(&self, signature: &CardSignature) -> Option<&BaseCardType> {
        self.types
            .iter()
            .filter_map(|bt| {
                let w = bt.match_weight(signature);
                if w > 0.0 { Some((bt, w)) } else { None }
            })
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(bt, _)| bt)
    }
}

pub fn populate_default_types(registry: &mut BaseCardTypeRegistry) {
    registry.register(BaseCardType {
        name: "Weapon".to_string(),
        base_signature: CardSignature::new([0.8, 0.3, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
        match_radius: 1.5,
        category: CardCategory::Equipment,
        modifiers: vec![
            ResidualModifier {
                source_element: Element::Solidum,
                modifier_type: ModifierType::Power,
                intensity: 2.0,
                use_positive: true,
            },
            ResidualModifier {
                source_element: Element::Febris,
                modifier_type: ModifierType::Speed,
                intensity: 1.0,
                use_positive: true,
            },
        ],
    });
    registry.register(BaseCardType {
        name: "Spell".to_string(),
        base_signature: CardSignature::new([0.0, 0.8, 0.0, 0.3, 0.0, 0.0, 0.0, 0.0]),
        match_radius: 1.5,
        category: CardCategory::Skill,
        modifiers: vec![
            ResidualModifier {
                source_element: Element::Febris,
                modifier_type: ModifierType::Power,
                intensity: 1.5,
                use_positive: true,
            },
            ResidualModifier {
                source_element: Element::Lumines,
                modifier_type: ModifierType::Range,
                intensity: 1.0,
                use_positive: true,
            },
        ],
    });
    registry.register(BaseCardType {
        name: "Shield".to_string(),
        base_signature: CardSignature::new([0.8, 0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0]),
        match_radius: 1.5,
        category: CardCategory::Equipment,
        modifiers: vec![
            ResidualModifier {
                source_element: Element::Solidum,
                modifier_type: ModifierType::Defense,
                intensity: 2.0,
                use_positive: true,
            },
            ResidualModifier {
                source_element: Element::Ordinem,
                modifier_type: ModifierType::Duration,
                intensity: 1.0,
                use_positive: true,
            },
        ],
    });
    registry.register(BaseCardType {
        name: "Healer".to_string(),
        base_signature: CardSignature::new([0.0, 0.0, 0.0, 0.3, 0.0, 0.0, 0.8, 0.0]),
        match_radius: 1.5,
        category: CardCategory::Skill,
        modifiers: vec![
            ResidualModifier {
                source_element: Element::Subsidium,
                modifier_type: ModifierType::Healing,
                intensity: 2.0,
                use_positive: true,
            },
            ResidualModifier {
                source_element: Element::Lumines,
                modifier_type: ModifierType::Duration,
                intensity: 1.0,
                use_positive: true,
            },
        ],
    });
    registry.register(BaseCardType {
        name: "Scout".to_string(),
        base_signature: CardSignature::new([0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.8]),
        match_radius: 2.0,
        category: CardCategory::Playstyle,
        modifiers: vec![
            ResidualModifier {
                source_element: Element::Spatium,
                modifier_type: ModifierType::Speed,
                intensity: 1.5,
                use_positive: true,
            },
            ResidualModifier {
                source_element: Element::Inertiae,
                modifier_type: ModifierType::Range,
                intensity: 1.0,
                use_positive: true,
            },
        ],
    });
}
