use crate::mixer::MixerTrack;
use crate::playback::PlaybackId;
use crate::sound::SoundData;

pub trait AudioBackend: Send + Sync {
    fn play_on_track(&mut self, sound: &SoundData, track: MixerTrack) -> PlaybackId;
    fn stop(&mut self, id: PlaybackId);
    fn set_volume(&mut self, volume: f32);
    fn set_track_volume(&mut self, track: MixerTrack, volume: f32);
}

pub struct NullAudioBackend {
    next_id: u32,
}

impl NullAudioBackend {
    #[must_use]
    pub fn new() -> Self {
        Self { next_id: 0 }
    }

    #[must_use]
    pub fn play_count(&self) -> u32 {
        self.next_id
    }
}

impl Default for NullAudioBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioBackend for NullAudioBackend {
    fn play_on_track(&mut self, _sound: &SoundData, _track: MixerTrack) -> PlaybackId {
        self.next_id += 1;
        PlaybackId(self.next_id)
    }

    fn stop(&mut self, _id: PlaybackId) {}

    fn set_volume(&mut self, _volume: f32) {}

    fn set_track_volume(&mut self, _track: MixerTrack, _volume: f32) {}
}
