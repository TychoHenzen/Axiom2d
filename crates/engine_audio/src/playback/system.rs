use bevy_ecs::prelude::{Res, ResMut};
use engine_core::prelude::EventBus;

use super::buffer::PlaySound;
use crate::audio_res::AudioRes;
use crate::mixer::MixerState;
use crate::sound::SoundData;
use crate::sound::SoundLibrary;
use crate::spatial::SpatialGains;

const DEFAULT_SAMPLE_RATE: u32 = 44_100;
const DEFAULT_DURATION: f32 = 0.5;

pub fn play_sound_system(
    mut bus: ResMut<EventBus<PlaySound>>,
    library: Option<Res<SoundLibrary>>,
    mixer: Option<Res<MixerState>>,
    mut audio: ResMut<AudioRes>,
) {
    let Some(library) = library else {
        // Drain bus even when no library to avoid unbounded growth.
        bus.drain().for_each(drop);
        return;
    };

    if let Some(mixer) = &mixer {
        for track in crate::mixer::MixerTrack::ALL {
            audio.set_track_volume(track, mixer.track_volume(track));
        }
    }

    for cmd in bus.drain() {
        if let Some(effect) = library.get(&cmd.name) {
            let sound = effect.synthesize(DEFAULT_SAMPLE_RATE, DEFAULT_DURATION);
            let sound = if let Some(gains) = cmd.spatial_gains {
                if gains.left.abs() < f32::EPSILON && gains.right.abs() < f32::EPSILON {
                    continue;
                }
                apply_spatial_gains(&sound, gains)
            } else {
                sound
            };
            audio.play_on_track(&sound, cmd.track);
        }
    }
}

fn apply_spatial_gains(sound: &SoundData, gains: SpatialGains) -> SoundData {
    let frame_count = sound.frame_count();
    let mut stereo_samples = Vec::with_capacity(frame_count * 2);

    if sound.channels == 1 {
        for &sample in &sound.samples {
            stereo_samples.push(sample * gains.left);
            stereo_samples.push(sample * gains.right);
        }
    } else {
        for frame in sound.samples.chunks(sound.channels as usize) {
            let left = frame.first().copied().unwrap_or(0.0);
            let right = frame.get(1).copied().unwrap_or(0.0);
            stereo_samples.push(left * gains.left);
            stereo_samples.push(right * gains.right);
        }
    }

    SoundData {
        samples: stereo_samples,
        sample_rate: sound.sample_rate,
        channels: 2,
    }
}
