use bevy_ecs::component::Component;

use crate::card::identity::base_type::BaseCardTypeRegistry;
use crate::card::identity::signature::{Aspect, CardSignature, Element, Rarity};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tier {
    Dormant,
    Active,
    Intense,
}

const ELEMENTS: [Element; 8] = [
    Element::Solidum,
    Element::Febris,
    Element::Ordinem,
    Element::Lumines,
    Element::Varias,
    Element::Inertiae,
    Element::Subsidium,
    Element::Spatium,
];

#[derive(Component, Debug, Clone, PartialEq)]
pub struct SignatureProfile {
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
        let tiers = ELEMENTS.map(|element| match signature.intensity(element) {
            i if i >= 0.7 => Tier::Intense,
            i if i >= 0.3 => Tier::Active,
            _ => Tier::Dormant,
        });
        let aspects = ELEMENTS.map(|element| signature.dominant_aspect(element));

        let mut intensities: [(Element, f32); 8] = ELEMENTS.map(|e| (e, signature.intensity(e)));
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
            tiers,
            aspects,
            dominant_axis,
            secondary_axis,
            rarity: signature.rarity(),
            archetype: None,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::card::identity::base_type::BaseCardTypeRegistry;
    use crate::card::identity::signature::{Aspect, CardSignature, Element};

    fn sig_with_axis(index: usize, value: f32) -> CardSignature {
        let mut axes = [0.0_f32; 8];
        axes[index] = value;
        CardSignature::new(axes)
    }

    fn sig_with_two_axes(idx_a: usize, val_a: f32, idx_b: usize, val_b: f32) -> CardSignature {
        let mut axes = [0.0_f32; 8];
        axes[idx_a] = val_a;
        axes[idx_b] = val_b;
        CardSignature::new(axes)
    }

    fn profile_of(sig: &CardSignature) -> SignatureProfile {
        SignatureProfile::without_archetype(sig)
    }

    #[test]
    fn when_axis_below_active_threshold_then_tier_is_dormant() {
        // Arrange
        let boundary = sig_with_axis(0, 0.29);
        let zero = CardSignature::new([0.0; 8]);

        // Act
        let profile_boundary = profile_of(&boundary);
        let profile_zero = profile_of(&zero);

        // Assert
        assert_eq!(
            profile_boundary.tiers[0],
            Tier::Dormant,
            "intensity 0.29 should be Dormant"
        );
        assert_eq!(
            profile_zero.tiers[0],
            Tier::Dormant,
            "intensity 0.0 should be Dormant"
        );
    }

    #[test]
    fn when_axis_between_thresholds_then_tier_is_active() {
        // Arrange
        let low = sig_with_axis(1, 0.3);
        let mid = sig_with_axis(1, 0.5);
        let high = sig_with_axis(1, 0.69);

        // Act
        let profile_low = profile_of(&low);
        let profile_mid = profile_of(&mid);
        let profile_high = profile_of(&high);

        // Assert
        assert_eq!(
            profile_low.tiers[1],
            Tier::Active,
            "intensity 0.3 should be Active"
        );
        assert_eq!(
            profile_mid.tiers[1],
            Tier::Active,
            "intensity 0.5 should be Active"
        );
        assert_eq!(
            profile_high.tiers[1],
            Tier::Active,
            "intensity 0.69 should be Active"
        );
    }

    #[test]
    fn when_axis_at_or_above_intense_threshold_then_tier_is_intense() {
        // Arrange
        let at_threshold = sig_with_axis(2, 0.7);
        let at_max = sig_with_axis(2, 1.0);

        // Act
        let profile_threshold = profile_of(&at_threshold);
        let profile_max = profile_of(&at_max);

        // Assert
        assert_eq!(
            profile_threshold.tiers[2],
            Tier::Intense,
            "intensity 0.7 should be Intense"
        );
        assert_eq!(
            profile_max.tiers[2],
            Tier::Intense,
            "intensity 1.0 should be Intense"
        );
    }

    #[test]
    fn when_axis_is_negative_then_tier_uses_absolute_value() {
        // Arrange
        let sig = sig_with_axis(5, -0.8);

        // Act
        let profile = profile_of(&sig);

        // Assert
        assert_eq!(
            profile.tiers[5],
            Tier::Intense,
            "intensity abs(-0.8) = 0.8 should be Intense, not Dormant"
        );
    }

    #[test]
    fn when_all_axes_zero_then_all_tiers_are_dormant() {
        // Arrange
        let sig = CardSignature::new([0.0; 8]);

        // Act
        let profile = profile_of(&sig);

        // Assert
        for (i, &tier) in profile.tiers.iter().enumerate() {
            assert_eq!(tier, Tier::Dormant, "axis {i} should be Dormant when zero");
        }
    }

