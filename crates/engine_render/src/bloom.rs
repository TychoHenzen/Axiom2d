use bevy_ecs::prelude::{Res, ResMut, Resource};

use crate::renderer::RendererRes;

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct BloomSettings {
    pub enabled: bool,
    pub threshold: f32,
    pub intensity: f32,
    pub blur_radius: u32,
}

impl Default for BloomSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            threshold: 0.8,
            intensity: 0.3,
            blur_radius: 4,
        }
    }
}

pub fn post_process_system(
    settings: Option<Res<BloomSettings>>,
    mut renderer: ResMut<RendererRes>,
) {
    if settings.as_ref().is_some_and(|s| s.enabled) {
        renderer.apply_post_process();
    }
}

pub fn compute_gaussian_weights(radius: u32) -> Vec<f32> {
    let sigma = radius.max(1) as f32;
    let size = 2 * radius + 1;
    let mut weights: Vec<f32> = (0..size)
        .map(|i| {
            let x = i as f32 - radius as f32;
            (-x * x / (2.0 * sigma * sigma)).exp()
        })
        .collect();
    let sum: f32 = weights.iter().sum();
    weights.iter_mut().for_each(|w| *w /= sum);
    weights
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_gaussian_weights_radius1_then_center_is_largest() {
        // Act
        let weights = compute_gaussian_weights(1);

        // Assert
        assert_eq!(weights.len(), 3);
        assert!(weights[1] > weights[0]);
        assert!(weights[1] > weights[2]);
    }

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

    #[cfg(feature = "testing")]
    mod system_tests {
        use std::sync::{Arc, Mutex};

        use bevy_ecs::prelude::*;

        use crate::renderer::RendererRes;
        use crate::testing::SpyRenderer;

        use super::*;

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
    }
}
