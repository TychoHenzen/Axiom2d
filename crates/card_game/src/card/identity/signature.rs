mod algorithms;
mod types;

pub use algorithms::{compute_seed, geometric_level};
pub use types::{Aspect, CardSignature, Element, Rarity, RarityTierConfig};
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::card::identity::signature_profile::Tier;

    #[test]
    fn when_constructing_signature_with_value_above_one_then_axis_is_clamped_to_one() {
        let mut input = [0.0_f32; 8];
        input[3] = 1.5;
        let sig = CardSignature::new(input);
        assert_eq!(sig.axes()[3], 1.0);
    }

    #[test]
    fn when_constructing_signature_with_value_below_minus_one_then_axis_is_clamped_to_minus_one() {
        let mut input = [0.0_f32; 8];
        input[5] = -2.3;
        let sig = CardSignature::new(input);
        assert_eq!(sig.axes()[5], -1.0);
    }

    #[test]
    fn when_constructing_signature_with_all_axes_out_of_range_then_all_axes_are_clamped() {
        let input = [2.0, -3.0, 1.1, -1.1, 5.0, -5.0, 100.0, -100.0];
        let sig = CardSignature::new(input);
        let expected = [1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0, -1.0];
        assert_eq!(sig.axes(), expected);
    }

    #[test]
    fn when_constructing_signature_with_values_in_range_then_values_are_unchanged() {
        let input: [f32; 8] = [0.1, -0.2, 0.5, -0.5, 0.9, -0.9, 0.3, -0.7];
        let sig = CardSignature::new(input);
        assert_eq!(sig.axes(), input);
    }

    #[test]
    fn when_indexing_signature_with_element_enum_then_returns_correct_axis_value() {
        let input = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8];
        let sig = CardSignature::new(input);
        assert_eq!(sig[Element::Solidum], 0.1);
        assert_eq!(sig[Element::Febris], 0.2);
        assert_eq!(sig[Element::Ordinem], 0.3);
        assert_eq!(sig[Element::Lumines], 0.4);
        assert_eq!(sig[Element::Varias], 0.5);
        assert_eq!(sig[Element::Inertiae], 0.6);
        assert_eq!(sig[Element::Subsidium], 0.7);
        assert_eq!(sig[Element::Spatium], 0.8);
    }

    #[test]
    fn when_computing_distance_between_identical_signatures_then_result_is_zero() {
        let sig = CardSignature::new([0.5, -0.3, 0.1, 0.9, -0.7, 0.2, -0.4, 0.6]);
        let dist = sig.distance_to(&sig);
        assert!(dist.abs() < 1e-5, "expected 0.0, got {dist}");
    }

    #[test]
    fn when_computing_distance_between_fully_opposite_signatures_then_result_equals_expected() {
        let a = CardSignature::new([1.0; 8]);
        let b = CardSignature::new([-1.0; 8]);
        let dist = a.distance_to(&b);
        let expected = 32_f32.sqrt();
        assert!(
            (dist - expected).abs() < 1e-5,
            "expected {expected}, got {dist}"
        );
    }

    #[test]
    fn when_computing_distance_then_a_to_b_equals_b_to_a() {
        let a = CardSignature::new([0.1, -0.5, 0.8, -0.2, 0.4, -0.9, 0.3, -0.7]);
        let b = CardSignature::new([-0.3, 0.6, -0.1, 0.7, -0.5, 0.2, -0.8, 0.4]);
        let ab = a.distance_to(&b);
        let ba = b.distance_to(&a);
        assert!((ab - ba).abs() < 1e-5, "a→b={ab}, b→a={ba}");
    }

    #[test]
    fn when_dominant_aspect_called_with_positive_value_then_returns_positive_variant() {
        let mut input = [0.0_f32; 8];
        input[1] = 0.7;
        let sig = CardSignature::new(input);
        assert_eq!(sig.dominant_aspect(Element::Febris), Aspect::Heat);
    }

    #[test]
    fn when_dominant_aspect_called_with_negative_value_then_returns_negative_variant() {
        let mut input = [0.0_f32; 8];
        input[1] = -0.7;
        let sig = CardSignature::new(input);
        assert_eq!(sig.dominant_aspect(Element::Febris), Aspect::Cold);
    }

    #[test]
    fn when_dominant_aspect_called_with_zero_value_then_returns_negative_variant() {
        let sig = CardSignature::new([0.0; 8]);
        assert_eq!(sig.dominant_aspect(Element::Solidum), Aspect::Fragile);
    }

    #[test]
    fn when_dominant_aspect_called_for_all_elements_then_each_returns_distinct_positive_variant() {
        let sig = CardSignature::new([0.5; 8]);
        let elements = Element::ALL;
        let aspects: Vec<Aspect> = elements.iter().map(|e| sig.dominant_aspect(*e)).collect();
        let mut unique = aspects.clone();
        unique.sort_by_key(|a| format!("{a:?}"));
        unique.dedup();
        assert_eq!(
            unique.len(),
            8,
            "expected 8 distinct aspects, got {}",
            unique.len()
        );
    }

    #[test]
    fn when_intensity_called_for_negative_axis_then_returns_absolute_value() {
        let mut input = [0.0_f32; 8];
        input[4] = -0.6;
        let sig = CardSignature::new(input);
        let result = sig.intensity(Element::Varias);
        assert!((result - 0.6).abs() < 1e-5, "expected 0.6, got {result}");
    }

    #[test]
    fn when_intensity_called_for_positive_and_negative_of_same_magnitude_then_both_equal() {
        let mut a_input = [0.0_f32; 8];
        a_input[2] = 0.4;
        let a = CardSignature::new(a_input);
        let mut b_input = [0.0_f32; 8];
        b_input[2] = -0.4;
        let b = CardSignature::new(b_input);
        let ia = a.intensity(Element::Ordinem);
        let ib = b.intensity(Element::Ordinem);
        assert!((ia - ib).abs() < 1e-5, "expected equal, got {ia} vs {ib}");
    }

    #[test]
    fn when_random_with_same_seed_twice_then_results_are_identical() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;

        let mut rng1 = ChaCha8Rng::seed_from_u64(42);
        let mut rng2 = ChaCha8Rng::seed_from_u64(42);
        let sig1 = CardSignature::random(&mut rng1);
        let sig2 = CardSignature::random(&mut rng2);
        assert_eq!(sig1.axes(), sig2.axes());
    }

    #[test]
    fn when_random_signature_generated_then_all_axes_within_bounds() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;

        let seeds = [0_u64, 1, 42, u64::MAX];
        for seed in seeds {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);
            let sig = CardSignature::random(&mut rng);
            for (i, &v) in sig.axes().iter().enumerate() {
                assert!(
                    (-1.0..=1.0).contains(&v),
                    "seed {seed}, axis {i}: value {v} out of bounds"
                );
            }
        }
    }

    #[test]
    fn when_random_with_different_seeds_then_results_differ() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;

        let mut rng1 = ChaCha8Rng::seed_from_u64(0);
        let mut rng2 = ChaCha8Rng::seed_from_u64(1);
        let sig1 = CardSignature::random(&mut rng1);
        let sig2 = CardSignature::random(&mut rng2);
        assert_ne!(sig1.axes(), sig2.axes());
    }

    #[test]
    fn when_subtracting_two_signatures_then_each_axis_is_the_difference() {
        let a = CardSignature::new([0.8, 0.5, -0.2, 0.3, -0.1, 0.6, -0.4, 0.7]);
        let b = CardSignature::new([0.3, 0.2, -0.1, 0.1, -0.3, 0.4, -0.2, 0.5]);
        let result = a.subtract(&b);
        let expected = [0.5, 0.3, -0.1, 0.2, 0.2, 0.2, -0.2, 0.2];
        for (i, &exp) in expected.iter().enumerate() {
            assert!(
                (result.axes()[i] - exp).abs() < 1e-5,
                "axis {i}: expected {exp}, got {}",
                result.axes()[i]
            );
        }
    }

    #[test]
    fn when_subtracting_produces_values_outside_range_then_result_is_clamped() {
        let a = CardSignature::new([1.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let b = CardSignature::new([-1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let result = a.subtract(&b);
        assert_eq!(result.axes()[0], 1.0);
        assert_eq!(result.axes()[1], -1.0);
    }

    #[test]
    fn when_default_rarity_tier_config_constructed_then_advance_rates_are_0_point_3() {
        let config = RarityTierConfig::default();
        assert_eq!(config.rarity_advance_rate, 0.3_f32);
        assert_eq!(config.tier_advance_rate, 0.3_f32);
    }

    #[test]
    fn when_geometric_level_called_with_rate_zero_then_always_returns_first_level() {
        let values = [0.0_f32, 0.1, 0.5, 0.9, 0.99];
        for v in values {
            let level = geometric_level(v, 0.0, 5);
            assert_eq!(level, 0, "value={v}: expected level 0, got {level}");
        }
    }

    #[test]
    fn when_geometric_level_called_with_rate_one_then_always_returns_max_level() {
        let values = [0.0_f32, 0.1, 0.5, 0.9, 0.99];
        let max_levels: usize = 5;
        for v in values {
            let level = geometric_level(v, 1.0, max_levels);
            assert_eq!(
                level,
                max_levels - 1,
                "value={v}: expected level {}, got {level}",
                max_levels - 1
            );
        }
    }

    #[test]
    fn when_geometric_level_called_with_value_below_rate_then_advances_past_first_level() {
        let value = 0.15_f32;
        let advance_rate = 0.3_f32;
        let level = geometric_level(value, advance_rate, 5);
        assert!(level >= 1, "expected level >= 1, got {level}");
    }

    #[test]
    fn when_geometric_level_called_with_value_above_rate_then_stays_at_level_zero() {
        let value = 0.85_f32;
        let advance_rate = 0.3_f32;
        let level = geometric_level(value, advance_rate, 5);
        assert_eq!(level, 0, "expected level 0, got {level}");
    }

    #[test]
    fn when_geometric_level_called_across_full_range_then_levels_are_monotonically_non_increasing()
    {
        let values: Vec<f32> = (0..20).map(|i| i as f32 * 0.05).collect();
        let advance_rate = 0.3_f32;
        let levels: Vec<usize> = values
            .iter()
            .map(|&v| geometric_level(v, advance_rate, 5))
            .collect();
        for i in 1..levels.len() {
            assert!(
                levels[i] <= levels[i - 1],
                "non-monotone at index {i}: level[{}]={} < level[{}]={}",
                i - 1,
                levels[i - 1],
                i,
                levels[i]
            );
        }
    }

    #[test]
    fn when_rarity_called_twice_then_result_is_identical() {
        let sig = CardSignature::new([0.3, -0.7, 0.5, -0.1, 0.9, -0.2, 0.4, -0.8]);
        assert_eq!(sig.rarity(), sig.rarity());
    }

    #[test]
    fn when_two_identical_signatures_compute_rarity_then_results_are_equal() {
        let axes = [0.3, -0.7, 0.5, -0.1, 0.9, -0.2, 0.4, -0.8];
        let sig_a = CardSignature::new(axes);
        let sig_b = CardSignature::new(axes);
        assert_eq!(sig_a.rarity(), sig_b.rarity());
    }

    #[test]
    fn when_many_different_signatures_compute_rarity_then_not_all_the_same() {
        let sigs = [
            CardSignature::new([0.0; 8]),
            CardSignature::new([1.0; 8]),
            CardSignature::new([-1.0; 8]),
            CardSignature::new([0.5; 8]),
            CardSignature::new([-0.5; 8]),
            CardSignature::new([0.1, -0.9, 0.3, -0.7, 0.5, -0.3, 0.8, -0.2]),
            CardSignature::new([0.9, -0.1, 0.7, -0.3, 0.5, -0.5, 0.2, -0.8]),
            CardSignature::new([0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0]),
            CardSignature::new([0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0]),
            CardSignature::new([0.3, 0.3, 0.3, 0.3, 0.3, 0.3, 0.3, 0.3]),
        ];
        let rarities: Vec<Rarity> = sigs.iter().map(CardSignature::rarity).collect();
        let first = rarities[0];
        assert!(
            rarities.iter().any(|&r| r != first),
            "all 10 signatures produced the same rarity {first:?}"
        );
    }

    #[test]
    fn when_rarity_computed_with_default_config_then_result_is_one_of_five_valid_variants() {
        let valid = [
            Rarity::Common,
            Rarity::Uncommon,
            Rarity::Rare,
            Rarity::Epic,
            Rarity::Legendary,
        ];
        let sigs = [
            CardSignature::new([0.0; 8]),
            CardSignature::new([0.5; 8]),
            CardSignature::new([-0.5; 8]),
            CardSignature::new([1.0; 8]),
            CardSignature::new([0.1, -0.9, 0.3, -0.7, 0.5, -0.3, 0.8, -0.2]),
        ];
        for sig in &sigs {
            let r = sig.rarity();
            assert!(
                valid.contains(&r),
                "signature {:?} produced invalid rarity {:?}",
                sig.axes(),
                r
            );
        }
    }

    #[test]
    fn when_many_random_signatures_compute_rarity_then_common_is_most_frequent() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;

        let mut rng = ChaCha8Rng::seed_from_u64(0xdead_beef);
        let sigs: Vec<CardSignature> = (0..1_000)
            .map(|_| CardSignature::random(&mut rng))
            .collect();
        let mut counts = [0usize; 5];
        for sig in &sigs {
            let idx = match sig.rarity() {
                Rarity::Common => 0,
                Rarity::Uncommon => 1,
                Rarity::Rare => 2,
                Rarity::Epic => 3,
                Rarity::Legendary => 4,
            };
            counts[idx] += 1;
        }
        assert!(counts[0] > counts[1]);
        assert!(counts[1] > counts[2]);
        assert!(counts[2] > counts[3]);
        assert!(counts[3] > counts[4]);
    }

    #[test]
    fn when_rarity_computed_with_higher_advance_rate_then_rare_or_above_frequency_increases() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;

        let mut rng = ChaCha8Rng::seed_from_u64(0xcafe_f00d);
        let sigs: Vec<CardSignature> = (0..500).map(|_| CardSignature::random(&mut rng)).collect();
        let default_config = RarityTierConfig::default();
        let high_config = RarityTierConfig {
            rarity_advance_rate: 0.7,
            ..RarityTierConfig::default()
        };
        let rare_above_default = sigs
            .iter()
            .filter(|s| {
                matches!(
                    s.rarity_with_config(&default_config),
                    Rarity::Rare | Rarity::Epic | Rarity::Legendary
                )
            })
            .count();
        let rare_above_high = sigs
            .iter()
            .filter(|s| {
                matches!(
                    s.rarity_with_config(&high_config),
                    Rarity::Rare | Rarity::Epic | Rarity::Legendary
                )
            })
            .count();
        assert!(rare_above_high > rare_above_default);
    }

    #[test]
    fn when_sign_opposite_signatures_compute_rarity_then_both_have_valid_rarity() {
        let sig_pos = CardSignature::new([0.5; 8]);
        let sig_neg = CardSignature::new([-0.5; 8]);
        let valid = [
            Rarity::Common,
            Rarity::Uncommon,
            Rarity::Rare,
            Rarity::Epic,
            Rarity::Legendary,
        ];
        let rarity_pos = sig_pos.rarity();
        let rarity_neg = sig_neg.rarity();
        assert!(valid.contains(&rarity_pos));
        assert!(valid.contains(&rarity_neg));
    }

    #[test]
    fn when_card_tier_computed_twice_then_results_are_identical() {
        let sig = CardSignature::new([0.3, -0.7, 0.5, -0.1, 0.9, -0.2, 0.4, -0.8]);
        assert_eq!(sig.card_tier(), sig.card_tier());
    }

    #[test]
    fn when_card_tier_computed_then_result_is_one_of_three_valid_variants() {
        let valid = [Tier::Dormant, Tier::Active, Tier::Intense];
        let sigs = [
            CardSignature::new([0.0; 8]),
            CardSignature::new([1.0; 8]),
            CardSignature::new([-1.0; 8]),
            CardSignature::new([0.5; 8]),
            CardSignature::new([0.1, -0.9, 0.3, -0.7, 0.5, -0.3, 0.8, -0.2]),
        ];
        for sig in &sigs {
            let t = sig.card_tier();
            assert!(
                valid.contains(&t),
                "signature {:?} produced invalid tier {:?}",
                sig.axes(),
                t
            );
        }
    }

    #[test]
    fn when_many_random_signatures_compute_card_tier_then_dormant_is_most_frequent() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;

        let mut rng = ChaCha8Rng::seed_from_u64(0xdead_beef);
        let sigs: Vec<CardSignature> = (0..1_000)
            .map(|_| CardSignature::random(&mut rng))
            .collect();
        let mut counts = [0usize; 3];
        for sig in &sigs {
            let idx = match sig.card_tier() {
                Tier::Dormant => 0,
                Tier::Active => 1,
                Tier::Intense => 2,
            };
            counts[idx] += 1;
        }
        assert!(counts[0] > counts[1]);
        assert!(counts[1] > counts[2]);
    }

    #[test]
    fn when_rarity_and_card_tier_computed_for_same_signature_then_they_can_differ() {
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;

        let mut rng = ChaCha8Rng::seed_from_u64(42);
        let found = (0..1_000).any(|_| {
            let sig = CardSignature::random(&mut rng);
            sig.rarity() == Rarity::Common && sig.card_tier() != Tier::Dormant
        });
        assert!(found);
    }

    proptest::proptest! {
        #[test]
        fn when_any_valid_axis_values_then_distance_is_non_negative(
            a0 in -1.0_f32..=1.0, a1 in -1.0_f32..=1.0,
            a2 in -1.0_f32..=1.0, a3 in -1.0_f32..=1.0,
            a4 in -1.0_f32..=1.0, a5 in -1.0_f32..=1.0,
            a6 in -1.0_f32..=1.0, a7 in -1.0_f32..=1.0,
            b0 in -1.0_f32..=1.0, b1 in -1.0_f32..=1.0,
            b2 in -1.0_f32..=1.0, b3 in -1.0_f32..=1.0,
            b4 in -1.0_f32..=1.0, b5 in -1.0_f32..=1.0,
            b6 in -1.0_f32..=1.0, b7 in -1.0_f32..=1.0,
        ) {
            let a = CardSignature::new([a0, a1, a2, a3, a4, a5, a6, a7]);
            let b = CardSignature::new([b0, b1, b2, b3, b4, b5, b6, b7]);
            let dist = a.distance_to(&b);
            proptest::prop_assert!(dist >= 0.0, "distance was {dist}");
        }
    }

    proptest::proptest! {
        #[test]
        fn when_any_valid_signature_computes_card_tier_then_always_returns_valid_tier(
            a0 in -1.0_f32..=1.0, a1 in -1.0_f32..=1.0,
            a2 in -1.0_f32..=1.0, a3 in -1.0_f32..=1.0,
            a4 in -1.0_f32..=1.0, a5 in -1.0_f32..=1.0,
            a6 in -1.0_f32..=1.0, a7 in -1.0_f32..=1.0,
        ) {
            let sig = CardSignature::new([a0, a1, a2, a3, a4, a5, a6, a7]);
            let t = sig.card_tier();
            let valid = [Tier::Dormant, Tier::Active, Tier::Intense];
            proptest::prop_assert!(valid.contains(&t));
        }
    }

    proptest::proptest! {
        #[test]
        fn when_any_valid_signature_computes_rarity_then_always_returns_valid_rarity(
            a0 in -1.0_f32..=1.0, a1 in -1.0_f32..=1.0,
            a2 in -1.0_f32..=1.0, a3 in -1.0_f32..=1.0,
            a4 in -1.0_f32..=1.0, a5 in -1.0_f32..=1.0,
            a6 in -1.0_f32..=1.0, a7 in -1.0_f32..=1.0,
        ) {
            let sig = CardSignature::new([a0, a1, a2, a3, a4, a5, a6, a7]);
            let r = sig.rarity();
            let valid = [
                Rarity::Common,
                Rarity::Uncommon,
                Rarity::Rare,
                Rarity::Epic,
                Rarity::Legendary,
            ];
            proptest::prop_assert!(valid.contains(&r));
        }
    }
}
