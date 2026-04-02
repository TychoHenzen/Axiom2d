use std::sync::Arc;

use crate::mixer::{MixerTrack, TRACK_COUNT};
use crate::playback::PlaybackId;

pub struct ActiveSound {
    pub id: PlaybackId,
    pub samples: Arc<Vec<f32>>,
    pub cursor: usize,
    pub track: MixerTrack,
}

pub struct SharedState {
    pub volume: f32,
    pub next_id: u32,
    pub active_sounds: Vec<ActiveSound>,
    pub track_volumes: [f32; TRACK_COUNT],
}

pub fn mix_into(output: &mut [f32], state: &mut SharedState) {
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
