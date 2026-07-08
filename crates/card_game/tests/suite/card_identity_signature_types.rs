#![allow(clippy::unwrap_used)]

use card_game::card::identity::signature::{
    Aspect, CardSignature, Element, Rarity, RarityTierConfig,
};
use card_game::card::identity::signature_profile::Tier;

/// @doc: Verifies `Element::ALL` contains exactly 8 entries, one per element.
#[test]
fn when_element_all_contains_8_entries() {
    assert_eq!(Element::ALL.len(), 8, "expected 8 elements");
}

/// @doc: Verifies `Element::ALL` contains all 8 unique element variants with no duplicates.
#[test]
fn when_element_all_contains_unique_variants() {
    let mut mask = 0u32;
    for e in Element::ALL {
        match e {
            Element::Solidum => mask |= 1 << 0,
            Element::Febris => mask |= 1 << 1,
            Element::Ordinem => mask |= 1 << 2,
            Element::Lumines => mask |= 1 << 3,
            Element::Varias => mask |= 1 << 4,
            Element::Inertiae => mask |= 1 << 5,
            Element::Subsidium => mask |= 1 << 6,
            Element::Spatium => mask |= 1 << 7,
        }
    }
    assert_eq!(mask, 0xFF, "not all element variants present in ALL");
}

/// @doc: Verifies all 5 Rarity variants compile and can be constructed.
#[test]
fn when_rarity_enum_has_5_variants() {
    let variants = [
        Rarity::Common,
        Rarity::Uncommon,
        Rarity::Rare,
        Rarity::Epic,
        Rarity::Legendary,
    ];
    assert_eq!(variants.len(), 5, "expected 5 rarity variants");
}

/// @doc: Verifies all 16 Aspect variants exist and pair correctly with their parent elements.
#[test]
fn when_aspect_enum_has_16_variants_with_correct_element_pairings() {
    let sig_pos = CardSignature::new([1.0; 8]);
    let sig_neg = CardSignature::new([-1.0; 8]);

    assert_eq!(sig_pos.dominant_aspect(Element::Solidum), Aspect::Solid);
    assert_eq!(sig_neg.dominant_aspect(Element::Solidum), Aspect::Fragile);
    assert_eq!(sig_pos.dominant_aspect(Element::Febris), Aspect::Heat);
    assert_eq!(sig_neg.dominant_aspect(Element::Febris), Aspect::Cold);
    assert_eq!(sig_pos.dominant_aspect(Element::Ordinem), Aspect::Order);
    assert_eq!(sig_neg.dominant_aspect(Element::Ordinem), Aspect::Chaos);
    assert_eq!(sig_pos.dominant_aspect(Element::Lumines), Aspect::Light);
    assert_eq!(sig_neg.dominant_aspect(Element::Lumines), Aspect::Dark);
    assert_eq!(sig_pos.dominant_aspect(Element::Varias), Aspect::Change);
    assert_eq!(sig_neg.dominant_aspect(Element::Varias), Aspect::Stasis);
    assert_eq!(sig_pos.dominant_aspect(Element::Inertiae), Aspect::Force);
    assert_eq!(sig_neg.dominant_aspect(Element::Inertiae), Aspect::Calm);
    assert_eq!(sig_pos.dominant_aspect(Element::Subsidium), Aspect::Growth);
    assert_eq!(sig_neg.dominant_aspect(Element::Subsidium), Aspect::Decay);
    assert_eq!(sig_pos.dominant_aspect(Element::Spatium), Aspect::Expansion);
    assert_eq!(
        sig_neg.dominant_aspect(Element::Spatium),
        Aspect::Contraction
    );
}

/// @doc: Verifies `RarityTierConfig` default values are 0.3 for both fields.
#[test]
fn when_rarity_tier_config_default_returns_expected_values() {
    let config = RarityTierConfig::default();
    assert!(
        (config.rarity_advance_rate - 0.3).abs() < f32::EPSILON,
        "expected 0.3, got {}",
        config.rarity_advance_rate
    );
    assert!(
        (config.tier_advance_rate - 0.3).abs() < f32::EPSILON,
        "expected 0.3, got {}",
        config.tier_advance_rate
    );
}

