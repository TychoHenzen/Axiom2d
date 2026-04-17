use std::f32::consts::FRAC_PI_2;

use bevy_ecs::component::Component;
use bevy_ecs::prelude::{Query, ResMut, With, Without};
use glam::Vec2;
use serde::{Deserialize, Serialize};

use engine_scene::prelude::GlobalTransform2D;

use crate::playback::{PlaySound, SpatialGains};
use engine_core::prelude::EventBus;

/// Linear distance attenuation: 1.0 at distance 0, 0.0 at `max_distance`, clamped.
#[must_use]
pub fn distance_attenuation(distance: f32, max_distance: f32) -> f32 {
    (1.0 - distance / max_distance).clamp(0.0, 1.0)
}

/// Constant-power stereo panning from listener to emitter positions.
/// Returns `(left_gain, right_gain)`. Centered when emitter is directly
/// ahead or at the same position as the listener.
#[must_use]
pub fn compute_pan(listener_pos: Vec2, emitter_pos: Vec2) -> (f32, f32) {
    let diff = emitter_pos - listener_pos;
    let direction = if diff.length_squared() < f32::EPSILON {
        Vec2::Y
    } else {
        diff.normalize()
    };

    let pan = ((direction.x + 1.0) * 0.5).clamp(0.0, 1.0);
    let left = (FRAC_PI_2 * pan).cos().max(0.0);
    let right = (FRAC_PI_2 * pan).sin().max(0.0);
    (left, right)
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct AudioListener;

#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AudioEmitter {
    pub volume: f32,
    pub max_distance: f32,
}

/// Computes spatial gains for an emitter relative to a listener.
#[must_use]
pub fn compute_spatial_gains(
    listener_pos: Vec2,
    emitter_pos: Vec2,
    emitter_volume: f32,
    max_distance: f32,
) -> SpatialGains {
    let diff = emitter_pos - listener_pos;
    let distance = diff.length();
    let attenuation = distance_attenuation(distance, max_distance);
    let (pan_left, pan_right) = compute_pan(listener_pos, emitter_pos);
    SpatialGains {
        left: pan_left * attenuation * emitter_volume,
        right: pan_right * attenuation * emitter_volume,
    }
}

pub fn spatial_audio_system(
    listener_q: Query<&GlobalTransform2D, With<AudioListener>>,
    emitter_q: Query<(&AudioEmitter, &GlobalTransform2D), Without<AudioListener>>,
    mut bus: ResMut<EventBus<PlaySound>>,
) {
    let Ok(listener_transform) = listener_q.single() else {
        return;
    };
    let listener_pos = listener_transform.0.translation;

    for cmd in &mut *bus {
        if cmd.spatial_gains.is_some() {
            continue;
        }

        let Some(emitter_entity) = cmd.emitter else {
            continue;
        };

        if let Ok((emitter, transform)) = emitter_q.get(emitter_entity) {
            let emitter_pos = transform.0.translation;
            cmd.spatial_gains = Some(compute_spatial_gains(
                listener_pos,
                emitter_pos,
                emitter.volume,
                emitter.max_distance,
            ));
        }
    }
}
