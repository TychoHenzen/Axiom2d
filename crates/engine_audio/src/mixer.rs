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