/// @doc: Verifies `RarityTierConfig` can be constructed with custom values.
#[test]
fn when_rarity_tier_config_custom_construction_then_values_are_stored() {
    let config = RarityTierConfig {
        rarity_advance_rate: 0.7,
        tier_advance_rate: 0.1,
    };
    assert!(
        (config.rarity_advance_rate - 0.7).abs() < f32::EPSILON,
        "expected 0.7, got {}",
        config.rarity_advance_rate
    );
    assert!(
        (config.tier_advance_rate - 0.1).abs() < f32::EPSILON,
        "expected 0.1, got {}",
        config.tier_advance_rate
    );
}

/// @doc: Verifies `CardSignature` default produces all-zero axes.
#[test]
fn when_signature_default_then_all_axes_are_zero() {
    let sig: CardSignature = CardSignature::default();
    assert_eq!(
        sig.axes(),
        [0.0; 8],
        "default signature should have all zero axes"
    );
}

/// @doc: Verifies `CardSignature` can be cloned with identical axis values.
#[test]
fn when_signature_cloned_then_values_match() {
    let input = [0.1, -0.2, 0.3, -0.4, 0.5, -0.6, 0.7, -0.8];
    let sig = CardSignature::new(input);
    let cloned = sig;
    assert_eq!(
        sig.axes(),
        cloned.axes(),
        "cloned signature axes should match original"
    );
}

/// @doc: Verifies `CardSignature` implements Copy — original remains usable after copy.
#[test]
fn when_signature_copied_then_original_remains_usable() {
    let sig = CardSignature::new([0.5; 8]);
    let _copied = sig; // moves via Copy
    let axes = sig.axes(); // original still accessible
    assert_eq!(axes, [0.5; 8], "original should remain usable after copy");
}

/// @doc: Verifies `CardSignature` Eq is reflexive — each value equals itself.
#[test]
fn when_signature_eq_self_then_true() {
    let sig = CardSignature::new([0.1, -0.2, 0.3, -0.4, 0.5, -0.6, 0.7, -0.8]);
    assert_eq!(sig, sig, "signature should equal itself");
}

/// @doc: Verifies `CardSignature` Ord ordering is consistent with Eq for equal values.
#[test]
fn when_signature_ord_equal_then_eq_is_true() {
    let a = CardSignature::new([0.1, -0.2, 0.3, -0.4, 0.5, -0.6, 0.7, -0.8]);
    let b = CardSignature::new([0.1, -0.2, 0.3, -0.4, 0.5, -0.6, 0.7, -0.8]);
    assert_eq!(
        a.cmp(&b),
        std::cmp::Ordering::Equal,
        "Ord should return Equal"
    );
    assert_eq!(a, b, "Eq should hold for Ord-equal signatures");
}

/// @doc: Verifies `CardSignature` `axes()` returns an owned array, independent from the original.
#[test]
fn when_signature_axes_returns_owned_array() {
    let sig = CardSignature::new([0.1, -0.2, 0.3, -0.4, 0.5, -0.6, 0.7, -0.8]);
    let mut axes = sig.axes();
    // Mutating the returned array must not affect the original
    axes[0] = 99.0;
    assert_eq!(axes[0], 99.0, "mutation on local copy should persist");
    assert_eq!(
        sig.axes()[0],
        0.1,
        "mutating local axes array should not affect signature"
    );
}

/// @doc: Verifies `card_tier_with_config` uses the provided config rate, not the default.
#[test]
fn when_card_tier_with_config_uses_provided_rate() {
    let sig = CardSignature::new([1.0; 8]);

    // With tier_advance_rate = 0.0, geometric_level always returns level 0 → Dormant
    let low_config = RarityTierConfig {
        tier_advance_rate: 0.0,
        ..RarityTierConfig::default()
    };
    assert_eq!(
        sig.card_tier_with_config(&low_config),
        Tier::Dormant,
        "expected Dormant with rate 0.0"
    );

    // With tier_advance_rate = 1.0, geometric_level always returns max level → Intense
    let high_config = RarityTierConfig {
        tier_advance_rate: 1.0,
        ..RarityTierConfig::default()
    };
    assert_eq!(
        sig.card_tier_with_config(&high_config),
        Tier::Intense,
        "expected Intense with rate 1.0"
    );
}
