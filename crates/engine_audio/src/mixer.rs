use bevy_ecs::resource::Resource;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MixerTrack {
    Master,
    Music,
    Sfx,
    Ambient,
    Ui,
}

impl MixerTrack {
    pub const ALL: [Self; 5] = [
        Self::Master,
        Self::Music,
        Self::Sfx,
        Self::Ambient,
        Self::Ui,
    ];

    #[must_use]
    pub fn index(self) -> usize {
        self as usize
    }
}

pub const TRACK_COUNT: usize = MixerTrack::ALL.len();

#[derive(Resource, Debug, Clone)]
pub struct MixerState {
    volumes: [f32; TRACK_COUNT],
}

impl Default for MixerState {
    fn default() -> Self {
        Self {
            volumes: [1.0; TRACK_COUNT],
        }
    }
}

impl MixerState {
    #[must_use]
    pub fn track_volume(&self, track: MixerTrack) -> f32 {
        self.volumes[track.index()]
    }

    pub fn set_track_volume(&mut self, track: MixerTrack, volume: f32) {
        self.volumes[track.index()] = volume;
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

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
}
