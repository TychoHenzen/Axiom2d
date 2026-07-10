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

/// @doc: Verifies that `NullAudioBackend.play_on_track` increments play count for SFX track.
#[test]
fn when_play_on_track_with_sfx_then_play_count_increments() {
    // Arrange
    let mut backend = NullAudioBackend::new();
    let sound = minimal_sound();

    // Act
    backend.play_on_track(&sound, MixerTrack::Sfx);

    // Assert
    assert_eq!(
        backend.play_count(),
        1,
        "play count should increment to 1 after one SFX play"
    );
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
    assert_ne!(
        id1, id2,
        "each play_on_track call should return a unique ID"
    );
}

/// @doc: Verifies that `NullAudioBackend.play_on_track` increments play count for Music track.
#[test]
fn when_play_on_track_called_then_play_count_increments() {
    // Arrange
    let mut backend = NullAudioBackend::new();
    let sound = minimal_sound();

    // Act
    backend.play_on_track(&sound, MixerTrack::Music);

    // Assert
    assert_eq!(
        backend.play_count(),
        1,
        "play count should increment to 1 after one Music track play"
    );
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
    assert_eq!(
        backend.play_count(),
        3,
        "play count should accumulate to 3 after three plays"
    );
}

/// @doc: Playing a zero-sample sound still increments play count — backend doesn't validate sample data.
#[test]
fn when_empty_sound_played_then_play_count_increments() {
    // Arrange
    let mut backend = NullAudioBackend::new();
    let empty = SoundData {
        samples: vec![],
        sample_rate: 44_100,
        channels: 1,
    };

    // Act
    backend.play_on_track(&empty, MixerTrack::Sfx);

    // Assert
    assert_eq!(
        backend.play_count(),
        1,
        "play count should increment even with empty SoundData"
    );
}

/// @doc: Stopping an unregistered `PlaybackId` is a no-op and must not panic.
#[test]
fn when_stop_unknown_id_then_no_panic() {
    // Arrange
    let mut backend = NullAudioBackend::new();

    // Act
    backend.stop(PlaybackId(999));

    // Assert — reached without panic
    assert_eq!(
        backend.play_count(),
        0,
        "play count unchanged after stopping unknown ID"
    );
}

/// @doc: All `MixerTrack` variants (Master, Music, Sfx, Ambient, Ui) accept playback.
#[test]
fn when_play_on_all_tracks_then_play_count_accumulates() {
    // Arrange
    let mut backend = NullAudioBackend::new();
    let sound = minimal_sound();

    // Act
    for track in &MixerTrack::ALL {
        backend.play_on_track(&sound, *track);
    }

    // Assert
    assert_eq!(
        backend.play_count(),
        5,
        "play count should be 5 after one play on each of 5 tracks"
    );
}

/// @doc: A freshly created `NullAudioBackend` has a play count of zero.
#[test]
fn when_backend_new_then_play_count_zero() {
    // Arrange / Act
    let backend = NullAudioBackend::new();

    // Assert
    assert_eq!(
        backend.play_count(),
        0,
        "new backend should have zero play count"
    );
}

/// @doc: Calling `set_volume` on `NullAudioBackend` is a no-op and does not alter play count.
#[test]
fn when_set_volume_then_play_count_unchanged() {
    // Arrange
    let mut backend = NullAudioBackend::new();
    let sound = minimal_sound();
    backend.play_on_track(&sound, MixerTrack::Sfx);

    // Act
    backend.set_volume(0.5);

    // Assert
    assert_eq!(
        backend.play_count(),
        1,
        "set_volume should not change play count"
    );
}

/// @doc: Calling `set_track_volume` on `NullAudioBackend` is a no-op and does not alter play count.
#[test]
fn when_set_track_volume_then_play_count_unchanged() {
    // Arrange
    let mut backend = NullAudioBackend::new();
    let sound = minimal_sound();
    backend.play_on_track(&sound, MixerTrack::Sfx);

    // Act
    backend.set_track_volume(MixerTrack::Music, 0.3);

    // Assert
    assert_eq!(
        backend.play_count(),
        1,
        "set_track_volume should not change play count"
    );
}
