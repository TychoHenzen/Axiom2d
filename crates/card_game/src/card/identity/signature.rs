use serde::{Deserialize, Serialize};

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
        for i in 0..8 {
            result[i] = self.axes[i] - other.axes[i];
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
        let raw_score: f32 = self.axes.iter().map(|v| v.abs()).sum();
        let normalized = raw_score.ln_1p() / 8.0_f32.ln_1p();
        match normalized {
            n if n >= 0.85 => Rarity::Legendary,
            n if n >= 0.70 => Rarity::Epic,
            n if n >= 0.50 => Rarity::Rare,
            n if n >= 0.30 => Rarity::Uncommon,
            _ => Rarity::Common,
        }
    }
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
        for i in 0..8 {
            assert!(
                (result.axes()[i] - expected[i]).abs() < 1e-5,
                "axis {i}: expected {}, got {}",
                expected[i],
                result.axes()[i]
            );
        }
    }

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

    // ===== rarity =====

    #[test]
    fn when_all_axes_zero_then_rarity_is_common() {
        // Arrange
        let sig = CardSignature::new([0.0; 8]);

        // Act / Assert
        assert_eq!(sig.rarity(), Rarity::Common);
    }

    #[test]
    fn when_all_axes_at_max_then_rarity_is_legendary() {
        // Arrange
        let sig = CardSignature::new([1.0; 8]);

        // Act / Assert
        assert_eq!(sig.rarity(), Rarity::Legendary);
    }

    #[test]
    fn when_all_axes_at_negative_max_then_rarity_is_legendary() {
        // Arrange
        let sig = CardSignature::new([-1.0; 8]);

        // Act / Assert
        assert_eq!(sig.rarity(), Rarity::Legendary);
    }

    #[test]
    fn when_rarity_called_twice_then_result_is_identical() {
        // Arrange
        let sig = CardSignature::new([0.3, -0.7, 0.5, -0.1, 0.9, -0.2, 0.4, -0.8]);

        // Act / Assert
        assert_eq!(sig.rarity(), sig.rarity());
    }

    #[test]
    fn when_more_extreme_signature_then_rarity_is_equal_or_higher() {
        // Arrange
        let low = CardSignature::new([0.1; 8]);
        let mid = CardSignature::new([0.5; 8]);
        let high = CardSignature::new([0.9; 8]);

        // Assert
        assert!((low.rarity() as u8) <= (mid.rarity() as u8));
        assert!((mid.rarity() as u8) <= (high.rarity() as u8));
    }

    #[test]
    fn when_moderate_extremity_then_rarity_is_between_common_and_legendary() {
        // Arrange
        let sig = CardSignature::new([0.4, -0.3, 0.2, -0.5, 0.1, -0.2, 0.3, -0.4]);

        // Act / Assert
        assert_ne!(sig.rarity(), Rarity::Legendary);
        assert_ne!(sig.rarity(), Rarity::Common);
    }
}
