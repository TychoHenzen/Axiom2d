#![allow(clippy::unwrap_used)]

use engine_audio::backend::{AudioBackend, NullAudioBackend};
use engine_audio::mixer::MixerTrack;
use engine_audio::playback::PlaybackId;
use engine_audio::sound::SoundData;

fn minimal_sound() -> SoundData {
    SoundData {
        samples: vec![0.0],
        sample_rate: 44_100,
        channels: 1,
    }
}

#[test]
fn when_play_on_track_with_sfx_then_play_count_increments() {
    // Arrange
    let mut backend = NullAudioBackend::new();
    let sound = minimal_sound();

    // Act
    backend.play_on_track(&sound, MixerTrack::Sfx);

    // Assert
    assert_eq!(backend.play_count(), 1);
}

/// @doc: Each playback gets a unique ID — enables stopping individual sounds without affecting others
#[test]
fn when_play_on_track_called_twice_with_sfx_then_ids_differ() {
    // Arrange
    let mut backend = NullAudioBackend::new();
    let sound = minimal_sound();

    // Act
    let id1 = backend.play_on_track(&sound, MixerTrack::Sfx);
    let id2 = backend.play_on_track(&sound, MixerTrack::Sfx);

    // Assert
    assert_ne!(id1, id2);
}

#[test]
fn when_stop_called_then_does_not_panic() {
    // Arrange
    let mut backend = NullAudioBackend::new();

    // Act
    backend.stop(PlaybackId(42));
}

#[test]
fn when_set_volume_called_then_does_not_panic() {
    // Arrange
    let mut backend = NullAudioBackend::new();

    // Act
    backend.set_volume(0.0);
    backend.set_volume(0.5);
    backend.set_volume(1.0);
}

#[test]
fn when_set_track_volume_on_null_backend_then_no_panic() {
    // Arrange
    let mut backend = NullAudioBackend::new();

    // Act
    backend.set_track_volume(MixerTrack::Music, 0.5);
    backend.set_track_volume(MixerTrack::Sfx, 0.0);
}

#[test]
fn when_play_on_track_called_then_play_count_increments() {
    // Arrange
    let mut backend = NullAudioBackend::new();
    let sound = minimal_sound();

    // Act
    backend.play_on_track(&sound, MixerTrack::Music);

    // Assert
    assert_eq!(backend.play_count(), 1);
}

/// @doc: Play count accumulates across calls — tracks total sounds queued for playback
#[test]
fn when_three_sounds_played_then_play_count_returns_three() {
    // Arrange
    let mut backend = NullAudioBackend::new();
    let sound = minimal_sound();

    // Act
    backend.play_on_track(&sound, MixerTrack::Sfx);
    backend.play_on_track(&sound, MixerTrack::Sfx);
    backend.play_on_track(&sound, MixerTrack::Sfx);

    // Assert
    assert_eq!(backend.play_count(), 3);
}