    #[test]
    fn when_positive_axis_then_aspect_is_positive_variant() {
        // Arrange
        let sig = sig_with_axis(Element::Febris as usize, 0.6);

        // Act
        let profile = profile_of(&sig);

        // Assert
        assert_eq!(
            profile.aspects[Element::Febris as usize],
            Aspect::Heat,
            "Febris axis +0.6 should produce Aspect::Heat"
        );
    }

    #[test]
    fn when_negative_axis_then_aspect_is_negative_variant() {
        // Arrange
        let sig = sig_with_axis(Element::Subsidium as usize, -0.5);

        // Act
        let profile = profile_of(&sig);

        // Assert
        assert_eq!(
            profile.aspects[Element::Subsidium as usize],
            Aspect::Decay,
            "Subsidium axis -0.5 should produce Aspect::Decay"
        );
    }

    #[test]
    fn when_all_axes_positive_then_all_aspects_are_positive_variants() {
        // Arrange
        let sig = CardSignature::new([0.5; 8]);
        let expected: [(Element, Aspect); 8] = [
            (Element::Solidum, Aspect::Solid),
            (Element::Febris, Aspect::Heat),
            (Element::Ordinem, Aspect::Order),
            (Element::Lumines, Aspect::Light),
            (Element::Varias, Aspect::Change),
            (Element::Inertiae, Aspect::Force),
            (Element::Subsidium, Aspect::Growth),
            (Element::Spatium, Aspect::Expansion),
        ];

        // Act
        let profile = profile_of(&sig);

        // Assert
        for (element, aspect) in expected {
            assert_eq!(
                profile.aspects[element as usize], aspect,
                "{element:?} axis +0.5 should produce {aspect:?}"
            );
        }
    }

    #[test]
    fn when_one_axis_clearly_highest_then_dominant_is_that_element() {
        // Arrange
        let mut axes = [0.1_f32; 8];
        axes[Element::Febris as usize] = 0.9;
        let sig = CardSignature::new(axes);

        // Act
        let profile = profile_of(&sig);

        // Assert
        assert_eq!(
            profile.dominant_axis,
            Some(Element::Febris),
            "Febris at 0.9 should be dominant over all others at 0.1"
        );
    }

    #[test]
    fn when_dominant_axis_is_negative_then_dominant_is_still_correct() {
        // Arrange
        let mut axes = [0.1_f32; 8];
        axes[Element::Spatium as usize] = -0.95;
        let sig = CardSignature::new(axes);

        // Act
        let profile = profile_of(&sig);

        // Assert
        assert_eq!(
            profile.dominant_axis,
            Some(Element::Spatium),
            "Spatium at -0.95 (abs=0.95) should be dominant over all others at 0.1"
        );
    }

    #[test]
    fn when_all_axes_zero_then_dominant_is_first_element() {
        // Arrange
        let sig = CardSignature::new([0.0; 8]);

        // Act
        let profile = profile_of(&sig);

        // Assert
        assert_eq!(
            profile.dominant_axis,
            Some(Element::Solidum),
            "all-zero signature should resolve tie to index-0 element (Solidum)"
        );
    }

    #[test]
    fn when_two_axes_tied_for_highest_then_dominant_is_lower_index_element() {
        // Arrange
        let sig = sig_with_two_axes(
            Element::Solidum as usize,
            0.8,
            Element::Febris as usize,
            0.8,
        );

        // Act
        let profile = profile_of(&sig);

        // Assert
        assert_eq!(
            profile.dominant_axis,
            Some(Element::Solidum),
            "Solidum (index 0) ties with Febris (index 1) — lower index wins"
        );
    }

    #[test]
    fn when_top_axis_exceeds_1_5x_second_then_secondary_is_none() {
        // Arrange — Solidum=0.9, Febris=0.4, ratio=2.25 > 1.5
        let sig = sig_with_two_axes(
            Element::Solidum as usize,
            0.9,
            Element::Febris as usize,
            0.4,
        );

        // Act
        let profile = profile_of(&sig);

        // Assert
        assert_eq!(
            profile.secondary_axis, None,
            "ratio 0.9/0.4=2.25 exceeds 1.5x — secondary should be suppressed"
        );
    }

    #[test]
    fn when_top_axis_within_1_5x_of_second_then_secondary_is_present() {
        // Arrange — Solidum=0.9, Febris=0.7, ratio=1.286 < 1.5
        let sig = sig_with_two_axes(
            Element::Solidum as usize,
            0.9,
            Element::Febris as usize,
            0.7,
        );

        // Act
        let profile = profile_of(&sig);

        // Assert
        assert_eq!(
            profile.secondary_axis,
            Some(Element::Febris),
            "ratio 0.9/0.7≈1.29 is within 1.5x — secondary should be present"
        );
    }

    #[test]
    fn when_ratio_is_exactly_1_5x_then_secondary_is_none() {
        // Arrange — Ordinem=0.75, Lumines=0.5, ratio=1.5 exactly
        // Uses f32-exact values (0.75 and 0.5 are representable in IEEE 754)
        let sig = sig_with_two_axes(
            Element::Ordinem as usize,
            0.75,
            Element::Lumines as usize,
            0.5,
        );

        // Act
        let profile = profile_of(&sig);

        // Assert
        assert_eq!(
            profile.secondary_axis, None,
            "ratio 0.75/0.5=1.5 exactly — boundary treated as suppressed (secondary=None)"
        );
    }

