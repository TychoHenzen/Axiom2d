// EVOLVE-BLOCK-START
use bevy_ecs::component::Component;

use crate::card::identity::base_type::BaseCardTypeRegistry;
use crate::card::identity::signature::{Aspect, CardSignature, Element, Rarity};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tier {
    Dormant,
    Active,
    Intense,
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct SignatureProfile {
    pub tier: Tier,
    pub tiers: [Tier; 8],
    pub aspects: [Aspect; 8],
    pub dominant_axis: Option<Element>,
    pub secondary_axis: Option<Element>,
    pub rarity: Rarity,
    pub archetype: Option<String>,
}

impl SignatureProfile {
    pub fn new(signature: &CardSignature, registry: &BaseCardTypeRegistry) -> Self {
        let mut profile = Self::without_archetype(signature);
        profile.archetype = registry.best_match(signature).map(|bt| bt.name.clone());
        profile
    }

    pub fn without_archetype(signature: &CardSignature) -> Self {
        let tiers = Element::ALL.map(|element| match signature.intensity(element) {
            i if i >= 0.7 => Tier::Intense,
            i if i >= 0.3 => Tier::Active,
            _ => Tier::Dormant,
        });
        let aspects = Element::ALL.map(|element| signature.dominant_aspect(element));

        let mut intensities: [(Element, f32); 8] =
            Element::ALL.map(|e| (e, signature.intensity(e)));
        intensities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let dominant_axis = Some(intensities[0].0);
        let secondary_axis = {
            let top = intensities[0].1;
            let second = intensities[1].1;
            if second > 0.0 && top < 1.5 * second {
                Some(intensities[1].0)
            } else {
                None
            }
        };

        Self {
            tier: signature.card_tier(),
            tiers,
            aspects,
            dominant_axis,
            secondary_axis,
            rarity: signature.rarity(),
            archetype: None,
        }
    }
}
// EVOLVE-BLOCK-END
