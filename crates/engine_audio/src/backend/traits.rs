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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_helpers::minimal_sound;

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
}
