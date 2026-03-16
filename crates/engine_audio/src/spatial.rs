use std::f32::consts::FRAC_PI_2;

use bevy_ecs::component::Component;
use bevy_ecs::prelude::{Query, ResMut, With, Without};
use glam::Vec2;
use serde::{Deserialize, Serialize};

use engine_scene::prelude::GlobalTransform2D;

use crate::playback::PlaySoundBuffer;

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpatialGains {
    pub left: f32,
    pub right: f32,
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
    mut buffer: ResMut<PlaySoundBuffer>,
) {
    let Ok(listener_transform) = listener_q.single() else {
        return;
    };
    let listener_pos = listener_transform.0.translation;

    for cmd in &mut *buffer {
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use bevy_ecs::prelude::{Schedule, World};
    use bevy_ecs::schedule::IntoScheduleConfigs;
    use engine_core::prelude::Transform2D;
    use engine_scene::prelude::{
        ChildOf, GlobalTransform2D, hierarchy_maintenance_system, transform_propagation_system,
    };
    use glam::Affine2;

    use crate::playback::{PlaySound, PlaySoundBuffer};

    use super::*;

    #[test]
    fn when_audio_emitter_serialized_to_ron_then_deserializes_to_equal_value() {
        // Arrange
        let emitter = AudioEmitter {
            volume: 0.8,
            max_distance: 500.0,
        };

        // Act
        let ron = ron::to_string(&emitter).unwrap();
        let back: AudioEmitter = ron::from_str(&ron).unwrap();

        // Assert
        assert_eq!(emitter, back);
    }

    fn setup_world() -> World {
        let mut world = World::new();
        world.insert_resource(PlaySoundBuffer::default());
        world
    }

    fn spawn_listener(world: &mut World, x: f32, y: f32) -> bevy_ecs::entity::Entity {
        world
            .spawn((
                AudioListener,
                GlobalTransform2D(Affine2::from_translation(Vec2::new(x, y))),
            ))
            .id()
    }

    fn spawn_emitter(
        world: &mut World,
        x: f32,
        y: f32,
        volume: f32,
        max_distance: f32,
    ) -> bevy_ecs::entity::Entity {
        world
            .spawn((
                AudioEmitter {
                    volume,
                    max_distance,
                },
                GlobalTransform2D(Affine2::from_translation(Vec2::new(x, y))),
            ))
            .id()
    }

    fn run_spatial_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(spatial_audio_system);
        schedule.run(world);
    }

    #[test]
    fn when_distance_zero_then_attenuation_is_one() {
        // Act
        let result = distance_attenuation(0.0, 100.0);

        // Assert
        assert!((result - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn when_distance_equals_max_then_attenuation_is_zero() {
        // Act
        let result = distance_attenuation(50.0, 50.0);

        // Assert
        assert!(result.abs() < f32::EPSILON);
    }

    #[test]
    fn when_distance_half_max_then_attenuation_is_half() {
        // Act
        let result = distance_attenuation(50.0, 100.0);

        // Assert
        assert!((result - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn when_distance_exceeds_max_then_attenuation_clamped_to_zero() {
        // Act
        let result = distance_attenuation(200.0, 100.0);

        // Assert
        assert!(result.abs() < f32::EPSILON);
        assert!(result >= 0.0);
    }

    /// @doc: Constant-power stereo panning — emitter fully to the right produces 100% right channel gain
    #[test]
    fn when_emitter_right_of_listener_then_right_gain_one() {
        // Act
        let (left, right) = compute_pan(Vec2::ZERO, Vec2::new(10.0, 0.0));

        // Assert
        assert!(left.abs() < 0.001, "left should be ~0, got {left}");
        assert!(
            (right - 1.0).abs() < 0.001,
            "right should be ~1, got {right}"
        );
    }

    #[test]
    fn when_emitter_left_of_listener_then_left_gain_one() {
        // Act
        let (left, right) = compute_pan(Vec2::ZERO, Vec2::new(-10.0, 0.0));

        // Assert
        assert!((left - 1.0).abs() < 0.001, "left should be ~1, got {left}");
        assert!(right.abs() < 0.001, "right should be ~0, got {right}");
    }

    /// @doc: Centered panning when emitter is on listener's forward axis — no left/right bias
    #[test]
    fn when_emitter_ahead_of_listener_then_gains_equal() {
        // Act
        let (left, right) = compute_pan(Vec2::ZERO, Vec2::new(0.0, 10.0));

        // Assert
        let expected = std::f32::consts::FRAC_1_SQRT_2;
        assert!(
            (left - expected).abs() < 0.001,
            "left should be ~0.707, got {left}"
        );
        assert!(
            (right - expected).abs() < 0.001,
            "right should be ~0.707, got {right}"
        );
    }

    /// @doc: Coincident positions must not produce NaN — atan2(0,0) edge case handled by defaulting to centered pan
    #[test]
    fn when_emitter_at_listener_then_gains_equal_no_nan() {
        // Act
        let (left, right) = compute_pan(Vec2::ZERO, Vec2::ZERO);

        // Assert
        assert!(!left.is_nan());
        assert!(!right.is_nan());
        let expected = std::f32::consts::FRAC_1_SQRT_2;
        assert!(
            (left - expected).abs() < 0.001,
            "left should be ~0.707, got {left}"
        );
        assert!(
            (right - expected).abs() < 0.001,
            "right should be ~0.707, got {right}"
        );
    }

    #[test]
    fn when_listener_nonzero_and_emitter_to_left_then_left_gain_dominates() {
        // Arrange — emitter at x=150, listener at x=200: emitter is LEFT of listener
        let listener = Vec2::new(200.0, 0.0);
        let emitter = Vec2::new(150.0, 0.0);

        // Act
        let (left, right) = compute_pan(listener, emitter);

        // Assert — diff = (-50, 0) → leftward, so left should dominate
        assert!(left > right, "left={left} should exceed right={right}");
    }

    proptest::proptest! {
        #[test]
        fn when_any_distance_then_attenuation_in_zero_to_one(
            distance in 0.0_f32..=1000.0,
            max_distance in 0.001_f32..=1000.0,
        ) {
            // Act
            let result = distance_attenuation(distance, max_distance);

            // Assert
            assert!(
                (0.0..=1.0).contains(&result),
                "attenuation {result} out of [0,1] for distance={distance}, max={max_distance}"
            );
        }

        #[test]
        fn when_any_two_positions_then_constant_power_property_holds(
            lx in -1000.0_f32..=1000.0,
            ly in -1000.0_f32..=1000.0,
            ex in -1000.0_f32..=1000.0,
            ey in -1000.0_f32..=1000.0,
        ) {
            // Act
            let (left, right) = compute_pan(Vec2::new(lx, ly), Vec2::new(ex, ey));

            // Assert — constant-power: left^2 + right^2 ≈ 1.0
            let power = left * left + right * right;
            assert!(
                (power - 1.0).abs() < 1e-4,
                "constant-power violated: L={left}, R={right}, L²+R²={power}"
            );

            // Assert — gains in [0, 1]
            assert!((0.0..=1.0).contains(&left), "left gain {left} out of [0,1]");
            assert!((0.0..=1.0).contains(&right), "right gain {right} out of [0,1]");
        }
    }

    #[test]
    fn when_compute_spatial_gains_then_left_equals_pan_times_attenuation_times_volume() {
        // Arrange — emitter at (50, 0), listener at origin, max_distance=200, volume=0.8
        let listener = Vec2::ZERO;
        let emitter = Vec2::new(50.0, 0.0);

        // Act
        let gains = compute_spatial_gains(listener, emitter, 0.8, 200.0);

        // Assert — manual computation
        let distance = 50.0;
        let atten = distance_attenuation(distance, 200.0); // 1 - 50/200 = 0.75
        let (pan_l, pan_r) = compute_pan(listener, emitter);
        let expected_left = pan_l * atten * 0.8;
        let expected_right = pan_r * atten * 0.8;
        assert!(
            (gains.left - expected_left).abs() < 1e-6,
            "left: expected {expected_left}, got {}",
            gains.left
        );
        assert!(
            (gains.right - expected_right).abs() < 1e-6,
            "right: expected {expected_right}, got {}",
            gains.right
        );
    }

    #[test]
    fn when_compute_spatial_gains_emitter_behind_listener_then_direction_reverses() {
        // Arrange — emitter to the left
        let listener = Vec2::new(100.0, 0.0);
        let emitter = Vec2::new(50.0, 0.0);

        // Act
        let gains = compute_spatial_gains(listener, emitter, 1.0, 200.0);

        // Assert — emitter is to the LEFT of listener, so left gain > right
        assert!(
            gains.left > gains.right,
            "left ({}) should exceed right ({}) for leftward emitter",
            gains.left,
            gains.right
        );
    }

    #[test]
    fn when_emitter_to_right_then_spatial_gains_reflect_pan_and_attenuation() {
        // Arrange
        let mut world = setup_world();
        spawn_listener(&mut world, 0.0, 0.0);
        let emitter = spawn_emitter(&mut world, 50.0, 0.0, 1.0, 100.0);
        world
            .resource_mut::<PlaySoundBuffer>()
            .push(PlaySound::at_emitter("beep", emitter));

        // Act
        run_spatial_system(&mut world);

        // Assert
        let cmds: Vec<_> = world.resource_mut::<PlaySoundBuffer>().drain().collect();
        assert_eq!(cmds.len(), 1);
        let gains = cmds[0].spatial_gains.expect("should have spatial gains");
        assert!(
            gains.right > gains.left,
            "right should be louder: L={} R={}",
            gains.left,
            gains.right
        );
        assert!(
            gains.right < 1.0,
            "attenuation should reduce from full: R={}",
            gains.right
        );
        assert!(gains.right > 0.0);
    }

    /// @doc: Without an `AudioListener` entity, spatial processing is a no-op — gains remain unchanged
    #[test]
    fn when_no_listener_then_system_runs_without_panic() {
        // Arrange
        let mut world = setup_world();
        let emitter = spawn_emitter(&mut world, 50.0, 0.0, 1.0, 100.0);
        world
            .resource_mut::<PlaySoundBuffer>()
            .push(PlaySound::at_emitter("beep", emitter));

        // Act
        run_spatial_system(&mut world);

        // Assert
        let cmds: Vec<_> = world.resource_mut::<PlaySoundBuffer>().drain().collect();
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].spatial_gains.is_none());
    }

    /// @doc: Linear distance attenuation drops to zero beyond `max_distance`, effectively culling inaudible sounds
    #[test]
    fn when_emitter_beyond_max_distance_then_gains_are_zero() {
        // Arrange
        let mut world = setup_world();
        spawn_listener(&mut world, 0.0, 0.0);
        let emitter = spawn_emitter(&mut world, 200.0, 0.0, 1.0, 100.0);
        world
            .resource_mut::<PlaySoundBuffer>()
            .push(PlaySound::at_emitter("beep", emitter));

        // Act
        run_spatial_system(&mut world);

        // Assert
        let cmds: Vec<_> = world.resource_mut::<PlaySoundBuffer>().drain().collect();
        let gains = cmds[0].spatial_gains.expect("should have spatial gains");
        assert!(gains.left.abs() < f32::EPSILON);
        assert!(gains.right.abs() < f32::EPSILON);
    }

    #[test]
    fn when_nonzero_listener_and_fractional_volume_then_gains_exact() {
        // Arrange — listener at (100,0), emitter at (150,0): diff = (50,0), distance = 50
        // atten = 1 - 50/200 = 0.75; emitter right of listener → right gain ≈ 1.0
        // expected right = 1.0 * 0.75 * 0.5 = 0.375
        let listener = Vec2::new(100.0, 0.0);
        let emitter = Vec2::new(150.0, 0.0);

        // Act
        let gains = compute_spatial_gains(listener, emitter, 0.5, 200.0);

        // Assert
        assert!(
            (gains.right - 0.375).abs() < 1e-4,
            "expected right ≈ 0.375, got {}",
            gains.right
        );
        // left pan is ~0 when fully right-panned
        assert!(gains.right > gains.left, "should be right-panned");
    }

    #[test]
    fn when_emitter_ahead_then_spatial_gains_multiply_pan_by_attenuation() {
        // Arrange — centered panning (emitter directly ahead), non-trivial attenuation
        let listener = Vec2::ZERO;
        let emitter = Vec2::new(0.0, 50.0);

        // Act
        let gains = compute_spatial_gains(listener, emitter, 1.0, 100.0);

        // Assert — both channels should be attenuated (not amplified)
        assert!(
            gains.left < 1.0,
            "left must be attenuated, got {}",
            gains.left
        );
        assert!(
            gains.right < 1.0,
            "right must be attenuated, got {}",
            gains.right
        );
        let expected = std::f32::consts::FRAC_1_SQRT_2 * 0.5;
        assert!(
            (gains.left - expected).abs() < 1e-4,
            "left expected {expected}, got {}",
            gains.left
        );
    }

    #[test]
    fn when_emitter_at_epsilon_distance_then_panned_not_centered() {
        // Arrange — emitter displaced only in X by a tiny but non-zero amount.
        // 2^-11 squared = 2^-22 > EPSILON (2^-23), so length_squared > EPSILON and
        // the coincident-position guard does not fire — compute_pan normalizes the
        // direction and returns a right-biased result, not the centered fallback.
        // This test catches regressions where the guard is widened (e.g. via a larger
        // threshold) causing tiny-displacement inputs to be incorrectly centered.
        let x = 2.0_f32.powi(-11);

        // Act
        let (left, right) = compute_pan(Vec2::ZERO, Vec2::new(x, 0.0));

        // Assert — should be right-panned (right ≈ 1.0), not centered (≈ 0.707)
        assert!(
            right > left + 0.1,
            "should be right-panned, not centered: left={left}, right={right}"
        );
    }

    #[test]
    fn when_play_sound_without_emitter_then_gains_unchanged() {
        // Arrange
        let mut world = setup_world();
        spawn_listener(&mut world, 0.0, 0.0);
        world
            .resource_mut::<PlaySoundBuffer>()
            .push(PlaySound::new("beep"));

        // Act
        run_spatial_system(&mut world);

        // Assert
        let cmds: Vec<_> = world.resource_mut::<PlaySoundBuffer>().drain().collect();
        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].spatial_gains.is_none());
    }

    /// @doc: Spatial audio uses `GlobalTransform2D` (world space), not local `Transform2D` — hierarchy must propagate first
    #[test]
    fn when_emitter_is_child_entity_then_world_position_used() {
        // Arrange
        let mut world = setup_world();
        spawn_listener(&mut world, 0.0, 0.0);

        // Parent at (80, 0), child emitter at local (0, 0) -> world (80, 0)
        let parent = world
            .spawn((Transform2D {
                position: Vec2::new(80.0, 0.0),
                ..Default::default()
            },))
            .id();
        let child = world
            .spawn((
                Transform2D::default(),
                ChildOf(parent),
                AudioEmitter {
                    volume: 1.0,
                    max_distance: 200.0,
                },
            ))
            .id();

        // Run hierarchy + transform propagation first
        let mut schedule = Schedule::default();
        schedule.add_systems(
            (
                hierarchy_maintenance_system,
                transform_propagation_system,
                spatial_audio_system,
            )
                .chain(),
        );
        world
            .resource_mut::<PlaySoundBuffer>()
            .push(PlaySound::at_emitter("beep", child));
        schedule.run(&mut world);

        // Assert
        let cmds: Vec<_> = world.resource_mut::<PlaySoundBuffer>().drain().collect();
        assert_eq!(cmds.len(), 1);
        let gains = cmds[0].spatial_gains.expect("should have spatial gains");
        // Emitter at world (80, 0) relative to listener at (0, 0)
        // -> right-panned, distance attenuation = 1 - 80/200 = 0.6
        assert!(gains.right > gains.left, "should be right-panned");
        assert!(
            gains.right > 0.0 && gains.right < 1.0,
            "should have distance attenuation"
        );
    }
}
