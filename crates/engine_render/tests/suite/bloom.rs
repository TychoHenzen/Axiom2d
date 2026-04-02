#![allow(clippy::unwrap_used)]

use engine_render::bloom::*;

#[test]
fn when_gaussian_weights_radius1_then_center_is_largest() {
    // Act
    let weights = compute_gaussian_weights(1);

    // Assert
    assert_eq!(weights.len(), 3);
    assert!(weights[1] > weights[0]);
    assert!(weights[1] > weights[2]);
}

/// @doc: Normalized kernel ensures bloom doesn't change overall image brightness
#[test]
fn when_gaussian_weights_radius3_then_sum_is_one() {
    // Act
    let weights = compute_gaussian_weights(3);

    // Assert
    assert_eq!(weights.len(), 7);
    let sum: f32 = weights.iter().sum();
    assert!(
        (sum - 1.0).abs() < 1e-5,
        "weights must sum to 1.0, got {sum}"
    );
}

/// @doc: Symmetry allows separable (H+V) blur — same kernel for both passes
#[test]
fn when_gaussian_weights_computed_then_kernel_is_symmetric() {
    // Act
    let weights = compute_gaussian_weights(3);

    // Assert
    let n = weights.len();
    for i in 0..=3 {
        let mirror = n - 1 - i;
        assert!(
            (weights[i] - weights[mirror]).abs() < 1e-5,
            "weights[{i}]={} != weights[{mirror}]={}",
            weights[i],
            weights[mirror],
        );
    }
}

#[test]
fn when_gaussian_weights_radius0_then_single_weight_of_one() {
    // Act
    let weights = compute_gaussian_weights(0);

    // Assert
    assert_eq!(weights.len(), 1);
    assert!(
        (weights[0] - 1.0).abs() < 1e-5,
        "single weight must be 1.0, got {}",
        weights[0],
    );
}

mod system_tests {
    use std::sync::{Arc, Mutex};

    use bevy_ecs::prelude::*;

    use engine_render::bloom::*;
    use engine_render::renderer::RendererRes;
    use engine_render::testing::SpyRenderer;

    #[test]
    fn when_post_process_system_runs_then_log_records_apply_post_process() {
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log.clone());
        let mut world = World::new();
        world.insert_resource(RendererRes::new(Box::new(spy)));
        world.insert_resource(BloomSettings::default());
        let mut schedule = Schedule::default();
        schedule.add_systems(post_process_system);

        // Act
        schedule.run(&mut world);

        // Assert
        assert!(
            log.lock()
                .unwrap()
                .contains(&"apply_post_process".to_string())
        );
    }

    #[test]
    fn when_bloom_disabled_then_post_process_system_skips() {
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log.clone());
        let mut world = World::new();
        world.insert_resource(RendererRes::new(Box::new(spy)));
        world.insert_resource(BloomSettings {
            enabled: false,
            ..BloomSettings::default()
        });
        let mut schedule = Schedule::default();
        schedule.add_systems(post_process_system);

        // Act
        schedule.run(&mut world);

        // Assert
        assert!(
            !log.lock()
                .unwrap()
                .contains(&"apply_post_process".to_string())
        );
    }

    /// @doc: `BloomSettings` is opt-in — no resource insertion means zero post-process overhead
    #[test]
    fn when_no_bloom_settings_then_post_process_system_skips() {
        // Arrange
        let log = Arc::new(Mutex::new(Vec::new()));
        let spy = SpyRenderer::new(log.clone());
        let mut world = World::new();
        world.insert_resource(RendererRes::new(Box::new(spy)));
        let mut schedule = Schedule::default();
        schedule.add_systems(post_process_system);

        // Act
        schedule.run(&mut world);

        // Assert
        assert!(
            !log.lock()
                .unwrap()
                .contains(&"apply_post_process".to_string())
        );
    }
}

proptest::proptest! {
    #[test]
    fn when_any_radius_then_gaussian_weights_sum_to_one_and_are_symmetric(
        radius in 0u32..=16,
    ) {
        // Act
        let weights = compute_gaussian_weights(radius);
        let size = weights.len();

        // Assert — length
        assert_eq!(size, (2 * radius + 1) as usize);

        // Assert — sum to 1.0
        let sum: f32 = weights.iter().sum();
        assert!(
            (sum - 1.0).abs() < 1e-5,
            "weights must sum to 1.0, got {sum}"
        );

        // Assert — symmetry
        for i in 0..size / 2 {
            let mirror = size - 1 - i;
            assert!(
                (weights[i] - weights[mirror]).abs() < 1e-6,
                "weights[{i}]={} != weights[{mirror}]={}",
                weights[i],
                weights[mirror],
            );
        }

        // Assert — center is maximum
        let center = radius as usize;
        for (i, w) in weights.iter().enumerate() {
            assert!(
                weights[center] >= *w,
                "center {} should be >= weights[{i}]={}",
                weights[center],
                w,
            );
        }
    }
}

#[test]
fn when_gaussian_weights_radius3_then_weight_ratios_match_formula() {
    // The Gaussian formula: w(x) = exp(-x^2 / (2*sigma^2)), sigma = max(radius, 1)
    // For radius=3, sigma=3: denominator = 2*3*3 = 18
    // This radius is chosen so 2*sigma != 2+sigma (6 != 5),
    // catching mutations like `2.0 * sigma` → `2.0 + sigma`.
    let weights = compute_gaussian_weights(3);

    // Assert
    assert_eq!(weights.len(), 7);
    let center = weights[3]; // x=0
    let adjacent = weights[2]; // x=-1
    let edge = weights[0]; // x=-3

    // Ratio w(0)/w(1) = exp(1/18)
    let expected_adj_ratio = (1.0_f32 / 18.0).exp();
    let actual_adj_ratio = center / adjacent;
    assert!(
        (actual_adj_ratio - expected_adj_ratio).abs() < 1e-4,
        "center/adjacent ratio: expected {expected_adj_ratio}, got {actual_adj_ratio}"
    );

    // Ratio w(0)/w(3) = exp(9/18) = exp(0.5)
    let expected_edge_ratio = (0.5_f32).exp();
    let actual_edge_ratio = center / edge;
    assert!(
        (actual_edge_ratio - expected_edge_ratio).abs() < 1e-4,
        "center/edge ratio: expected {expected_edge_ratio}, got {actual_edge_ratio}"
    );
}
