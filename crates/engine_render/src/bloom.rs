use bevy_ecs::prelude::{Res, ResMut, Resource};
use serde::{Deserialize, Serialize};

use crate::renderer::RendererRes;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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
    for w in &mut weights {
        *w /= sum;
    }
    weights
}
