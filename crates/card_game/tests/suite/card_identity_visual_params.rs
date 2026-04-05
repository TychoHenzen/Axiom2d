#![allow(clippy::unwrap_used)]

use card_game::card::identity::signature::{CardSignature, Element, Rarity, compute_seed};
use card_game::card::identity::signature_profile::SignatureProfile;
use card_game::card::identity::visual_params::{
    PATTERN_COUNT, element_base_color, generate_card_visuals,
};
use card_game::card::rendering::art_shader::ShaderVariant;
use engine_core::color::Color;

/// @doc: Ensures all 8 elements map to visually distinct base colors.
/// Without this test, two elements could accidentally share a color, breaking visual uniqueness on the card.
#[test]
fn when_element_hue_mapping_called_for_each_element_then_all_eight_hues_are_distinct() {
    // Arrange
    let elements = Element::ALL;

    // Act
    let colors: [Color; 8] = elements.map(element_base_color);

    // Assert — all 28 pairs must differ
    for i in 0..colors.len() {
        for j in (i + 1)..colors.len() {
            assert_ne!(
                colors[i], colors[j],
                "{:?} and {:?} produced the same base color {:?}",
                elements[i], elements[j], colors[i]
            );
        }
    }
}

/// @doc: Confirms dominant element drives the tint color, not signature axes directly.
/// This ensures rarity-independent visual consistency: same element always produces same tint.
#[test]
fn when_profile_has_dominant_febris_then_element_tint_matches_febris_base_color() {
    // Arrange
    let sig = CardSignature::new([0.0, 0.9, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    let profile = SignatureProfile::without_archetype(&sig);

    // Act
    let params = generate_card_visuals(&sig, &profile);

    // Assert
    assert_eq!(
        params.element_tint,
        element_base_color(Element::Febris),
        "dominant Febris profile should tint with Febris base color"
    );
}

#[test]
fn when_profile_has_dominant_spatium_then_element_tint_matches_spatium_base_color() {
    // Arrange
    let sig = CardSignature::new([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.9]);
    let profile = SignatureProfile::without_archetype(&sig);

    // Act
    let params = generate_card_visuals(&sig, &profile);

    // Assert
    assert_eq!(
        params.element_tint,
        element_base_color(Element::Spatium),
        "dominant Spatium profile should tint with Spatium base color"
    );
}

#[test]
fn when_profile_has_dominant_lumines_then_element_tint_matches_lumines_base_color() {
    // Arrange
    let sig = CardSignature::new([0.0, 0.0, 0.0, 0.9, 0.0, 0.0, 0.0, 0.0]);
    let profile = SignatureProfile::without_archetype(&sig);

    // Act
    let params = generate_card_visuals(&sig, &profile);

    // Assert
    assert_eq!(
        params.element_tint,
        element_base_color(Element::Lumines),
        "dominant Lumines profile should tint with Lumines base color"
    );
}

/// @doc: Validates that art color deviates from element tint by at most `COLOR_NOISE` (0.05 per channel).
/// This noise prevents cards of the same element from looking identical while respecting visual coherence.
#[test]
fn when_generate_card_visuals_called_then_art_color_is_close_to_element_tint_within_noise() {
    // Arrange
    let sig = CardSignature::new([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.8]);
    let profile = SignatureProfile::without_archetype(&sig);

    // Act
    let params = generate_card_visuals(&sig, &profile);

    // Assert — each channel within COLOR_NOISE of the element tint
    let tint = params.element_tint;
    let color_noise = 0.05;
    assert!(
        (params.art_color.r - tint.r).abs() <= color_noise,
        "red channel {} too far from tint {}",
        params.art_color.r,
        tint.r
    );
    assert!(
        (params.art_color.g - tint.g).abs() <= color_noise,
        "green channel {} too far from tint {}",
        params.art_color.g,
        tint.g
    );
    assert!(
        (params.art_color.b - tint.b).abs() <= color_noise,
        "blue channel {} too far from tint {}",
        params.art_color.b,
        tint.b
    );
}

/// @doc: Tier detail in visuals comes from the card-level hash-based tier,
/// not axis magnitude. This decouples visual power representation from
/// the signature's type identity.
#[test]
fn when_generate_card_visuals_called_then_tier_detail_matches_profile_card_tier() {
    // Arrange
    let sig = CardSignature::new([0.5, -0.3, 0.8, -0.1, 0.6, -0.4, 0.2, -0.9]);
    let profile = SignatureProfile::without_archetype(&sig);

    // Act
    let params = generate_card_visuals(&sig, &profile);

    // Assert
    assert_eq!(params.tier_detail, profile.tier);
}

/// @doc: Element identity must be visually distinct in the tint color, not just metadata.
/// Different element profiles must produce perceptually different colors to players.
#[test]
fn when_two_profiles_with_different_dominant_elements_then_element_tints_differ() {
    // Arrange
    let sig_a = CardSignature::new([0.9, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    let sig_b = CardSignature::new([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.9]);
    let profile_a = SignatureProfile::without_archetype(&sig_a);
    let profile_b = SignatureProfile::without_archetype(&sig_b);

    // Act
    let params_a = generate_card_visuals(&sig_a, &profile_a);
    let params_b = generate_card_visuals(&sig_b, &profile_b);

    // Assert
    assert_ne!(
        params_a.element_tint, params_b.element_tint,
        "Solidum-dominant and Spatium-dominant should have different tints"
    );
}

/// @doc: Element tint depends only on dominant element, not intensity.
/// This ensures consistent color branding across Common/Rare/Legendary variants of same element.
#[test]
fn when_two_profiles_with_same_dominant_element_but_different_intensities_then_tints_equal() {
    // Arrange — both Solidum-dominant, different intensities
    let sig_a = CardSignature::new([0.4, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    let sig_b = CardSignature::new([0.9, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
    let profile_a = SignatureProfile::without_archetype(&sig_a);
    let profile_b = SignatureProfile::without_archetype(&sig_b);

    // Act
    let params_a = generate_card_visuals(&sig_a, &profile_a);
    let params_b = generate_card_visuals(&sig_b, &profile_b);

    // Assert
    assert_eq!(
        params_a.element_tint, params_b.element_tint,
        "same dominant element should produce same tint regardless of intensity"
    );
}

#[test]
fn when_two_profiles_with_same_dominant_but_different_signatures_then_art_colors_within_noise() {
    // Arrange — both Solidum-dominant but substantially different signatures
    let sig_a = CardSignature::new([0.9, 0.1, 0.2, 0.0, 0.0, 0.0, 0.0, 0.0]);
    let sig_b = CardSignature::new([0.9, 0.0, 0.0, 0.3, 0.1, 0.2, 0.0, 0.0]);
    let profile_a = SignatureProfile::without_archetype(&sig_a);
    let profile_b = SignatureProfile::without_archetype(&sig_b);

    // Act
    let params_a = generate_card_visuals(&sig_a, &profile_a);
    let params_b = generate_card_visuals(&sig_b, &profile_b);

    // Assert — both art_colors are close to the shared Solidum tint
    let tint = element_base_color(Element::Solidum);
    let color_noise = 0.05;
    for (label, params) in [("sig_a", &params_a), ("sig_b", &params_b)] {
        assert!(
            (params.art_color.r - tint.r).abs() <= color_noise,
            "{label} red channel {} too far from tint {}",
            params.art_color.r,
            tint.r
        );
        assert!(
            (params.art_color.g - tint.g).abs() <= color_noise,
            "{label} green channel {} too far from tint {}",
            params.art_color.g,
            tint.g
        );
        assert!(
            (params.art_color.b - tint.b).abs() <= color_noise,
            "{label} blue channel {} too far from tint {}",
            params.art_color.b,
            tint.b
        );
    }
}

/// @doc: Signature-to-RNG seed mapping must be deterministic so cards are reproducible.
/// Save/load or multiplayer sync would break if same signature produced different visuals.
#[test]
fn when_compute_seed_called_on_identical_signatures_then_results_are_equal() {
    // Arrange
    let axes = [0.1, -0.3, 0.5, -0.7, 0.9, -0.2, 0.4, -0.6];
    let sig_a = CardSignature::new(axes);
    let sig_b = CardSignature::new(axes);

    // Act
    let seed_a = compute_seed(&sig_a);
    let seed_b = compute_seed(&sig_b);

    // Assert
    assert_eq!(seed_a, seed_b);
}

/// @doc: Different signatures must hash to different seeds (distribution property).
/// Poor hashing could cause two distinct cards to generate identical art, defeating signature uniqueness.
#[test]
fn when_compute_seed_called_on_different_signatures_then_results_differ() {
    // Arrange
    let sig_zeros = CardSignature::new([0.0; 8]);
    let sig_ones = CardSignature::new([1.0; 8]);

    // Act
    let seed_a = compute_seed(&sig_zeros);
    let seed_b = compute_seed(&sig_ones);

    // Assert
    assert_ne!(seed_a, seed_b);
}

/// @doc: Sign-opposite signatures (e.g., +0.5 vs -0.5 Heat vs Cold) must hash differently.
/// Without this test, positive/negative axes could accidentally collide, breaking aspect uniqueness.
#[test]
fn when_compute_seed_called_on_sign_opposite_signatures_then_seeds_differ() {
    // Arrange
    let sig_pos = CardSignature::new([0.5; 8]);
    let sig_neg = CardSignature::new([-0.5; 8]);

    // Act
    let seed_pos = compute_seed(&sig_pos);
    let seed_neg = compute_seed(&sig_neg);

    // Assert
    assert_ne!(seed_pos, seed_neg);
}

/// @doc: Visual generation is deterministic: same signature always produces identical visuals.
/// Non-determinism would make card identity unstable (critical for save files and UI consistency).
#[test]
fn when_generate_card_visuals_called_twice_with_same_signature_then_results_are_identical() {
    // Arrange
    let sig = CardSignature::new([0.3, -0.7, 0.1, 0.9, -0.5, 0.2, -0.8, 0.6]);

    // Act
    let params_a = generate_card_visuals(&sig, &SignatureProfile::without_archetype(&sig));
    let params_b = generate_card_visuals(&sig, &SignatureProfile::without_archetype(&sig));

    // Assert
    assert_eq!(params_a, params_b);
}

/// @doc: Two cards with different signatures must not render with the same color.
/// Collision in art colors would create visual confusion, defeating signature differentiation.
#[test]
fn when_generate_card_visuals_called_on_different_signatures_then_art_colors_differ() {
    // Arrange
    let sig_a = CardSignature::new([0.0; 8]);
    let sig_b = CardSignature::new([1.0; 8]);

    // Act
    let params_a = generate_card_visuals(&sig_a, &SignatureProfile::without_archetype(&sig_a));
    let params_b = generate_card_visuals(&sig_b, &SignatureProfile::without_archetype(&sig_b));

    // Assert
    assert_ne!(params_a.art_color, params_b.art_color);
}

/// @doc: Pattern index selection must always be within [0, `PATTERN_COUNT`) to avoid out-of-bounds art lookups.
/// Overflow here would panic or access wrong art textures at runtime.
#[test]
fn when_generate_card_visuals_called_then_pattern_index_is_within_bounds() {
    // Arrange
    let sig = CardSignature::new([0.5, -0.3, 0.8, 0.0, -0.1, 0.6, -0.9, 0.2]);

    // Act
    let params = generate_card_visuals(&sig, &SignatureProfile::without_archetype(&sig));

    // Assert
    assert!(
        params.pattern_index < PATTERN_COUNT,
        "pattern_index {} must be < PATTERN_COUNT {}",
        params.pattern_index,
        PATTERN_COUNT
    );
}

/// @doc: Common rarity maps to no shader effects (`ShaderVariant::None`).
/// Rarity-driven visual feedback requires correct shader selection, otherwise all cards look identical.
#[test]
fn when_generate_card_visuals_called_with_common_rarity_then_shader_variant_is_none() {
    // Arrange
    let sig = CardSignature::new([0.0; 8]);
    let mut profile = SignatureProfile::without_archetype(&sig);
    profile.rarity = Rarity::Common;

    // Act
    let params = generate_card_visuals(&sig, &profile);

    // Assert
    assert_eq!(params.shader_variant, ShaderVariant::None);
}

/// @doc: Legendary rarity maps to `ShaderVariant::Foil` for premium visual effect.
/// Without this test, high-rarity cards would not visually distinguish themselves in gameplay.
#[test]
fn when_generate_card_visuals_called_with_legendary_rarity_then_shader_variant_is_foil() {
    // Arrange
    let sig = CardSignature::new([1.0; 8]);
    let mut profile = SignatureProfile::without_archetype(&sig);
    profile.rarity = Rarity::Legendary;

    // Act
    let params = generate_card_visuals(&sig, &profile);

    // Assert
    assert_eq!(params.shader_variant, ShaderVariant::Foil);
}

#[test]
fn when_signature_serialized_to_json_and_deserialized_then_visuals_are_identical() {
    // Arrange — values that survive f32 JSON roundtrip cleanly
    let original = CardSignature::new([0.5, -0.5, 0.25, -0.25, 0.75, -0.75, 1.0, -1.0]);
    let json = serde_json::to_string(&original).unwrap();
    let restored: CardSignature = serde_json::from_str(&json).unwrap();

    // Act
    let params_original =
        generate_card_visuals(&original, &SignatureProfile::without_archetype(&original));
    let params_restored =
        generate_card_visuals(&restored, &SignatureProfile::without_archetype(&restored));

    // Assert
    assert_eq!(params_original, params_restored);
}
