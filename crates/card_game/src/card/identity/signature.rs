use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RarityTierConfig {
    pub rarity_advance_rate: f32,
    pub tier_advance_rate: f32,
}

impl Default for RarityTierConfig {
    fn default() -> Self {
        Self {
            rarity_advance_rate: 0.3,
            tier_advance_rate: 0.3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Element {
    Solidum,
    Febris,
    Ordinem,
    Lumines,
    Varias,
    Inertiae,
    Subsidium,
    Spatium,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Aspect {
    // Solidum
    Solid,
    Fragile,
    // Febris
    Heat,
    Cold,
    // Ordinem
    Order,
    Chaos,
    // Lumines
    Light,
    Dark,
    // Varias
    Change,
    Stasis,
    // Inertiae
    Force,
    Calm,
    // Subsidium
    Growth,
    Decay,
    // Spatium
    Expansion,
    Contraction,
}

impl Element {
    pub const ALL: [Self; 8] = [
        Self::Solidum,
        Self::Febris,
        Self::Ordinem,
        Self::Lumines,
        Self::Varias,
        Self::Inertiae,
        Self::Subsidium,
        Self::Spatium,
    ];

    fn index(self) -> usize {
        self as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CardSignature {
    axes: [f32; 8],
}

impl Default for CardSignature {
    fn default() -> Self {
        Self { axes: [0.0; 8] }
    }
}

impl CardSignature {
    pub fn new(values: [f32; 8]) -> Self {
        let mut axes = values;
        for v in &mut axes {
            *v = v.clamp(-1.0, 1.0);
        }
        Self { axes }
    }

    pub fn random(rng: &mut rand_chacha::ChaCha8Rng) -> Self {
        use rand::Rng;
        let mut axes = [0.0; 8];
        for v in &mut axes {
            *v = rng.gen_range(-1.0..=1.0);
        }
        Self { axes }
    }

    pub fn subtract(&self, other: &Self) -> Self {
        let mut result = [0.0; 8];
        for (i, val) in result.iter_mut().enumerate() {
            *val = self.axes[i] - other.axes[i];
        }
        Self::new(result)
    }

    pub fn distance_to(&self, other: &Self) -> f32 {
        self.axes
            .iter()
            .zip(other.axes.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f32>()
            .sqrt()
    }

    pub fn intensity(&self, element: Element) -> f32 {
        self.axes[element.index()].abs()
    }

    pub fn dominant_aspect(&self, element: Element) -> Aspect {
        let value = self.axes[element.index()];
        if value > 0.0 {
            match element {
                Element::Solidum => Aspect::Solid,
                Element::Febris => Aspect::Heat,
                Element::Ordinem => Aspect::Order,
                Element::Lumines => Aspect::Light,
                Element::Varias => Aspect::Change,
                Element::Inertiae => Aspect::Force,
                Element::Subsidium => Aspect::Growth,
                Element::Spatium => Aspect::Expansion,
            }
        } else {
            match element {
                Element::Solidum => Aspect::Fragile,
                Element::Febris => Aspect::Cold,
                Element::Ordinem => Aspect::Chaos,
                Element::Lumines => Aspect::Dark,
                Element::Varias => Aspect::Stasis,
                Element::Inertiae => Aspect::Calm,
                Element::Subsidium => Aspect::Decay,
                Element::Spatium => Aspect::Contraction,
            }
        }
    }

    pub fn axes(&self) -> [f32; 8] {
        self.axes
    }

    pub fn rarity(&self) -> Rarity {
        self.rarity_with_config(&RarityTierConfig::default())
    }

    pub fn card_tier(&self) -> crate::card::identity::signature_profile::Tier {
        self.card_tier_with_config(&RarityTierConfig::default())
    }

    pub fn card_tier_with_config(
        &self,
        config: &RarityTierConfig,
    ) -> crate::card::identity::signature_profile::Tier {
        use crate::card::identity::signature_profile::Tier;
        let seed = compute_seed(self);
        // Use high 32 bits (rarity uses low 32) for independence
        let value = (seed >> 32) as f32 / u32::MAX as f32;
        let level = geometric_level(value, config.tier_advance_rate, 3);
        match level {
            0 => Tier::Dormant,
            1 => Tier::Active,
            _ => Tier::Intense,
        }
    }

    pub fn rarity_with_config(&self, config: &RarityTierConfig) -> Rarity {
        let seed = compute_seed(self);
        let value = (seed & 0xFFFF_FFFF) as f32 / u32::MAX as f32;
        let level = geometric_level(value, config.rarity_advance_rate, 5);
        match level {
            0 => Rarity::Common,
            1 => Rarity::Uncommon,
            2 => Rarity::Rare,
            3 => Rarity::Epic,
            _ => Rarity::Legendary,
        }
    }
}

/// Hash a `CardSignature` into a stable 64-bit seed.
///
/// Used by rarity/tier assignment (different bit ranges for independence) and by visual parameter
/// generation. Placing this here avoids a circular dependency between `signature` and
/// `visual_params`.
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

pub fn geometric_level(value: f32, advance_rate: f32, max_levels: usize) -> usize {
    let mut remaining = value;
    for level in 0..max_levels - 1 {
        if remaining >= advance_rate {
            return level;
        }
        remaining /= advance_rate;
    }
    max_levels - 1
}

impl std::ops::Index<Element> for CardSignature {
    type Output = f32;

    fn index(&self, element: Element) -> &f32 {
        &self.axes[element.index()]
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::card::identity::signature_profile::Tier;

    /// @doc: Signature axes are clamped to [-1, 1] — prevents runaway values from signature arithmetic
    #[test]
    fn when_constructing_signature_with_value_above_one_then_axis_is_clamped_to_one() {
        // Arrange
        let mut input = [0.0_f32; 8];
        input[3] = 1.5;

        // Act
        let sig = CardSignature::new(input);

        // Assert
        assert_eq!(sig.axes()[3], 1.0);
    }

    #[test]
    fn when_constructing_signature_with_value_below_minus_one_then_axis_is_clamped_to_minus_one() {
        // Arrange
        let mut input = [0.0_f32; 8];
        input[5] = -2.3;

        // Act
        let sig = CardSignature::new(input);

        // Assert
        assert_eq!(sig.axes()[5], -1.0);
    }

    #[test]
    fn when_constructing_signature_with_all_axes_out_of_range_then_all_axes_are_clamped() {
        // Arrange
        let input = [2.0, -3.0, 1.1, -1.1, 5.0, -5.0, 100.0, -100.0];

        // Act
        let sig = CardSignature::new(input);

        // Assert
        let expected = [1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0, -1.0];
        assert_eq!(sig.axes(), expected);
    }

    /// @doc: Element-based indexing maps each enum variant to its axis position
    /// in the 8D signature array. If the mapping drifted (e.g., from reordering
    /// Element variants), every card's identity would silently scramble — Febris
    /// cards would display Ordinem art.
    #[test]
    fn when_indexing_signature_with_element_enum_then_returns_correct_axis_value() {
        // Arrange
        let input = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8];
        let sig = CardSignature::new(input);

        // Act & Assert
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
        // Arrange
        let sig = CardSignature::new([0.5, -0.3, 0.1, 0.9, -0.7, 0.2, -0.4, 0.6]);

        // Act
        let dist = sig.distance_to(&sig);

        // Assert
        assert!((dist).abs() < 1e-5, "expected 0.0, got {dist}");
    }

    #[test]
    fn when_computing_distance_between_fully_opposite_signatures_then_result_equals_expected() {
        // Arrange
        let a = CardSignature::new([1.0; 8]);
        let b = CardSignature::new([-1.0; 8]);

        // Act
        let dist = a.distance_to(&b);

        // Assert
        let expected = 32_f32.sqrt();
        assert!(
            (dist - expected).abs() < 1e-5,
            "expected {expected}, got {dist}"
        );
    }

    /// @doc: Distance is symmetric — base type matching doesn't depend on which signature is the reference point
    #[test]
    fn when_computing_distance_then_a_to_b_equals_b_to_a() {
        // Arrange
        let a = CardSignature::new([0.1, -0.5, 0.8, -0.2, 0.4, -0.9, 0.3, -0.7]);
        let b = CardSignature::new([-0.3, 0.6, -0.1, 0.7, -0.5, 0.2, -0.8, 0.4]);

        // Act
        let ab = a.distance_to(&b);
        let ba = b.distance_to(&a);

        // Assert
        assert!((ab - ba).abs() < 1e-5, "a→b={ab}, b→a={ba}");
    }

    #[test]
    fn when_dominant_aspect_called_with_positive_value_then_returns_positive_variant() {
        // Arrange
        let mut input = [0.0_f32; 8];
        input[1] = 0.7; // Febris
        let sig = CardSignature::new(input);

        // Act
        let aspect = sig.dominant_aspect(Element::Febris);

        // Assert
        assert_eq!(aspect, Aspect::Heat);
    }

    #[test]
    fn when_dominant_aspect_called_with_negative_value_then_returns_negative_variant() {
        // Arrange
        let mut input = [0.0_f32; 8];
        input[1] = -0.7; // Febris
        let sig = CardSignature::new(input);

        // Act
        let aspect = sig.dominant_aspect(Element::Febris);

        // Assert
        assert_eq!(aspect, Aspect::Cold);
    }

    #[test]
    fn when_dominant_aspect_called_with_zero_value_then_returns_negative_variant() {
        // Arrange
        let sig = CardSignature::new([0.0; 8]);

        // Act
        let aspect = sig.dominant_aspect(Element::Solidum);

        // Assert
        assert_eq!(aspect, Aspect::Fragile);
    }

    #[test]
    fn when_dominant_aspect_called_for_all_elements_then_each_returns_distinct_positive_variant() {
        // Arrange
        let sig = CardSignature::new([0.5; 8]);
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

        // Act
        let aspects: Vec<Aspect> = elements.iter().map(|e| sig.dominant_aspect(*e)).collect();

        // Assert
        let mut unique = aspects.clone();
        unique.sort_by_key(|a| format!("{a:?}"));
        unique.dedup();
        assert_eq!(
            unique.len(),
            8,
            "expected 8 distinct aspects, got {}: {aspects:?}",
            unique.len()
        );
    }

    #[test]
    fn when_intensity_called_for_negative_axis_then_returns_absolute_value() {
        // Arrange
        let mut input = [0.0_f32; 8];
        input[4] = -0.6; // Varias
        let sig = CardSignature::new(input);

        // Act
        let result = sig.intensity(Element::Varias);

        // Assert
        assert!((result - 0.6).abs() < 1e-5, "expected 0.6, got {result}");
    }

    #[test]
    fn when_intensity_called_for_positive_and_negative_of_same_magnitude_then_both_equal() {
        // Arrange
        let mut a_input = [0.0_f32; 8];
        a_input[2] = 0.4;
        let a = CardSignature::new(a_input);

        let mut b_input = [0.0_f32; 8];
        b_input[2] = -0.4;
        let b = CardSignature::new(b_input);

        // Act
        let ia = a.intensity(Element::Ordinem);
        let ib = b.intensity(Element::Ordinem);

        // Assert
        assert!((ia - ib).abs() < 1e-5, "expected equal, got {ia} vs {ib}");
    }

    #[test]
    fn when_random_with_same_seed_twice_then_results_are_identical() {
        // Arrange
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;
        let mut rng1 = ChaCha8Rng::seed_from_u64(42);
        let mut rng2 = ChaCha8Rng::seed_from_u64(42);

        // Act
        let sig1 = CardSignature::random(&mut rng1);
        let sig2 = CardSignature::random(&mut rng2);

        // Assert
        assert_eq!(sig1.axes(), sig2.axes());
    }

    #[test]
    fn when_random_signature_generated_then_all_axes_within_bounds() {
        // Arrange
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;
        let seeds = [0_u64, 1, 42, u64::MAX];

        for seed in seeds {
            let mut rng = ChaCha8Rng::seed_from_u64(seed);

            // Act
            let sig = CardSignature::random(&mut rng);

            // Assert
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
        // Arrange
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;
        let mut rng1 = ChaCha8Rng::seed_from_u64(0);
        let mut rng2 = ChaCha8Rng::seed_from_u64(1);

        // Act
        let sig1 = CardSignature::random(&mut rng1);
        let sig2 = CardSignature::random(&mut rng2);

        // Assert
        assert_ne!(sig1.axes(), sig2.axes());
    }

    #[test]
    fn when_subtracting_two_signatures_then_each_axis_is_the_difference() {
        // Arrange
        let a = CardSignature::new([0.8, 0.5, -0.2, 0.3, -0.1, 0.6, -0.4, 0.7]);
        let b = CardSignature::new([0.3, 0.2, -0.1, 0.1, -0.3, 0.4, -0.2, 0.5]);

        // Act
        let result = a.subtract(&b);

        // Assert
        let expected = [0.5, 0.3, -0.1, 0.2, 0.2, 0.2, -0.2, 0.2];
        for (i, &exp) in expected.iter().enumerate() {
            assert!(
                (result.axes()[i] - exp).abs() < 1e-5,
                "axis {i}: expected {exp}, got {}",
                result.axes()[i]
            );
        }
    }

    /// @doc: Subtraction result is clamped — residual computation can't produce out-of-range signature axes
    #[test]
    fn when_subtracting_produces_values_outside_range_then_result_is_clamped() {
        // Arrange
        let a = CardSignature::new([1.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let b = CardSignature::new([-1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);

        // Act
        let result = a.subtract(&b);

        // Assert — raw differences would be 2.0 and -2.0, both clamped
        assert_eq!(result.axes()[0], 1.0);
        assert_eq!(result.axes()[1], -1.0);
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

    #[test]
    fn when_constructing_signature_with_values_in_range_then_values_are_unchanged() {
        // Arrange
        let input: [f32; 8] = [0.1, -0.2, 0.5, -0.5, 0.9, -0.9, 0.3, -0.7];

        // Act
        let sig = CardSignature::new(input);

        // Assert
        assert_eq!(sig.axes(), input);
    }

    // ===== config =====

    /// @doc: The 70/30 geometric split is load-bearing for distribution balance.
    /// If either advance rate drifts from 0.3, the probability of graduating from
    /// Common to Uncommon (and from any tier to the next) changes, silently skewing
    /// the entire card-pool distribution. The default is the contract; this test
    /// pins it so any accidental change is caught at CI rather than in playtesting.
    #[test]
    fn when_default_rarity_tier_config_constructed_then_advance_rates_are_0_point_3() {
        // Arrange
        // (no preconditions beyond the type existing)

        // Act
        let config = RarityTierConfig::default();

        // Assert
        assert_eq!(config.rarity_advance_rate, 0.3_f32);
        assert_eq!(config.tier_advance_rate, 0.3_f32);
    }

    // ===== geometric_level =====

    /// @doc: `advance_rate=0.0` is the degenerate case — no value is ever below 0.0,
    /// so the advance condition (value < `advance_rate`) is never satisfied and every
    /// input stays at level 0.
    #[test]
    fn when_geometric_level_called_with_rate_zero_then_always_returns_first_level() {
        // Arrange
        let values = [0.0_f32, 0.1, 0.5, 0.9, 0.99];

        // Act & Assert
        for v in values {
            let level = geometric_level(v, 0.0, 5);
            assert_eq!(level, 0, "value={v}: expected level 0, got {level}");
        }
    }

    /// @doc: `advance_rate=1.0` means every value in [0,1) satisfies value < 1.0,
    /// so each stage always advances — every input reaches the maximum level.
    #[test]
    fn when_geometric_level_called_with_rate_one_then_always_returns_max_level() {
        // Arrange
        let values = [0.0_f32, 0.1, 0.5, 0.9, 0.99];
        let max_levels: usize = 5;

        // Act & Assert
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

    /// @doc: With `advance_rate=0.3`, a value of 0.15 is below the threshold and
    /// qualifies to advance — it must reach at least level 1.
    #[test]
    fn when_geometric_level_called_with_value_below_rate_then_advances_past_first_level() {
        // Arrange
        let value = 0.15_f32;
        let advance_rate = 0.3_f32;

        // Act
        let level = geometric_level(value, advance_rate, 5);

        // Assert
        assert!(level >= 1, "expected level >= 1, got {level}");
    }

    /// @doc: With `advance_rate=0.3`, a value of 0.85 is above the threshold —
    /// the advance condition is not met and the card stays at level 0.
    #[test]
    fn when_geometric_level_called_with_value_above_rate_then_stays_at_level_zero() {
        // Arrange
        let value = 0.85_f32;
        let advance_rate = 0.3_f32;

        // Act
        let level = geometric_level(value, advance_rate, 5);

        // Assert
        assert_eq!(level, 0, "expected level 0, got {level}");
    }

    /// @doc: Because lower values advance and higher values stay, the level sequence
    /// over increasing input values must be non-increasing — a larger draw value must
    /// never yield a higher level than a smaller one.
    #[test]
    fn when_geometric_level_called_across_full_range_then_levels_are_monotonically_non_increasing()
    {
        // Arrange
        let values: Vec<f32> = (0..20).map(|i| i as f32 * 0.05).collect();
        let advance_rate = 0.3_f32;

        // Act
        let levels: Vec<usize> = values
            .iter()
            .map(|&v| geometric_level(v, advance_rate, 5))
            .collect();

        // Assert
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

    // ===== rarity =====

    #[test]
    fn when_rarity_called_twice_then_result_is_identical() {
        // Arrange
        let sig = CardSignature::new([0.3, -0.7, 0.5, -0.1, 0.9, -0.2, 0.4, -0.8]);

        // Act / Assert
        assert_eq!(sig.rarity(), sig.rarity());
    }

    /// @doc: Hash-based rarity is deterministic — two independently constructed signatures
    /// with identical axes must produce the same Rarity, so cards are reproducible across
    /// save/load and multiplayer sync.
    #[test]
    fn when_two_identical_signatures_compute_rarity_then_results_are_equal() {
        // Arrange
        let axes = [0.3, -0.7, 0.5, -0.1, 0.9, -0.2, 0.4, -0.8];
        let sig_a = CardSignature::new(axes);
        let sig_b = CardSignature::new(axes);

        // Act
        let rarity_a = sig_a.rarity();
        let rarity_b = sig_b.rarity();

        // Assert
        assert_eq!(rarity_a, rarity_b);
    }

    /// @doc: Rarity must vary across distinct signatures — if the hash collapsed all inputs
    /// to the same value every card would share a rarity, breaking the rarity system entirely.
    /// We sample 10 diverse signatures; at least two must produce different rarities.
    #[test]
    fn when_many_different_signatures_compute_rarity_then_not_all_the_same() {
        // Arrange
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

        // Act
        let rarities: Vec<Rarity> = sigs.iter().map(CardSignature::rarity).collect();

        // Assert — not all the same
        let first = rarities[0];
        assert!(
            rarities.iter().any(|&r| r != first),
            "all 10 signatures produced the same rarity {first:?} — hash is not discriminating"
        );
    }

    /// @doc: Every possible rarity output must be a recognised Rarity variant — the level-to-enum
    /// mapping must be exhaustive and never produce an out-of-range index.
    #[test]
    fn when_rarity_computed_with_default_config_then_result_is_one_of_five_valid_variants() {
        // Arrange
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

        // Act & Assert
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

    /// @doc: With `advance_rate=0.3` and a uniform hash distribution, Common is the most probable
    /// outcome (probability 0.7) and each subsequent tier is rarer by factor 0.3.  Verifying the
    /// ordering over 1 000 seeded samples guards against an accidentally inverted level mapping.
    #[test]
    fn when_many_random_signatures_compute_rarity_then_common_is_most_frequent() {
        // Arrange
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;
        let mut rng = ChaCha8Rng::seed_from_u64(0xdead_beef);
        let sigs: Vec<CardSignature> = (0..1_000)
            .map(|_| CardSignature::random(&mut rng))
            .collect();

        // Act
        let mut counts = [0usize; 5]; // indexed by Rarity discriminant
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

        // Assert — each tier must be strictly less frequent than the one before it
        assert!(
            counts[0] > counts[1],
            "Common ({}) should outnumber Uncommon ({})",
            counts[0],
            counts[1]
        );
        assert!(
            counts[1] > counts[2],
            "Uncommon ({}) should outnumber Rare ({})",
            counts[1],
            counts[2]
        );
        assert!(
            counts[2] > counts[3],
            "Rare ({}) should outnumber Epic ({})",
            counts[2],
            counts[3]
        );
        assert!(
            counts[3] > counts[4],
            "Epic ({}) should outnumber Legendary ({})",
            counts[3],
            counts[4]
        );
    }

    /// @doc: A higher `rarity_advance_rate` widens the advance window at each level, so a larger
    /// fraction of hashes reach Rare or above. `rarity_with_config()` must expose this knob so
    /// designers can tune pool composition without touching source code.
    #[test]
    fn when_rarity_computed_with_higher_advance_rate_then_rare_or_above_frequency_increases() {
        // Arrange
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;
        let mut rng = ChaCha8Rng::seed_from_u64(0xcafe_f00d);
        let sigs: Vec<CardSignature> = (0..500).map(|_| CardSignature::random(&mut rng)).collect();

        let default_config = RarityTierConfig::default(); // advance_rate = 0.3
        let high_config = RarityTierConfig {
            rarity_advance_rate: 0.7,
            ..RarityTierConfig::default()
        };

        // Act
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

        // Assert
        assert!(
            rare_above_high > rare_above_default,
            "high advance_rate ({}) should produce more Rare+ cards than default ({}) — got {} vs {}",
            high_config.rarity_advance_rate,
            default_config.rarity_advance_rate,
            rare_above_high,
            rare_above_default
        );
    }

    /// @doc: Sign-opposite signatures are fundamentally different cards (Heat vs Cold, etc.) and
    /// must each resolve to a valid rarity without panicking or producing garbage.
    #[test]
    fn when_sign_opposite_signatures_compute_rarity_then_both_have_valid_rarity() {
        // Arrange
        let sig_pos = CardSignature::new([0.5; 8]);
        let sig_neg = CardSignature::new([-0.5; 8]);
        let valid = [
            Rarity::Common,
            Rarity::Uncommon,
            Rarity::Rare,
            Rarity::Epic,
            Rarity::Legendary,
        ];

        // Act
        let rarity_pos = sig_pos.rarity();
        let rarity_neg = sig_neg.rarity();

        // Assert
        assert!(
            valid.contains(&rarity_pos),
            "+0.5 signature produced invalid rarity {rarity_pos:?}"
        );
        assert!(
            valid.contains(&rarity_neg),
            "-0.5 signature produced invalid rarity {rarity_neg:?}"
        );
    }

    // ===== card_tier =====

    /// @doc: Card-level tier is deterministic via hash — the same signature always produces
    /// the same tier. Without this, a card's visual treatment (worn/shiny) would flicker
    /// on reload or across multiplayer clients.
    #[test]
    fn when_card_tier_computed_twice_then_results_are_identical() {
        // Arrange
        let sig = CardSignature::new([0.3, -0.7, 0.5, -0.1, 0.9, -0.2, 0.4, -0.8]);

        // Act
        let tier_a = sig.card_tier();
        let tier_b = sig.card_tier();

        // Assert
        assert_eq!(tier_a, tier_b);
    }

    /// @doc: Card-level tier must resolve to one of the three defined variants for any
    /// valid signature. An unmapped geometric level would panic or produce undefined behavior.
    #[test]
    fn when_card_tier_computed_then_result_is_one_of_three_valid_variants() {
        // Arrange
        let valid = [Tier::Dormant, Tier::Active, Tier::Intense];
        let sigs = [
            CardSignature::new([0.0; 8]),
            CardSignature::new([1.0; 8]),
            CardSignature::new([-1.0; 8]),
            CardSignature::new([0.5; 8]),
            CardSignature::new([0.1, -0.9, 0.3, -0.7, 0.5, -0.3, 0.8, -0.2]),
        ];

        // Act & Assert
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

    /// @doc: With `advance_rate=0.3`, Dormant should be most frequent (~70%), Active next (~21%),
    /// and Intense rarest (~9%). This guards against an inverted level mapping that would make
    /// most cards Intense.
    #[test]
    fn when_many_random_signatures_compute_card_tier_then_dormant_is_most_frequent() {
        // Arrange
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;
        let mut rng = ChaCha8Rng::seed_from_u64(0xdead_beef);
        let sigs: Vec<CardSignature> = (0..1_000)
            .map(|_| CardSignature::random(&mut rng))
            .collect();

        // Act
        let mut counts = [0usize; 3];
        for sig in &sigs {
            let idx = match sig.card_tier() {
                Tier::Dormant => 0,
                Tier::Active => 1,
                Tier::Intense => 2,
            };
            counts[idx] += 1;
        }

        // Assert
        assert!(
            counts[0] > counts[1],
            "Dormant ({}) should outnumber Active ({})",
            counts[0],
            counts[1]
        );
        assert!(
            counts[1] > counts[2],
            "Active ({}) should outnumber Intense ({})",
            counts[1],
            counts[2]
        );
    }

    /// @doc: Rarity and card-level tier use different hash bits, so they are independent
    /// variables. At least one signature must exist where the two classifications diverge
    /// (e.g., Common rarity but Active/Intense tier). Without independence, rarity and tier
    /// would be redundant, wasting a design axis.
    #[test]
    fn when_rarity_and_card_tier_computed_for_same_signature_then_they_can_differ() {
        // Arrange
        use rand::SeedableRng;
        use rand_chacha::ChaCha8Rng;
        let mut rng = ChaCha8Rng::seed_from_u64(42);

        // Act — find any signature where rarity is Common but tier is not Dormant
        let found = (0..1_000).any(|_| {
            let sig = CardSignature::random(&mut rng);
            sig.rarity() == Rarity::Common && sig.card_tier() != Tier::Dormant
        });

        // Assert
        assert!(
            found,
            "among 1000 signatures, should find at least one where rarity and tier diverge"
        );
    }

    proptest::proptest! {
        /// @doc: Card-level tier must never panic for any valid signature input.
        /// The 3-level geometric distribution should always map to a valid Tier variant.
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
            proptest::prop_assert!(
                valid.contains(&t),
                "signature axes {:?} produced invalid tier {:?}",
                sig.axes(),
                t
            );
        }
    }

    proptest::proptest! {
        /// @doc: Rarity must always be one of the five defined variants for any valid signature.
        /// An unmapped hash level (e.g., level 5 when only 0..=4 are handled) would be a panic
        /// or undefined behaviour — this property test exercises the full f32 input space.
        #[test]
        fn when_any_valid_signature_computes_rarity_then_always_returns_valid_rarity(
            a0 in -1.0_f32..=1.0, a1 in -1.0_f32..=1.0,
            a2 in -1.0_f32..=1.0, a3 in -1.0_f32..=1.0,
            a4 in -1.0_f32..=1.0, a5 in -1.0_f32..=1.0,
            a6 in -1.0_f32..=1.0, a7 in -1.0_f32..=1.0,
        ) {
            // Arrange
            let sig = CardSignature::new([a0, a1, a2, a3, a4, a5, a6, a7]);

            // Act
            let r = sig.rarity();

            // Assert
            let valid = [
                Rarity::Common,
                Rarity::Uncommon,
                Rarity::Rare,
                Rarity::Epic,
                Rarity::Legendary,
            ];
            proptest::prop_assert!(
                valid.contains(&r),
                "signature axes {:?} produced invalid rarity {:?}",
                sig.axes(),
                r
            );
        }
    }
}
