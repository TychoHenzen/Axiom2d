use bevy_ecs::prelude::{Res, ResMut};

use crate::audio_res::AudioRes;
use crate::play_sound_buffer::PlaySoundBuffer;
use crate::sound_library::SoundLibrary;

const DEFAULT_SAMPLE_RATE: u32 = 44_100;
const DEFAULT_DURATION: f32 = 0.5;

pub fn play_sound_system(
    mut buffer: ResMut<PlaySoundBuffer>,
    library: Option<Res<SoundLibrary>>,
    mut audio: ResMut<AudioRes>,
) {
    let commands: Vec<_> = buffer.drain().collect();

    let Some(library) = library else {
        return;
    };

    for cmd in commands {
        if let Some(effect) = library.get(&cmd.name) {
            let sound = effect.synthesize(DEFAULT_SAMPLE_RATE, DEFAULT_DURATION);
            audio.play(&sound);
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::{Arc, Mutex};

    use bevy_ecs::prelude::{Schedule, World};

    use crate::audio_backend::AudioBackend;
    use crate::play_sound_buffer::PlaySound;
    use crate::playback_id::PlaybackId;
    use crate::sound_data::SoundData;
    use crate::sound_effect::SoundEffect;

    use super::*;

    struct SpyAudioBackend {
        play_count: Arc<Mutex<u32>>,
        next_id: u32,
    }

    impl SpyAudioBackend {
        fn new(play_count: Arc<Mutex<u32>>) -> Self {
            Self {
                play_count,
                next_id: 0,
            }
        }
    }

    impl AudioBackend for SpyAudioBackend {
        fn play(&mut self, _sound: &SoundData) -> PlaybackId {
            self.next_id += 1;
            *self.play_count.lock().unwrap() += 1;
            PlaybackId(self.next_id)
        }

        fn stop(&mut self, _id: PlaybackId) {}
        fn set_volume(&mut self, _volume: f32) {}
    }

    fn test_effect() -> SoundEffect {
        use fundsp::hacker32::*;
        use fundsp::prelude::AudioUnit;
        SoundEffect::new(|| Box::new(dc(0.5)) as Box<dyn AudioUnit>)
    }

    fn setup_world(play_count: &Arc<Mutex<u32>>) -> World {
        let mut world = World::new();
        world.insert_resource(PlaySoundBuffer::default());
        world.insert_resource(AudioRes::new(Box::new(SpyAudioBackend::new(Arc::clone(
            play_count,
        )))));
        world
    }

    fn run_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(play_sound_system);
        schedule.run(world);
    }

    #[test]
    fn when_known_sound_then_audio_play_is_called() {
        // Arrange
        let play_count = Arc::new(Mutex::new(0u32));
        let mut world = setup_world(&play_count);
        let mut library = SoundLibrary::default();
        library.register("beep", test_effect());
        world.insert_resource(library);
        world
            .resource_mut::<PlaySoundBuffer>()
            .push(PlaySound::new("beep"));

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(*play_count.lock().unwrap(), 1);
    }

    #[test]
    fn when_known_sound_then_buffer_is_drained() {
        // Arrange
        let play_count = Arc::new(Mutex::new(0u32));
        let mut world = setup_world(&play_count);
        let mut library = SoundLibrary::default();
        library.register("beep", test_effect());
        world.insert_resource(library);
        world
            .resource_mut::<PlaySoundBuffer>()
            .push(PlaySound::new("beep"));

        // Act
        run_system(&mut world);

        // Assert
        let remaining: Vec<_> = world.resource_mut::<PlaySoundBuffer>().drain().collect();
        assert!(remaining.is_empty());
    }

    #[test]
    fn when_no_sound_library_then_audio_not_called() {
        // Arrange
        let play_count = Arc::new(Mutex::new(0u32));
        let mut world = setup_world(&play_count);
        // No SoundLibrary inserted
        world
            .resource_mut::<PlaySoundBuffer>()
            .push(PlaySound::new("beep"));

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(*play_count.lock().unwrap(), 0);
    }

    #[test]
    fn when_unknown_sound_name_then_audio_not_called() {
        // Arrange
        let play_count = Arc::new(Mutex::new(0u32));
        let mut world = setup_world(&play_count);
        let mut library = SoundLibrary::default();
        library.register("beep", test_effect());
        world.insert_resource(library);
        world
            .resource_mut::<PlaySoundBuffer>()
            .push(PlaySound::new("missing"));

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(*play_count.lock().unwrap(), 0);
    }

    #[test]
    fn when_unknown_sound_name_then_buffer_is_still_drained() {
        // Arrange
        let play_count = Arc::new(Mutex::new(0u32));
        let mut world = setup_world(&play_count);
        let mut library = SoundLibrary::default();
        library.register("beep", test_effect());
        world.insert_resource(library);
        world
            .resource_mut::<PlaySoundBuffer>()
            .push(PlaySound::new("missing"));

        // Act
        run_system(&mut world);

        // Assert
        let remaining: Vec<_> = world.resource_mut::<PlaySoundBuffer>().drain().collect();
        assert!(remaining.is_empty());
    }
}
