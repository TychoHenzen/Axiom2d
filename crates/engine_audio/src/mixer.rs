use bevy_ecs::resource::Resource;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    fn when_default_mixer_state_then_all_tracks_are_one() {
        // Arrange
        let state = MixerState::default();

        // Assert
        for track in MixerTrack::ALL {
            assert!(
                (state.track_volume(track) - 1.0).abs() < f32::EPSILON,
                "{track:?} should default to 1.0"
            );
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
