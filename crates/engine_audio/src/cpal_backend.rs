use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::audio_backend::AudioBackend;
use crate::playback_id::PlaybackId;
use crate::sound_data::SoundData;

use crate::mixer::MixerTrack;

struct ActiveSound {
    id: PlaybackId,
    samples: Arc<Vec<f32>>,
    cursor: usize,
    track: MixerTrack,
}

use crate::mixer::TRACK_COUNT;

struct SharedState {
    volume: f32,
    next_id: u32,
    active_sounds: Vec<ActiveSound>,
    track_volumes: [f32; TRACK_COUNT],
}

fn mix_into(output: &mut [f32], state: &mut SharedState) {
    for sample in output.iter_mut() {
        *sample = 0.0;
    }

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

#[allow(dead_code)]
struct StreamHandle(cpal::Stream);

// SAFETY: cpal::Stream is an OS handle to an audio callback thread.
// It does not access thread-local state and is safe to send/share.
unsafe impl Send for StreamHandle {}
unsafe impl Sync for StreamHandle {}

pub struct CpalBackend {
    state: Arc<Mutex<SharedState>>,
    _stream: Option<StreamHandle>,
}

impl CpalBackend {
    #[must_use]
    pub fn new() -> Self {
        let state = Arc::new(Mutex::new(SharedState {
            volume: 1.0,
            next_id: 0,
            active_sounds: Vec::new(),
            track_volumes: [1.0; TRACK_COUNT],
        }));

        let stream = Self::open_stream(Arc::clone(&state));

        Self {
            state,
            _stream: stream.map(StreamHandle),
        }
    }

    fn open_stream(state: Arc<Mutex<SharedState>>) -> Option<cpal::Stream> {
        let host = cpal::default_host();
        let device = host.default_output_device()?;
        let config = device.default_output_config().ok()?;
        let sample_format = config.sample_format();
        let config: cpal::StreamConfig = config.into();

        let stream = match sample_format {
            cpal::SampleFormat::F32 => device
                .build_output_stream(
                    &config,
                    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        let mut state = state.lock().expect("audio state lock poisoned");
                        mix_into(data, &mut state);
                    },
                    |err| eprintln!("audio stream error: {err}"),
                    None,
                )
                .ok()?,
            _ => return None,
        };

        stream.play().ok()?;
        Some(stream)
    }

    #[cfg(test)]
    fn volume(&self) -> f32 {
        self.state.lock().expect("lock poisoned").volume
    }

    #[cfg(test)]
    fn track_volume(&self, track: MixerTrack) -> f32 {
        self.state.lock().expect("lock poisoned").track_volumes[track.index()]
    }

    #[cfg(test)]
    fn active_sound_count(&self) -> usize {
        self.state
            .lock()
            .expect("lock poisoned")
            .active_sounds
            .len()
    }
}

impl Default for CpalBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioBackend for CpalBackend {
    fn play(&mut self, sound: &SoundData) -> PlaybackId {
        self.play_on_track(sound, MixerTrack::Sfx)
    }

    fn play_on_track(&mut self, sound: &SoundData, track: MixerTrack) -> PlaybackId {
        let mut state = self.state.lock().expect("audio state lock poisoned");
        state.next_id += 1;
        let id = PlaybackId(state.next_id);
        state.active_sounds.push(ActiveSound {
            id,
            samples: Arc::new(sound.samples.clone()),
            cursor: 0,
            track,
        });
        id
    }

    fn stop(&mut self, id: PlaybackId) {
        let mut state = self.state.lock().expect("audio state lock poisoned");
        state.active_sounds.retain(|s| s.id != id);
    }

    fn set_volume(&mut self, volume: f32) {
        let mut state = self.state.lock().expect("audio state lock poisoned");
        state.volume = volume;
    }

    fn set_track_volume(&mut self, track: MixerTrack, volume: f32) {
        let mut state = self.state.lock().expect("audio state lock poisoned");
        state.track_volumes[track.index()] = volume;
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn minimal_sound() -> SoundData {
        SoundData {
            samples: vec![0.0],
            sample_rate: 44_100,
            channels: 1,
        }
    }

    fn sound_with_samples(samples: Vec<f32>) -> SoundData {
        SoundData {
            samples,
            sample_rate: 44_100,
            channels: 1,
        }
    }

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
    fn when_constructed_then_volume_is_one() {
        // Act
        let backend = CpalBackend::new();

        // Assert
        assert!((backend.volume() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn when_play_called_twice_then_ids_are_unique() {
        // Arrange
        let mut backend = CpalBackend::new();
        let sound = minimal_sound();

        // Act
        let id1 = backend.play(&sound);
        let id2 = backend.play(&sound);

        // Assert
        assert_ne!(id1, id2);
    }

    #[test]
    fn when_stop_with_unknown_id_then_does_not_panic() {
        // Arrange
        let mut backend = CpalBackend::new();

        // Act
        backend.stop(PlaybackId(999));
    }

    #[test]
    fn when_play_called_then_active_sound_added() {
        // Arrange
        let mut backend = CpalBackend::new();
        let sound = sound_with_samples(vec![0.5, 0.5]);

        // Act
        let _id = backend.play(&sound);

        // Assert
        assert_eq!(backend.active_sound_count(), 1);
    }

    #[test]
    fn when_stop_called_then_sound_removed_from_active_list() {
        // Arrange
        let mut backend = CpalBackend::new();
        let id = backend.play(&sound_with_samples(vec![0.5]));

        // Act
        backend.stop(id);

        // Assert
        assert_eq!(backend.active_sound_count(), 0);
    }

    #[test]
    fn when_two_sounds_and_stop_one_then_other_remains() {
        // Arrange
        let mut backend = CpalBackend::new();
        let id1 = backend.play(&sound_with_samples(vec![0.5]));
        let _id2 = backend.play(&sound_with_samples(vec![0.3]));

        // Act
        backend.stop(id1);

        // Assert
        assert_eq!(backend.active_sound_count(), 1);
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
    fn when_set_track_volume_on_cpal_then_internal_state_updated() {
        // Arrange
        let mut backend = CpalBackend::new();

        // Act
        backend.set_track_volume(MixerTrack::Sfx, 0.6);

        // Assert
        assert!((backend.track_volume(MixerTrack::Sfx) - 0.6).abs() < f32::EPSILON);
        assert!((backend.track_volume(MixerTrack::Music) - 1.0).abs() < f32::EPSILON);
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
}
