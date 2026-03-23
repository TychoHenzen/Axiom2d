use crate::card::identity::signature::CardSignature;
use engine_core::color::Color;

pub fn compute_seed(signature: &CardSignature) -> u64 {
    signature
        .axes()
        .iter()
        .enumerate()
        .fold(0u64, |acc, (i, &v)| {
            let bits = u64::from(v.to_bits());
            // splitmix64-style mixing to spread each axis's contribution
            let mixed = bits
                .wrapping_add(0x9e37_79b9_7f4a_7c15)
                .wrapping_mul(i as u64 + 1);
            acc ^ mixed.rotate_left(17).wrapping_mul(0x94d0_49bb_1331_11eb)
        })
}

#[derive(Debug, PartialEq)]
pub struct CardVisualParams {
    pub art_color: Color,
    pub pattern_index: u8,
}

pub const PATTERN_COUNT: u8 = 4;

pub fn generate_card_visuals(signature: &CardSignature) -> CardVisualParams {
    use crate::card::identity::signature::Element;
    use rand::Rng;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    let seed = compute_seed(signature);
    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    let febris = signature[Element::Febris]; // [-1, 1]: -1=cold, +1=heat
    let lumines = signature[Element::Lumines]; // [-1, 1]: -1=dark, +1=bright

    // Lumines maps to overall brightness [0.3, 0.8]
    let brightness = 0.55 + lumines * 0.25;
    // Febris offsets red (warm) vs blue (cool) symmetrically around brightness
    let warmth = febris * 0.25;

    let base_r = brightness + warmth;
    let base_g = brightness;
    let base_b = brightness - warmth;

    // Small seeded noise (±0.05) preserves uniqueness across signatures
    // while keeping the semantic signal dominant (noise << warmth/brightness range)
    const NOISE: f32 = 0.05;
    let art_color = Color::new(
        (base_r + rng.gen_range(-NOISE..=NOISE)).clamp(0.0, 1.0),
        (base_g + rng.gen_range(-NOISE..=NOISE)).clamp(0.0, 1.0),
        (base_b + rng.gen_range(-NOISE..=NOISE)).clamp(0.0, 1.0),
        1.0,
    );
    let pattern_index = rng.gen_range(0..PATTERN_COUNT);

    CardVisualParams {
        art_color,
        pattern_index,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

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

        // Act
        let seed = compute_seed(&sig);

        // Assert — just needs to produce a value without panicking
        let _ = seed;
    }

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

    #[test]
    fn when_generate_card_visuals_called_then_art_color_is_fully_opaque() {
        // Arrange
        let sig = CardSignature::new([0.5, -0.3, 0.8, 0.0, -0.1, 0.6, -0.9, 0.2]);

        // Act
        let params = generate_card_visuals(&sig);

        // Assert
        assert_eq!(params.art_color.a, 1.0);
    }

    #[test]
    fn when_generate_card_visuals_called_twice_with_same_signature_then_results_are_identical() {
        // Arrange
        let sig = CardSignature::new([0.3, -0.7, 0.1, 0.9, -0.5, 0.2, -0.8, 0.6]);

        // Act
        let params_a = generate_card_visuals(&sig);
        let params_b = generate_card_visuals(&sig);

        // Assert
        assert_eq!(params_a, params_b);
    }

    #[test]
    fn when_generate_card_visuals_called_on_different_signatures_then_art_colors_differ() {
        // Arrange
        let sig_a = CardSignature::new([0.0; 8]);
        let sig_b = CardSignature::new([1.0; 8]);

        // Act
        let params_a = generate_card_visuals(&sig_a);
        let params_b = generate_card_visuals(&sig_b);

        // Assert
        assert_ne!(params_a.art_color, params_b.art_color);
    }

    #[test]
    fn when_generate_card_visuals_called_then_pattern_index_is_within_bounds() {
        // Arrange
        let sig = CardSignature::new([0.5, -0.3, 0.8, 0.0, -0.1, 0.6, -0.9, 0.2]);

        // Act
        let params = generate_card_visuals(&sig);

        // Assert
        assert!(
            params.pattern_index < PATTERN_COUNT,
            "pattern_index {} must be < PATTERN_COUNT {}",
            params.pattern_index,
            PATTERN_COUNT
        );
    }

    #[test]
    fn when_high_lumines_signature_then_art_color_is_brighter_than_low_lumines() {
        // Arrange — identical signatures except Lumines axis (index 3)
        let mut light_axes = [0.0_f32; 8];
        light_axes[3] = 1.0; // Lumines = max light
        let mut dark_axes = [0.0_f32; 8];
        dark_axes[3] = -1.0; // Lumines = max dark

        let sig_light = CardSignature::new(light_axes);
        let sig_dark = CardSignature::new(dark_axes);

        // Act
        let params_light = generate_card_visuals(&sig_light);
        let params_dark = generate_card_visuals(&sig_dark);

        // Assert — brightness proxy: sum of RGB channels
        let brightness_light =
            params_light.art_color.r + params_light.art_color.g + params_light.art_color.b;
        let brightness_dark =
            params_dark.art_color.r + params_dark.art_color.g + params_dark.art_color.b;
        assert!(
            brightness_light > brightness_dark,
            "light sig brightness {brightness_light} should exceed dark sig brightness {brightness_dark}"
        );
    }

    #[test]
    fn when_high_febris_signature_then_art_color_is_warmer_than_low_febris() {
        // Arrange — identical signatures except Febris axis (index 1)
        let mut heat_axes = [0.0_f32; 8];
        heat_axes[1] = 1.0; // Febris = Heat
        let mut cold_axes = [0.0_f32; 8];
        cold_axes[1] = -1.0; // Febris = Cold

        let sig_heat = CardSignature::new(heat_axes);
        let sig_cold = CardSignature::new(cold_axes);

        // Act
        let params_heat = generate_card_visuals(&sig_heat);
        let params_cold = generate_card_visuals(&sig_cold);

        // Assert — Heat: red > blue, Cold: blue > red
        assert!(
            params_heat.art_color.r > params_heat.art_color.b,
            "heat card should have red ({}) > blue ({})",
            params_heat.art_color.r,
            params_heat.art_color.b
        );
        assert!(
            params_cold.art_color.b > params_cold.art_color.r,
            "cold card should have blue ({}) > red ({})",
            params_cold.art_color.b,
            params_cold.art_color.r
        );
    }

    #[test]
    fn when_signature_serialized_to_json_and_deserialized_then_visuals_are_identical() {
        // Arrange — values that survive f32 JSON roundtrip cleanly
        let original = CardSignature::new([0.5, -0.5, 0.25, -0.25, 0.75, -0.75, 1.0, -1.0]);
        let json = serde_json::to_string(&original).unwrap();
        let restored: CardSignature = serde_json::from_str(&json).unwrap();

        // Act
        let params_original = generate_card_visuals(&original);
        let params_restored = generate_card_visuals(&restored);

        // Assert
        assert_eq!(params_original, params_restored);
    }
}
