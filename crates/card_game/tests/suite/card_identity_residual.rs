#![allow(clippy::unwrap_used)]

use card_game::card::identity::base_type::{BaseCardType, CardCategory};
use card_game::card::identity::residual::{ModifierType, ResidualModifier, ResidualStats};
use card_game::card::identity::signature::{CardSignature, Element};

// ===== Batch 1: calculate_effect =====

#[test]
fn when_positive_axis_and_use_positive_true_then_returns_axis_times_intensity() {
    // Arrange
    let modifier = ResidualModifier {
        source_element: Element::Febris,
        modifier_type: ModifierType::Power,
        intensity: 2.0,
        use_positive: true,
    };
    let residual = CardSignature::new([0.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);

    // Act
    let result = modifier.calculate_effect(&residual);

    // Assert
    assert!(
        (result - 1.0).abs() < 1e-5,
        "expected 1.0 (0.5 * 2.0), got {result}"
    );
}

#[test]
fn when_source_element_is_zero_but_other_axes_nonzero_then_returns_zero() {
    // Arrange
    let modifier = ResidualModifier {
        source_element: Element::Febris,
        modifier_type: ModifierType::Power,
        intensity: 2.0,
        use_positive: true,
    };
    let residual = CardSignature::new([0.9, 0.0, 0.8, 0.7, 0.6, 0.5, 0.4, 0.3]);

    // Act
    let result = modifier.calculate_effect(&residual);

    // Assert
    assert!(
        result.abs() < 1e-5,
        "expected 0.0 (Febris axis is zero), got {result}"
    );
}

/// @doc: Polarity gating — positive-gated modifiers ignore negative residuals, creating asymmetric stat scaling
#[test]
fn when_use_positive_true_and_axis_negative_then_returns_zero() {
    // Arrange
    let modifier = ResidualModifier {
        source_element: Element::Solidum,
        modifier_type: ModifierType::Defense,
        intensity: 3.0,
        use_positive: true,
    };
    let residual = CardSignature::new([-0.8, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);

    // Act
    let result = modifier.calculate_effect(&residual);

    // Assert
    assert!(
        result.abs() < 1e-5,
        "expected 0.0 (positive gate blocks negative), got {result}"
    );
}

#[test]
fn when_use_positive_false_and_axis_positive_then_returns_zero() {
    // Arrange
    let modifier = ResidualModifier {
        source_element: Element::Solidum,
        modifier_type: ModifierType::Defense,
        intensity: 3.0,
        use_positive: false,
    };
    let residual = CardSignature::new([0.8, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);

    // Act
    let result = modifier.calculate_effect(&residual);

    // Assert
    assert!(
        result.abs() < 1e-5,
        "expected 0.0 (negative gate blocks positive), got {result}"
    );
}

#[test]
fn when_use_positive_false_and_axis_negative_then_returns_signed_value_times_intensity() {
    // Arrange
    let modifier = ResidualModifier {
        source_element: Element::Lumines,
        modifier_type: ModifierType::Healing,
        intensity: 1.5,
        use_positive: false,
    };
    let residual = CardSignature::new([0.0, 0.0, 0.0, -0.4, 0.0, 0.0, 0.0, 0.0]);

    // Act
    let result = modifier.calculate_effect(&residual);

    // Assert
    assert!(
        (result - (-0.6)).abs() < 1e-5,
        "expected -0.6 (-0.4 * 1.5), got {result}"
    );
}

#[test]
fn when_all_axes_zero_then_returns_zero_regardless_of_polarity() {
    // Arrange
    let pos_mod = ResidualModifier {
        source_element: Element::Varias,
        modifier_type: ModifierType::Speed,
        intensity: 5.0,
        use_positive: true,
    };
    let neg_mod = ResidualModifier {
        source_element: Element::Varias,
        modifier_type: ModifierType::Speed,
        intensity: 5.0,
        use_positive: false,
    };
    let residual = CardSignature::new([0.0; 8]);

    // Act
    let pos_result = pos_mod.calculate_effect(&residual);
    let neg_result = neg_mod.calculate_effect(&residual);

    // Assert
    assert!(
        pos_result.abs() < 1e-5,
        "use_positive=true: expected 0.0, got {pos_result}"
    );
    assert!(
        neg_result.abs() < 1e-5,
        "use_positive=false: expected 0.0, got {neg_result}"
    );
}

#[test]
fn when_axis_at_max_positive_then_returns_intensity() {
    // Arrange
    let modifier = ResidualModifier {
        source_element: Element::Inertiae,
        modifier_type: ModifierType::Power,
        intensity: 0.75,
        use_positive: true,
    };
    let residual = CardSignature::new([0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0]);

    // Act
    let result = modifier.calculate_effect(&residual);

    // Assert
    assert!(
        (result - 0.75).abs() < 1e-5,
        "expected 0.75 (1.0 * 0.75), got {result}"
    );
}

// ===== Batch 2: ResidualStats::compute =====

#[test]
fn when_no_modifiers_then_all_stats_are_zero() {
    // Arrange
    let residual = CardSignature::new([0.5, -0.3, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0]);

    // Act
    let stats = ResidualStats::compute(&residual, &[]);

    // Assert
    let zero = ResidualStats {
        power: 0.0,
        cost: 0.0,
        duration: 0.0,
        range: 0.0,
        healing: 0.0,
        speed: 0.0,
        defense: 0.0,
        special: 0.0,
    };
    assert_eq!(stats, zero);
}

#[test]
fn when_single_modifier_then_only_that_stat_is_nonzero() {
    // Arrange
    let modifier = ResidualModifier {
        source_element: Element::Febris,
        modifier_type: ModifierType::Power,
        intensity: 1.0,
        use_positive: true,
    };
    let residual = CardSignature::new([0.0, 0.6, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);

    // Act
    let stats = ResidualStats::compute(&residual, &[modifier]);

    // Assert
    assert!(
        (stats.power - 0.6).abs() < 1e-5,
        "expected power=0.6, got {}",
        stats.power
    );
    assert!(stats.cost.abs() < 1e-5, "cost should be 0");
    assert!(stats.duration.abs() < 1e-5, "duration should be 0");
    assert!(stats.range.abs() < 1e-5, "range should be 0");
    assert!(stats.healing.abs() < 1e-5, "healing should be 0");
    assert!(stats.speed.abs() < 1e-5, "speed should be 0");
    assert!(stats.defense.abs() < 1e-5, "defense should be 0");
    assert!(stats.special.abs() < 1e-5, "special should be 0");
}

#[test]
fn when_two_modifiers_target_same_stat_then_effects_are_summed() {
    // Arrange
    let modifiers = vec![
        ResidualModifier {
            source_element: Element::Febris,
            modifier_type: ModifierType::Power,
            intensity: 1.0,
            use_positive: true,
        },
        ResidualModifier {
            source_element: Element::Solidum,
            modifier_type: ModifierType::Power,
            intensity: 0.5,
            use_positive: true,
        },
    ];
    let residual = CardSignature::new([0.6, 0.4, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);

    // Act
    let stats = ResidualStats::compute(&residual, &modifiers);

    // Assert — 0.4*1.0 + 0.6*0.5 = 0.7
    assert!(
        (stats.power - 0.7).abs() < 1e-5,
        "expected power=0.7, got {}",
        stats.power
    );
}

#[test]
fn when_modifiers_target_different_stats_then_each_accumulates_independently() {
    // Arrange
    let modifiers = vec![
        ResidualModifier {
            source_element: Element::Febris,
            modifier_type: ModifierType::Power,
            intensity: 1.0,
            use_positive: true,
        },
        ResidualModifier {
            source_element: Element::Ordinem,
            modifier_type: ModifierType::Cost,
            intensity: 2.0,
            use_positive: true,
        },
    ];
    let residual = CardSignature::new([0.0, 0.5, 0.3, 0.0, 0.0, 0.0, 0.0, 0.0]);

    // Act
    let stats = ResidualStats::compute(&residual, &modifiers);

    // Assert
    assert!(
        (stats.power - 0.5).abs() < 1e-5,
        "expected power=0.5, got {}",
        stats.power
    );
    assert!(
        (stats.cost - 0.6).abs() < 1e-5,
        "expected cost=0.6, got {}",
        stats.cost
    );
}

#[test]
fn when_modifier_polarity_blocks_then_stat_contributes_zero() {
    // Arrange
    let modifier = ResidualModifier {
        source_element: Element::Febris,
        modifier_type: ModifierType::Power,
        intensity: 2.0,
        use_positive: true,
    };
    let residual = CardSignature::new([0.0, -0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);

    // Act
    let stats = ResidualStats::compute(&residual, &[modifier]);

    // Assert
    assert!(
        stats.power.abs() < 1e-5,
        "expected power=0.0 (gate blocks negative), got {}",
        stats.power
    );
}

#[test]
fn when_edge_of_radius_card_then_stat_equals_intensity() {
    // Arrange
    let modifier = ResidualModifier {
        source_element: Element::Varias,
        modifier_type: ModifierType::Power,
        intensity: 0.9,
        use_positive: true,
    };
    let residual = CardSignature::new([0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0]);

    // Act
    let stats = ResidualStats::compute(&residual, &[modifier]);

    // Assert
    assert!(
        (stats.power - 0.9).abs() < 1e-5,
        "expected power=0.9 (1.0 * 0.9), got {}",
        stats.power
    );
}

#[test]
fn when_all_eight_modifier_types_present_then_all_eight_stats_populated() {
    // Arrange — one modifier per stat, each reading a distinct element
    let elements = [
        Element::Solidum,
        Element::Febris,
        Element::Ordinem,
        Element::Lumines,
        Element::Varias,
        Element::Inertiae,
        Element::Subsidium,
        Element::Spatium,
    ];
    let types = [
        ModifierType::Power,
        ModifierType::Cost,
        ModifierType::Duration,
        ModifierType::Range,
        ModifierType::Healing,
        ModifierType::Speed,
        ModifierType::Defense,
        ModifierType::Special,
    ];
    let modifiers: Vec<ResidualModifier> = elements
        .iter()
        .zip(types.iter())
        .map(|(&elem, &mtype)| ResidualModifier {
            source_element: elem,
            modifier_type: mtype,
            intensity: 1.0,
            use_positive: true,
        })
        .collect();
    let residual = CardSignature::new([0.5; 8]);

    // Act
    let stats = ResidualStats::compute(&residual, &modifiers);

    // Assert
    assert!((stats.power - 0.5).abs() < 1e-5, "power");
    assert!((stats.cost - 0.5).abs() < 1e-5, "cost");
    assert!((stats.duration - 0.5).abs() < 1e-5, "duration");
    assert!((stats.range - 0.5).abs() < 1e-5, "range");
    assert!((stats.healing - 0.5).abs() < 1e-5, "healing");
    assert!((stats.speed - 0.5).abs() < 1e-5, "speed");
    assert!((stats.defense - 0.5).abs() < 1e-5, "defense");
    assert!((stats.special - 0.5).abs() < 1e-5, "special");
}

#[test]
fn when_mixed_positive_and_negative_residuals_then_gated_modifiers_sum_correctly() {
    // Arrange — two modifiers both target Healing:
    // one fires on positive Subsidium, one fires on negative Spatium
    let modifiers = vec![
        ResidualModifier {
            source_element: Element::Subsidium,
            modifier_type: ModifierType::Healing,
            intensity: 1.0,
            use_positive: true,
        },
        ResidualModifier {
            source_element: Element::Spatium,
            modifier_type: ModifierType::Healing,
            intensity: 1.0,
            use_positive: false,
        },
    ];
    let residual = CardSignature::new([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.7, -0.3]);

    // Act
    let stats = ResidualStats::compute(&residual, &modifiers);

    // Assert — 0.7*1.0 + (-0.3)*1.0 = 0.4
    assert!(
        (stats.healing - 0.4).abs() < 1e-5,
        "expected healing=0.4, got {}",
        stats.healing
    );
}

// ===== Integration: from_card =====

/// @doc: Stats derive from the residual (card minus base) — the further a card drifts from its archetype, the stronger its stats
#[test]
fn when_from_card_called_then_computes_residual_and_applies_modifiers() {
    // Arrange
    let base_type = BaseCardType {
        name: "Weapon".to_string(),
        base_signature: CardSignature::new([0.3, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]),
        match_radius: 2.0,
        category: CardCategory::Equipment,
        modifiers: vec![ResidualModifier {
            source_element: Element::Solidum,
            modifier_type: ModifierType::Power,
            intensity: 1.0,
            use_positive: true,
        }],
    };
    // Card signature differs from base on Solidum: 0.8 - 0.3 = 0.5
    let card_sig = CardSignature::new([0.8, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);

    // Act
    let stats = ResidualStats::from_card(&card_sig, &base_type);

    // Assert
    assert!(
        (stats.power - 0.5).abs() < 1e-5,
        "expected power=0.5 (residual 0.5 * intensity 1.0), got {}",
        stats.power
    );
}

/// @doc: Zero residual means zero stats — a card exactly at its archetype center has no combat bonuses
#[test]
fn when_from_card_with_identical_signature_then_all_stats_zero() {
    // Arrange
    let sig = CardSignature::new([0.5, -0.3, 0.1, 0.0, 0.0, 0.0, 0.0, 0.0]);
    let base_type = BaseCardType {
        name: "Test".to_string(),
        base_signature: sig,
        match_radius: 1.0,
        category: CardCategory::Skill,
        modifiers: vec![ResidualModifier {
            source_element: Element::Solidum,
            modifier_type: ModifierType::Power,
            intensity: 5.0,
            use_positive: true,
        }],
    };

    // Act — identical signature means zero residual
    let stats = ResidualStats::from_card(&sig, &base_type);

    // Assert
    assert!(
        stats.power.abs() < 1e-5,
        "expected power=0.0 (zero residual), got {}",
        stats.power
    );
}
