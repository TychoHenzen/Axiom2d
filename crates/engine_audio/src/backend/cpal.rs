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
        let sample_rate = config.sample_rate();
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
