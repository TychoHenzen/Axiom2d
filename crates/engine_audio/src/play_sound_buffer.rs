use std::vec::Drain;

use bevy_ecs::resource::Resource;

#[derive(Debug, Clone)]
pub struct PlaySound {
    pub name: String,
}

impl PlaySound {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
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
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

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
