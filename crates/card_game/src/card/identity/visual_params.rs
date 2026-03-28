use crate::card::identity::signature::{CardSignature, Element};
use crate::card::identity::signature_profile::{SignatureProfile, Tier};
use crate::card::rendering::art_shader::ShaderVariant;
use engine_core::color::Color;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

pub fn compute_seed(signature: &CardSignature) -> u64 {
    signature
        .axes()
        .iter()
        .enumerate()
        .fold(0u64, |acc, (i, &v)| {
            let bits = u64::from(v.to_bits());
            let mixed = bits
                .wrapping_add(0x9e37_79b9_7f4a_7c15)
                .wrapping_mul(i as u64 + 1);
            acc ^ mixed.rotate_left(17).wrapping_mul(0x94d0_49bb_1331_11eb)
        })
}

#[derive(Debug, PartialEq)]
pub struct CardVisualParams {
    pub art_color: Color,
    pub element_tint: Color,
    pub tier_detail: Tier,
    pub pattern_index: u8,
    pub shader_variant: ShaderVariant,
}

pub const PATTERN_COUNT: u8 = 4;

const COLOR_NOISE: f32 = 0.05;

pub fn element_base_color(element: Element) -> Color {
    match element {
        Element::Solidum => Color::new(0.55, 0.38, 0.18, 1.0), // brown/amber — earth, stone
        Element::Febris => Color::new(0.85, 0.25, 0.10, 1.0),  // red-orange — heat, cold
        Element::Ordinem => Color::new(0.20, 0.40, 0.80, 1.0), // blue — order, chaos
        Element::Lumines => Color::new(0.90, 0.78, 0.15, 1.0), // gold/yellow — light, dark
        Element::Varias => Color::new(0.22, 0.70, 0.28, 1.0),  // green — change, stasis
        Element::Inertiae => Color::new(0.50, 0.58, 0.68, 1.0), // steel gray-blue — force, stillness
        Element::Subsidium => Color::new(0.10, 0.65, 0.52, 1.0), // emerald/teal — growth, decay
        Element::Spatium => Color::new(0.58, 0.20, 0.80, 1.0), // violet/purple — expansion, contraction
    }
}