    #[test]
    fn when_secondary_axis_is_negative_then_still_identified() {
        // Arrange — Varias=0.85, Inertiae=-0.65, ratio≈1.31 < 1.5
        let sig = sig_with_two_axes(
            Element::Varias as usize,
            0.85,
            Element::Inertiae as usize,
            -0.65,
        );

        // Act
        let profile = profile_of(&sig);

        // Assert
        assert_eq!(
            profile.secondary_axis,
            Some(Element::Inertiae),
            "Inertiae at -0.65 (abs=0.65) ratio 1.31 < 1.5 — negative secondary should still be identified"
        );
    }

    #[test]
    fn when_signature_has_known_rarity_then_profile_stores_same_value() {
        // Arrange — all axes at 1.0 should produce Legendary
        let sig = CardSignature::new([1.0; 8]);

        // Act
        let profile = profile_of(&sig);

        // Assert
        assert_eq!(
            profile.rarity,
            sig.rarity(),
            "profile rarity should match CardSignature::rarity()"
        );
    }

    #[test]
    fn when_signature_matches_registry_type_then_archetype_is_that_type() {
        // Arrange
        let mut registry = BaseCardTypeRegistry::default();
        crate::card::identity::base_type::populate_default_types(&mut registry);
        let sig = CardSignature::new([0.8, 0.3, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);

        // Act
        let profile = SignatureProfile::new(&sig, &registry);

        // Assert
        assert_eq!(
            profile.archetype.as_deref(),
            Some("Weapon"),
            "signature matching Weapon prototype should produce archetype 'Weapon'"
        );
    }

    #[test]
    fn when_signature_matches_no_registry_type_then_archetype_is_none() {
        // Arrange — use a tiny match_radius so nothing matches a distant signature
        use crate::card::identity::base_type::BaseCardType;
        let mut registry = BaseCardTypeRegistry::default();
        registry.register(BaseCardType {
            name: "Narrow".to_string(),
            base_signature: CardSignature::new([1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
            match_radius: 0.1,
            category: crate::card::identity::base_type::CardCategory::Equipment,
            modifiers: vec![],
        });
        let sig = CardSignature::new([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0]);

        // Act
        let profile = SignatureProfile::new(&sig, &registry);

        // Assert
        assert_eq!(
            profile.archetype, None,
            "signature far from all types should have archetype None"
        );
    }

    #[test]
    fn when_registry_is_empty_then_archetype_is_none() {
        // Arrange
        let sig = CardSignature::new([0.5; 8]);

        // Act
        let profile = profile_of(&sig);

        // Assert
        assert_eq!(
            profile.archetype, None,
            "empty registry should always produce archetype None"
        );
    }

    #[test]
    fn when_two_archetypes_match_then_archetype_is_closest() {
        // Arrange
        let mut registry = BaseCardTypeRegistry::default();
        crate::card::identity::base_type::populate_default_types(&mut registry);
        let sig = CardSignature::new([0.75, 0.35, 0.05, 0.0, 0.0, 0.0, 0.0, 0.0]);

        // Act
        let profile = SignatureProfile::new(&sig, &registry);

        // Assert
        assert_eq!(
            profile.archetype.as_deref(),
            Some("Weapon"),
            "signature closest to Weapon should produce archetype 'Weapon'"
        );
    }

    #[test]
    fn when_profile_constructed_then_all_fields_are_populated() {
        // Arrange — Solidum=0.8 (Intense), Febris=0.6 (Active), ratio 1.33 < 1.5
        let mut registry = BaseCardTypeRegistry::default();
        crate::card::identity::base_type::populate_default_types(&mut registry);
        let sig = CardSignature::new([0.8, 0.6, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1]);

        // Act
        let profile = SignatureProfile::new(&sig, &registry);

        // Assert — tiers
        assert_eq!(profile.tiers[0], Tier::Intense, "Solidum 0.8 → Intense");
        assert_eq!(profile.tiers[1], Tier::Active, "Febris 0.6 → Active");
        for i in 2..8 {
            assert_eq!(profile.tiers[i], Tier::Dormant, "axis {i} at 0.1 → Dormant");
        }

        // Assert — aspects (all positive)
        assert_eq!(profile.aspects[0], Aspect::Solid);
        assert_eq!(profile.aspects[1], Aspect::Heat);

        // Assert — dominant/secondary
        assert_eq!(profile.dominant_axis, Some(Element::Solidum));
        assert_eq!(profile.secondary_axis, Some(Element::Febris));

        // Assert — rarity
        assert_eq!(profile.rarity, sig.rarity());

        // Assert — archetype
        assert!(
            profile.archetype.is_some(),
            "should match a registered archetype"
        );
    }
}
