#![allow(clippy::unwrap_used)]

use engine_audio::backend::{AudioBackend, CpalBackend};
use engine_audio::mixer::MixerTrack;
use engine_audio::sound::SoundData;

fn minimal_sound() -> SoundData {
    SoundData {
        samples: vec![0.0],
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

// TODO: needs pub promotion — CpalBackend::active_sound_count() is #[cfg(test)] only
// #[test]
// fn when_play_called_then_active_sound_added()

// TODO: needs pub promotion — CpalBackend::active_sound_count() is #[cfg(test)] only
// #[test]
// fn when_stop_called_then_sound_removed_from_active_list()

// TODO: needs pub promotion — CpalBackend::active_sound_count() is #[cfg(test)] only
// #[test]
// fn when_two_sounds_and_stop_one_then_other_remains()

// TODO: needs pub promotion — CpalBackend::track_volume() is #[cfg(test)] only
// #[test]
// fn when_set_track_volume_on_cpal_then_internal_state_updated()

// TODO: needs pub promotion — CpalBackend::volume() is #[cfg(test)] only
// #[test]
// fn when_set_volume_called_then_volume_changes()
