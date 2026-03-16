use std::sync::Arc;

use crate::mixer::{MixerTrack, TRACK_COUNT};
use crate::playback::PlaybackId;

pub(crate) struct ActiveSound {
    pub(crate) id: PlaybackId,
    pub(crate) samples: Arc<Vec<f32>>,
    pub(crate) cursor: usize,
    pub(crate) track: MixerTrack,
}

pub(crate) struct SharedState {
    pub(crate) volume: f32,
    pub(crate) next_id: u32,
    pub(crate) active_sounds: Vec<ActiveSound>,
    pub(crate) track_volumes: [f32; TRACK_COUNT],
}

pub(crate) fn mix_into(output: &mut [f32], state: &mut SharedState) {
    output.fill(0.0);

    for sound in &mut state.active_sounds {
        let remaining = sound.samples.len() - sound.cursor;
        let to_mix = remaining.min(output.len());
        let effective_volume = state.volume * state.track_volumes[sound.track.index()];

        for (out, &src) in output[..to_mix]
            .iter_mut()
            .zip(&sound.samples[sound.cursor..sound.cursor + to_mix])
        {
            *out += src * effective_volume;
        }

        sound.cursor += to_mix;
    }

    state.active_sounds.retain(|s| s.cursor < s.samples.len());
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::mixer::MixerTrack;
    use crate::playback::PlaybackId;

    fn test_state(volume: f32, sounds: Vec<ActiveSound>) -> SharedState {
        SharedState {
            volume,
            next_id: sounds.len() as u32,
            active_sounds: sounds,
            track_volumes: [1.0; TRACK_COUNT],
        }
    }

    fn test_state_with_tracks(
        volume: f32,
        track_volumes: [f32; TRACK_COUNT],
        sounds: Vec<ActiveSound>,
    ) -> SharedState {
        SharedState {
            volume,
            next_id: sounds.len() as u32,
            active_sounds: sounds,
            track_volumes,
        }
    }

    fn active(id: u32, samples: Vec<f32>) -> ActiveSound {
        ActiveSound {
            id: PlaybackId(id),
            samples: Arc::new(samples),
            cursor: 0,
            track: MixerTrack::Sfx,
        }
    }

    fn active_on_track(id: u32, samples: Vec<f32>, track: MixerTrack) -> ActiveSound {
        ActiveSound {
            id: PlaybackId(id),
            samples: Arc::new(samples),
            cursor: 0,
            track,
        }
    }

    #[test]
    fn when_single_sound_at_full_volume_then_output_matches_samples() {
        // Arrange
        let mut state = test_state(1.0, vec![active(1, vec![0.2, 0.4, 0.6])]);
        let mut output = vec![0.0; 3];

        // Act
        mix_into(&mut output, &mut state);

        // Assert
        assert!((output[0] - 0.2).abs() < f32::EPSILON);
        assert!((output[1] - 0.4).abs() < f32::EPSILON);
        assert!((output[2] - 0.6).abs() < f32::EPSILON);
    }

    #[test]
    fn when_single_sound_at_half_volume_then_output_is_scaled() {
        // Arrange
        let mut state = test_state(0.5, vec![active(1, vec![0.8, 0.8])]);
        let mut output = vec![0.0; 2];

        // Act
        mix_into(&mut output, &mut state);

        // Assert
        assert!((output[0] - 0.4).abs() < f32::EPSILON);
        assert!((output[1] - 0.4).abs() < f32::EPSILON);
    }

    /// @doc: Audio mixing is additive — all active sounds summed into output buffer, scaled by volume
    #[test]
    fn when_two_active_sounds_then_output_is_sum() {
        // Arrange
        let mut state = test_state(
            1.0,
            vec![active(1, vec![0.3, 0.3]), active(2, vec![0.1, 0.1])],
        );
        let mut output = vec![0.0; 2];

        // Act
        mix_into(&mut output, &mut state);

        // Assert
        assert!((output[0] - 0.4).abs() < f32::EPSILON);
        assert!((output[1] - 0.4).abs() < f32::EPSILON);
    }

    /// @doc: Sounds auto-evict when cursor reaches end — no explicit `stop()` needed for one-shots
    #[test]
    fn when_sound_shorter_than_buffer_then_removed_after_last_sample() {
        // Arrange
        let mut state = test_state(1.0, vec![active(1, vec![0.5, 0.5])]);
        let mut output = vec![0.0; 4];

        // Act
        mix_into(&mut output, &mut state);

        // Assert
        assert!((output[0] - 0.5).abs() < f32::EPSILON);
        assert!((output[1] - 0.5).abs() < f32::EPSILON);
        assert!((output[2]).abs() < f32::EPSILON);
        assert!((output[3]).abs() < f32::EPSILON);
        assert!(state.active_sounds.is_empty());
    }

    #[test]
    fn when_mix_into_with_two_tracks_then_per_track_volume_applied() {
        // Arrange
        let mut track_volumes = [1.0; TRACK_COUNT];
        track_volumes[MixerTrack::Music.index()] = 0.5;
        let mut state = test_state_with_tracks(
            1.0,
            track_volumes,
            vec![
                active_on_track(1, vec![0.8], MixerTrack::Music),
                active_on_track(2, vec![0.4], MixerTrack::Sfx),
            ],
        );
        let mut output = vec![0.0; 1];

        // Act
        mix_into(&mut output, &mut state);

        // Assert: 0.8 * 0.5 + 0.4 * 1.0 = 0.8
        assert!((output[0] - 0.8).abs() < f32::EPSILON);
    }

    /// @doc: Effective volume = `global_volume` * `track_volume` — multiplicative stacking
    #[test]
    fn when_global_and_track_volume_both_half_then_output_quarter() {
        // Arrange
        let mut track_volumes = [1.0; TRACK_COUNT];
        track_volumes[MixerTrack::Music.index()] = 0.5;
        let mut state = test_state_with_tracks(
            0.5,
            track_volumes,
            vec![active_on_track(1, vec![1.0], MixerTrack::Music)],
        );
        let mut output = vec![0.0; 1];

        // Act
        mix_into(&mut output, &mut state);

        // Assert: 1.0 * 0.5 * 0.5 = 0.25
        assert!((output[0] - 0.25).abs() < f32::EPSILON);
    }

    #[test]
    fn when_sound_longer_than_buffer_then_cursor_advances() {
        // Arrange
        let mut state = test_state(1.0, vec![active(1, vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6])]);
        let mut output1 = vec![0.0; 3];
        let mut output2 = vec![0.0; 3];

        // Act
        mix_into(&mut output1, &mut state);
        mix_into(&mut output2, &mut state);

        // Assert
        assert!((output1[0] - 0.1).abs() < f32::EPSILON);
        assert!((output1[1] - 0.2).abs() < f32::EPSILON);
        assert!((output1[2] - 0.3).abs() < f32::EPSILON);
        assert!((output2[0] - 0.4).abs() < f32::EPSILON);
        assert!((output2[1] - 0.5).abs() < f32::EPSILON);
        assert!((output2[2] - 0.6).abs() < f32::EPSILON);
    }

    #[test]
    fn when_sound_partially_consumed_then_only_remaining_samples_mixed() {
        // Arrange — 4 samples, consume 3 in first call, 1 remains
        let mut state = test_state(1.0, vec![active(1, vec![0.1, 0.2, 0.3, 0.4])]);
        let mut output1 = vec![0.0; 3];
        let mut output2 = vec![0.0; 3];

        // Act
        mix_into(&mut output1, &mut state);
        mix_into(&mut output2, &mut state);

        // Assert
        assert!((output2[0] - 0.4).abs() < f32::EPSILON);
        assert!(output2[1].abs() < f32::EPSILON);
        assert!(output2[2].abs() < f32::EPSILON);
    }
}
