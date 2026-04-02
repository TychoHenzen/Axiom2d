#![allow(clippy::unwrap_used)]

use engine_audio::mixer::{MixerState, MixerTrack};

#[test]
fn when_mixer_track_variants_serialized_to_ron_then_each_deserializes_to_matching_variant() {
    for track in MixerTrack::ALL {
        let ron = ron::to_string(&track).unwrap();
        let back: MixerTrack = ron::from_str(&ron).unwrap();
        assert_eq!(track, back);
    }
}

#[test]
fn when_set_track_volume_then_only_that_track_changes() {
    // Arrange
    let mut state = MixerState::default();

    // Act
    state.set_track_volume(MixerTrack::Music, 0.3);

    // Assert
    assert!((state.track_volume(MixerTrack::Music) - 0.3).abs() < f32::EPSILON);
    assert!((state.track_volume(MixerTrack::Sfx) - 1.0).abs() < f32::EPSILON);
    assert!((state.track_volume(MixerTrack::Ambient) - 1.0).abs() < f32::EPSILON);
}

#[test]
fn when_volume_above_one_then_stored_unchanged() {
    // Arrange
    let mut state = MixerState::default();

    // Act
    state.set_track_volume(MixerTrack::Music, 2.0);

    // Assert
    assert!((state.track_volume(MixerTrack::Music) - 2.0).abs() < f32::EPSILON);
}
