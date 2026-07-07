#![allow(clippy::unwrap_used)]

use engine_audio::backend::{AudioBackend, CpalBackend, NullAudioBackend};
use engine_audio::mixer::MixerTrack;
use engine_audio::sound::SoundData;

fn minimal_sound() -> SoundData {
    SoundData {
        samples: vec![0.0],
        sample_rate: 44_100,
        channels: 1,
    }
}

fn silent_samples(len: usize) -> Vec<f32> {
    vec![0.0; len]
}

// ---------------------------------------------------------------------------
// CPAL backend — hardware-id uniqueness (real audio device init)
// ---------------------------------------------------------------------------

/// @doc: CPAL backend assigns unique IDs per playback — isolation enables per-sound lifecycle control
#[test]
fn when_cpal_play_called_twice_then_ids_are_unique() {
    // Arrange
    let mut backend = CpalBackend::new();
    let sound = minimal_sound();

    // Act
    let id1 = backend.play_on_track(&sound, MixerTrack::Sfx);
    let id2 = backend.play_on_track(&sound, MixerTrack::Sfx);

    // Assert
    assert_ne!(
        id1, id2,
        "consecutive play calls on CPAL backend must return distinct PlaybackIds"
    );
}

// ---------------------------------------------------------------------------
// NullAudioBackend — deterministic, no hardware dependency
// ---------------------------------------------------------------------------

/// @doc: Null backend assigns monotonically increasing IDs from 1
#[test]
fn when_null_play_called_then_ids_are_monotonic() {
    // Arrange
    let mut backend = NullAudioBackend::new();
    let sound = minimal_sound();

    // Act
    let id1 = backend.play_on_track(&sound, MixerTrack::Sfx);
    let id2 = backend.play_on_track(&sound, MixerTrack::Music);
    let id3 = backend.play_on_track(&sound, MixerTrack::Ambient);

    // Assert
    assert_eq!(id1.0, 1, "first playback ID should be 1");
    assert_eq!(id2.0, 2, "second playback ID should be 2");
    assert_eq!(id3.0, 3, "third playback ID should be 3");
}

/// @doc: play_count tracks total calls, not active sounds
#[test]
fn when_null_play_called_then_play_count_matches() {
    // Arrange
    let mut backend = NullAudioBackend::new();
    let sound = minimal_sound();

    // Act
    backend.play_on_track(&sound, MixerTrack::Sfx);
    backend.play_on_track(&sound, MixerTrack::Sfx);

    // Assert
    assert_eq!(
        backend.play_count(),
        2,
        "play_count should equal number of play_on_track calls"
    );
}

/// @doc: stop is a no-op on null backend (idempotent, does not crash)
#[test]
fn when_null_stop_called_then_does_not_panic() {
    // Arrange
    let mut backend = NullAudioBackend::new();
    let sound = minimal_sound();
    let id = backend.play_on_track(&sound, MixerTrack::Sfx);

    // Act — stop existing and non-existent IDs
    backend.stop(id);
    backend.stop(engine_audio::playback::PlaybackId(999));

    // Assert — play_count unchanged by stop (null backend doesn't track active set)
    assert_eq!(
        backend.play_count(),
        1,
        "stop on null backend should not affect play count"
    );
}

/// @doc: set_volume is a no-op on null backend
#[test]
fn when_null_set_volume_called_then_does_not_panic() {
    // Arrange
    let mut backend = NullAudioBackend::new();

    // Act — cover valid range and boundary values
    backend.set_volume(0.0);
    backend.set_volume(0.5);
    backend.set_volume(1.0);
    backend.set_volume(2.0); // above 1.0 — valid per spec

    // Assert — no crash is the contract; play_count unchanged
    assert_eq!(
        backend.play_count(),
        0,
        "set_volume on null backend should not affect play count"
    );
}

/// @doc: set_track_volume is a no-op on null backend
#[test]
fn when_null_set_track_volume_called_then_does_not_panic() {
    // Arrange
    let mut backend = NullAudioBackend::new();

    // Act — cover all track types and boundary values
    backend.set_track_volume(MixerTrack::Music, 0.0);
    backend.set_track_volume(MixerTrack::Sfx, 0.5);
    backend.set_track_volume(MixerTrack::Ambient, 1.0);
    backend.set_track_volume(MixerTrack::Ui, 2.0);

    // Assert
    assert_eq!(
        backend.play_count(),
        0,
        "set_track_volume on null backend should not affect play count"
    );
}

/// @doc: play after multiple previous plays still produces monotonic IDs
#[test]
fn when_null_play_stop_play_then_id_continues_monotonic() {
    // Arrange
    let mut backend = NullAudioBackend::new();
    let sound = minimal_sound();

    // Act
    let id1 = backend.play_on_track(&sound, MixerTrack::Sfx);
    backend.stop(id1);
    let id2 = backend.play_on_track(&sound, MixerTrack::Sfx);

    // Assert — stop doesn't reset IDs (null backend stateless about active set)
    assert_eq!(id2.0, 2, "IDs should continue monotonic after stop; got id2={}", id2.0);
}

/// @doc: empty sound (zero samples) is accepted by null backend
#[test]
fn when_null_play_empty_sound_then_does_not_panic() {
    // Arrange
    let mut backend = NullAudioBackend::new();
    let empty = SoundData {
        samples: vec![],
        sample_rate: 44_100,
        channels: 1,
    };

    // Act
    let id = backend.play_on_track(&empty, MixerTrack::Sfx);

    // Assert
    assert_eq!(id.0, 1, "empty sound should still get a valid PlaybackId");
    assert_eq!(backend.play_count(), 1, "empty sound should count as a play");
}

/// @doc: stereo sound (2 channels) is accepted by null backend
#[test]
fn when_null_play_stereo_sound_then_does_not_panic() {
    // Arrange
    let mut backend = NullAudioBackend::new();
    let stereo = SoundData {
        samples: silent_samples(88_200), // 1 second of stereo at 44100
        sample_rate: 44_100,
        channels: 2,
    };

    // Act
    let id = backend.play_on_track(&stereo, MixerTrack::Music);

    // Assert
    assert_eq!(id.0, 1, "stereo sound should get a valid PlaybackId");
}

/// @doc: each MixerTrack variant routes independently (null backend stores track per sound)
#[test]
fn when_null_play_different_tracks_then_id_sequence_continues() {
    // Arrange
    let mut backend = NullAudioBackend::new();
    let sound = minimal_sound();

    // Act
    let id1 = backend.play_on_track(&sound, MixerTrack::Music);
    let id2 = backend.play_on_track(&sound, MixerTrack::Sfx);
    let id3 = backend.play_on_track(&sound, MixerTrack::Ambient);
    let id4 = backend.play_on_track(&sound, MixerTrack::Ui);

    // Assert — all four track types accepted, IDs monotonic
    assert_eq!(
        (id1.0, id2.0, id3.0, id4.0),
        (1, 2, 3, 4),
        "all MixerTrack variants should get sequential monotonic IDs"
    );
}
