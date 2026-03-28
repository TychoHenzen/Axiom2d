use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use super::traits::AudioBackend;
use crate::mixer::MixerTrack;
use crate::mixer_engine::{ActiveSound, SharedState, mix_into};
use crate::playback::PlaybackId;
use crate::sound::SoundData;

use crate::mixer::TRACK_COUNT;

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
        let Some(device) = host.default_output_device() else {
            tracing::warn!("no audio output device found — audio will be silent");
            return None;
        };
        let config = device.default_output_config().ok()?;
        let sample_format = config.sample_format();
        let sample_rate = config.sample_rate().0;
        let config: cpal::StreamConfig = config.into();

        let stream = match sample_format {
            cpal::SampleFormat::F32 => match device.build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut state = state
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    mix_into(data, &mut state);
                },
                |err| tracing::error!("audio stream error: {err}"),
                None,
            ) {
                Ok(s) => s,
                Err(err) => {
                    tracing::warn!("failed to build audio stream: {err}");
                    return None;
                }
            },
            _ => return None,
        };

        match stream.play() {
            Ok(()) => {
                tracing::info!(sample_rate, "audio stream opened");
                Some(stream)
            }
            Err(err) => {
                tracing::warn!("failed to start audio stream: {err}");
                None
            }
        }
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
    fn play_on_track(&mut self, sound: &SoundData, track: MixerTrack) -> PlaybackId {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
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
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.active_sounds.retain(|s| s.id != id);
    }

    fn set_volume(&mut self, volume: f32) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.volume = volume;
    }

    fn set_track_volume(&mut self, track: MixerTrack, volume: f32) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.track_volumes[track.index()] = volume;
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_helpers::minimal_sound;

    fn sound_with_samples(samples: Vec<f32>) -> SoundData {
        SoundData {
            samples,
            sample_rate: 44_100,
            channels: 1,
        }
    }

    /// @doc: CPAL backend assigns unique IDs per playback — isolation enables per-sound lifecycle control
    #[test]
    fn when_play_called_twice_then_ids_are_unique() {
        // Arrange
        let mut backend = CpalBackend::new();
        let sound = minimal_sound();

        // Act
        let id1 = backend.play_on_track(&sound, MixerTrack::Sfx);
        let id2 = backend.play_on_track(&sound, MixerTrack::Sfx);

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
        let _id = backend.play_on_track(&sound, MixerTrack::Sfx);

        // Assert
        assert_eq!(backend.active_sound_count(), 1);
    }

    #[test]
    fn when_stop_called_then_sound_removed_from_active_list() {
        // Arrange
        let mut backend = CpalBackend::new();
        let id = backend.play_on_track(&sound_with_samples(vec![0.5]), MixerTrack::Sfx);

        // Act
        backend.stop(id);

        // Assert
        assert_eq!(backend.active_sound_count(), 0);
    }

    #[test]
    fn when_two_sounds_and_stop_one_then_other_remains() {
        // Arrange
        let mut backend = CpalBackend::new();
        let id1 = backend.play_on_track(&sound_with_samples(vec![0.5]), MixerTrack::Sfx);
        let _id2 = backend.play_on_track(&sound_with_samples(vec![0.3]), MixerTrack::Sfx);

        // Act
        backend.stop(id1);

        // Assert
        assert_eq!(backend.active_sound_count(), 1);
    }

    /// @doc: Per-track volume isolation — setting one track does not affect others in shared state
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
    fn when_set_volume_called_then_volume_changes() {
        // Arrange
        let mut backend = CpalBackend::new();

        // Act
        backend.set_volume(0.5);

        // Assert
        assert!((backend.volume() - 0.5).abs() < f32::EPSILON);
    }
}
