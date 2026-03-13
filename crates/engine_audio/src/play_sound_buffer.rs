use std::vec::Drain;

use bevy_ecs::entity::Entity;
use bevy_ecs::resource::Resource;

use crate::mixer::MixerTrack;
use crate::spatial::SpatialGains;

#[derive(Debug, Clone)]
pub struct PlaySound {
    pub name: String,
    pub track: MixerTrack,
    pub emitter: Option<Entity>,
    pub spatial_gains: Option<SpatialGains>,
}

impl PlaySound {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            track: MixerTrack::Sfx,
            emitter: None,
            spatial_gains: None,
        }
    }

    pub fn on_track(name: impl Into<String>, track: MixerTrack) -> Self {
        Self {
            name: name.into(),
            track,
            emitter: None,
            spatial_gains: None,
        }
    }

    pub fn at_emitter(name: impl Into<String>, emitter: Entity) -> Self {
        Self {
            name: name.into(),
            track: MixerTrack::Sfx,
            emitter: Some(emitter),
            spatial_gains: None,
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct PlaySoundBuffer {
    commands: Vec<PlaySound>,
}

impl PlaySoundBuffer {
    pub fn push(&mut self, cmd: PlaySound) {
        self.commands.push(cmd);
    }

    pub fn drain(&mut self) -> Drain<'_, PlaySound> {
        self.commands.drain(..)
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, PlaySound> {
        self.commands.iter_mut()
    }
}

impl<'a> IntoIterator for &'a mut PlaySoundBuffer {
    type Item = &'a mut PlaySound;
    type IntoIter = std::slice::IterMut<'a, PlaySound>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    use crate::mixer::MixerTrack;

    #[test]
    fn when_play_sound_new_then_track_defaults_to_sfx() {
        // Act
        let cmd = PlaySound::new("beep");

        // Assert
        assert_eq!(cmd.track, MixerTrack::Sfx);
    }

    #[test]
    fn when_play_sound_on_track_then_track_is_preserved() {
        // Act
        let cmd = PlaySound::on_track("bgm", MixerTrack::Music);

        // Assert
        assert_eq!(cmd.track, MixerTrack::Music);
    }

    #[test]
    fn when_push_and_drain_then_returns_one_command() {
        // Arrange
        let mut buffer = PlaySoundBuffer::default();
        buffer.push(PlaySound::new("beep"));

        // Act
        let commands: Vec<_> = buffer.drain().collect();

        // Assert
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].name, "beep");
    }

    #[test]
    fn when_drained_then_buffer_is_empty() {
        // Arrange
        let mut buffer = PlaySoundBuffer::default();
        buffer.push(PlaySound::new("a"));
        buffer.push(PlaySound::new("b"));

        // Act
        let _ = buffer.drain().count();
        let remaining: Vec<_> = buffer.drain().collect();

        // Assert
        assert!(remaining.is_empty());
    }
}