pub fn generate_card_visuals(
    signature: &CardSignature,
    profile: &SignatureProfile,
) -> CardVisualParams {
    let seed = compute_seed(signature);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    let element_tint = profile
        .dominant_axis
        .map_or(Color::new(0.5, 0.5, 0.5, 1.0), element_base_color);

    let art_color = Color::new(
        (element_tint.r + rng.gen_range(-COLOR_NOISE..=COLOR_NOISE)).clamp(0.0, 1.0),
        (element_tint.g + rng.gen_range(-COLOR_NOISE..=COLOR_NOISE)).clamp(0.0, 1.0),
        (element_tint.b + rng.gen_range(-COLOR_NOISE..=COLOR_NOISE)).clamp(0.0, 1.0),
        1.0,
    );
    let pattern_index = rng.gen_range(0..PATTERN_COUNT);
    let shader_variant = ShaderVariant::from_rarity(signature.rarity());
    let tier_detail = profile
        .dominant_axis
        .map_or(Tier::Dormant, |e| profile.tiers[e as usize]);

    CardVisualParams {
        art_color,
        element_tint,
        tier_detail,
        pattern_index,
        shader_variant,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::card::identity::signature_profile::{SignatureProfile, Tier};
    use crate::card::rendering::art_shader::ShaderVariant;

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

    /// @doc: Verifies all element base colors have alpha=1.0 (fully opaque).
    /// Transparent element tints would render incorrectly on card artwork.
    #[test]
    fn when_element_hue_mapping_called_then_all_colors_are_fully_opaque() {
        // Arrange / Act
        let colors: [Color; 8] = Element::ALL.map(element_base_color);

        // Assert
        for (i, c) in colors.iter().enumerate() {
            assert_eq!(
                c.a,
                1.0,
                "element {:?} base color alpha is {} (expected 1.0)",
                Element::ALL[i],
                c.a
            );
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

    /// @doc: Validates that art color deviates from element tint by at most COLOR_NOISE (0.05 per channel).
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
        assert!(
            (params.art_color.r - tint.r).abs() <= COLOR_NOISE,
            "red channel {} too far from tint {}",
            params.art_color.r,
            tint.r
        );
        assert!(
            (params.art_color.g - tint.g).abs() <= COLOR_NOISE,
            "green channel {} too far from tint {}",
            params.art_color.g,
            tint.g
        );
        assert!(
            (params.art_color.b - tint.b).abs() <= COLOR_NOISE,
            "blue channel {} too far from tint {}",
            params.art_color.b,
            tint.b
        );
    }

    /// @doc: Maps signature intensity to tier detail (Dormant < 0.3). Card artwork complexity scales with tier.
    /// This ensures weak signatures get minimal visual detail, preventing overstated power representation.
    #[test]
    fn when_dominant_axis_is_dormant_then_tier_detail_is_dormant() {
        // Arrange — Solidum at 0.1 (below 0.3 threshold = Dormant)
        let sig = CardSignature::new([0.1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let profile = SignatureProfile::without_archetype(&sig);

        // Act
        let params = generate_card_visuals(&sig, &profile);

        // Assert
        assert_eq!(params.tier_detail, Tier::Dormant);
    }

    /// @doc: Maps Active tier for signatures in [0.3, 0.7) range.
    /// Without this test, tier threshold logic could drift and misrepresent card intensity.
    #[test]
    fn when_dominant_axis_is_active_then_tier_detail_is_active() {
        // Arrange — Solidum at 0.5 (0.3..0.7 = Active)
        let sig = CardSignature::new([0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let profile = SignatureProfile::without_archetype(&sig);

        // Act
        let params = generate_card_visuals(&sig, &profile);

        // Assert
        assert_eq!(params.tier_detail, Tier::Active);
    }

    /// @doc: Maps Intense tier for signatures >= 0.7 (high power/rarity cards).
    /// Tier detail drives art complexity shaders, so incorrect mapping breaks visual hierarchy.
    #[test]
    fn when_dominant_axis_is_intense_then_tier_detail_is_intense() {
        // Arrange — Solidum at 0.8 (>= 0.7 = Intense)
        let sig = CardSignature::new([0.8, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let profile = SignatureProfile::without_archetype(&sig);

        // Act
        let params = generate_card_visuals(&sig, &profile);

        // Assert
        assert_eq!(params.tier_detail, Tier::Intense);
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
    fn when_two_profiles_with_same_dominant_but_different_signatures_then_art_colors_within_noise()
    {
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
        for (label, params) in [("sig_a", &params_a), ("sig_b", &params_b)] {
            assert!(
                (params.art_color.r - tint.r).abs() <= COLOR_NOISE,
                "{label} red channel {} too far from tint {}",
                params.art_color.r,
                tint.r
            );
            assert!(
                (params.art_color.g - tint.g).abs() <= COLOR_NOISE,
                "{label} green channel {} too far from tint {}",
                params.art_color.g,
                tint.g
            );
            assert!(
                (params.art_color.b - tint.b).abs() <= COLOR_NOISE,
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

    #[test]
    fn when_compute_seed_called_on_all_zero_signature_then_does_not_panic() {
        // Arrange
        let sig = CardSignature::default();

        // Act / Assert — must complete without panicking
        compute_seed(&sig);
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

    /// @doc: All generated art colors must be fully opaque (alpha=1.0) regardless of signature.
    /// Transparent colors would blend with background incorrectly, corrupting visual identity.
    #[test]
    fn when_generate_card_visuals_called_then_art_color_is_fully_opaque() {
        // Arrange
        let sig = CardSignature::new([0.5, -0.3, 0.8, 0.0, -0.1, 0.6, -0.9, 0.2]);

        // Act
        let params = generate_card_visuals(&sig, &SignatureProfile::without_archetype(&sig));

        // Assert
        assert_eq!(params.art_color.a, 1.0);
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

    /// @doc: Pattern index selection must always be within [0, PATTERN_COUNT) to avoid out-of-bounds art lookups.
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

    /// @doc: Common rarity (all axes near 0) maps to no shader effects (ShaderVariant::None).
    /// Rarity-driven visual feedback requires correct shader selection, otherwise all cards look identical.
    #[test]
    fn when_generate_card_visuals_called_with_common_signature_then_shader_variant_is_none() {
        // Arrange
        let sig = CardSignature::new([0.0; 8]);

        // Act
        let params = generate_card_visuals(&sig, &SignatureProfile::without_archetype(&sig));

        // Assert
        assert_eq!(params.shader_variant, ShaderVariant::None);
    }

    /// @doc: Legendary rarity (all axes at max) maps to ShaderVariant::Foil for premium visual effect.
    /// Without this test, high-rarity cards would not visually distinguish themselves in gameplay.
    #[test]
    fn when_generate_card_visuals_called_with_legendary_signature_then_shader_variant_is_foil() {
        // Arrange
        let sig = CardSignature::new([1.0; 8]);

        // Act
        let params = generate_card_visuals(&sig, &SignatureProfile::without_archetype(&sig));

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
}
