use bevy_ecs::prelude::{Res, ResMut};

use super::buffer::PlaySoundBuffer;
use crate::audio_res::AudioRes;
use crate::mixer::MixerState;
use crate::sound::SoundData;
use crate::sound::SoundLibrary;
use crate::spatial::SpatialGains;

const DEFAULT_SAMPLE_RATE: u32 = 44_100;
const DEFAULT_DURATION: f32 = 0.5;

pub fn play_sound_system(
    mut buffer: ResMut<PlaySoundBuffer>,
    library: Option<Res<SoundLibrary>>,
    mixer: Option<Res<MixerState>>,
    mut audio: ResMut<AudioRes>,
) {
    let Some(library) = library else {
        // Drain buffer even when no library to avoid unbounded growth.
        buffer.drain().for_each(drop);
        return;
    };

    if let Some(mixer) = &mixer {
        for track in crate::mixer::MixerTrack::ALL {
            audio.set_track_volume(track, mixer.track_volume(track));
        }
    }

    for cmd in buffer.drain() {
        if let Some(effect) = library.get(&cmd.name) {
            let sound = effect.synthesize(DEFAULT_SAMPLE_RATE, DEFAULT_DURATION);
            let sound = if let Some(gains) = cmd.spatial_gains {
                if gains.left.abs() < f32::EPSILON && gains.right.abs() < f32::EPSILON {
                    continue;
                }
                apply_spatial_gains(&sound, gains)
            } else {
                sound
            };
            audio.play_on_track(&sound, cmd.track);
        }
    }
}

fn apply_spatial_gains(sound: &SoundData, gains: SpatialGains) -> SoundData {
    let frame_count = sound.frame_count();
    let mut stereo_samples = Vec::with_capacity(frame_count * 2);

    if sound.channels == 1 {
        for &sample in &sound.samples {
            stereo_samples.push(sample * gains.left);
            stereo_samples.push(sample * gains.right);
        }
    } else {
        for frame in sound.samples.chunks(sound.channels as usize) {
            let left = frame.first().copied().unwrap_or(0.0);
            let right = frame.get(1).copied().unwrap_or(0.0);
            stereo_samples.push(left * gains.left);
            stereo_samples.push(right * gains.right);
        }
    }

    SoundData {
        samples: stereo_samples,
        sample_rate: sound.sample_rate,
        channels: 2,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::sync::{Arc, Mutex};

    use bevy_ecs::prelude::{Schedule, World};

    use crate::backend::AudioBackend;
    use crate::mixer::{MixerState, MixerTrack};
    use crate::playback::{PlaySound, PlaybackId};
    use crate::sound::{SoundData, SoundEffect};

    use super::*;

    struct SpyAudioBackend {
        play_count: Arc<Mutex<u32>>,
        played_tracks: Arc<Mutex<Vec<MixerTrack>>>,
        track_volume_calls: Arc<Mutex<Vec<(MixerTrack, f32)>>>,
        next_id: u32,
    }

    impl SpyAudioBackend {
        fn new(play_count: Arc<Mutex<u32>>) -> Self {
            Self {
                play_count,
                played_tracks: Arc::new(Mutex::new(Vec::new())),
                track_volume_calls: Arc::new(Mutex::new(Vec::new())),
                next_id: 0,
            }
        }

        fn with_track_captures(
            play_count: Arc<Mutex<u32>>,
            played_tracks: Arc<Mutex<Vec<MixerTrack>>>,
            track_volume_calls: Arc<Mutex<Vec<(MixerTrack, f32)>>>,
        ) -> Self {
            Self {
                play_count,
                played_tracks,
                track_volume_calls,
                next_id: 0,
            }
        }
    }

    impl AudioBackend for SpyAudioBackend {
        fn play_on_track(&mut self, _sound: &SoundData, track: MixerTrack) -> PlaybackId {
            self.next_id += 1;
            *self.play_count.lock().unwrap() += 1;
            self.played_tracks.lock().unwrap().push(track);
            PlaybackId(self.next_id)
        }

        fn stop(&mut self, _id: PlaybackId) {}
        fn set_volume(&mut self, _volume: f32) {}
        fn set_track_volume(&mut self, track: MixerTrack, volume: f32) {
            self.track_volume_calls
                .lock()
                .unwrap()
                .push((track, volume));
        }
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
    fn when_play_sound_with_mixer_state_then_track_volume_forwarded() {
        // Arrange
        let play_count = Arc::new(Mutex::new(0u32));
        let played_tracks = Arc::new(Mutex::new(Vec::new()));
        let track_volume_calls = Arc::new(Mutex::new(Vec::new()));
        let mut world = World::new();
        world.insert_resource(PlaySoundBuffer::default());
        world.insert_resource(AudioRes::new(Box::new(
            SpyAudioBackend::with_track_captures(
                Arc::clone(&play_count),
                Arc::clone(&played_tracks),
                Arc::clone(&track_volume_calls),
            ),
        )));
        let mut library = SoundLibrary::default();
        library.register("bgm", test_effect());
        world.insert_resource(library);
        let mut mixer = MixerState::default();
        mixer.set_track_volume(MixerTrack::Music, 0.3);
        world.insert_resource(mixer);
        world
            .resource_mut::<PlaySoundBuffer>()
            .push(PlaySound::on_track("bgm", MixerTrack::Music));

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(*play_count.lock().unwrap(), 1);
        assert_eq!(played_tracks.lock().unwrap()[0], MixerTrack::Music);
        let calls = track_volume_calls.lock().unwrap();
        let music_call = calls.iter().find(|(t, _)| *t == MixerTrack::Music);
        assert!(music_call.is_some());
        assert!((music_call.unwrap().1 - 0.3).abs() < f32::EPSILON);
    }

    #[test]
    fn when_play_sound_without_mixer_state_then_runs_normally() {
        // Arrange
        let play_count = Arc::new(Mutex::new(0u32));
        let mut world = setup_world(&play_count);
        let mut library = SoundLibrary::default();
        library.register("beep", test_effect());
        world.insert_resource(library);
        // No MixerState inserted
        world
            .resource_mut::<PlaySoundBuffer>()
            .push(PlaySound::new("beep"));

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(*play_count.lock().unwrap(), 1);
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

    /// @doc: Pre-computed spatial gains bypass attenuation — allows manual gain control or preview mode
    #[test]
    fn when_spatial_gains_present_then_play_sound_applies_them() {
        // Arrange
        let play_count = Arc::new(Mutex::new(0u32));
        let mut world = setup_world(&play_count);
        let mut library = SoundLibrary::default();
        library.register("beep", test_effect());
        world.insert_resource(library);
        let mut cmd = PlaySound::new("beep");
        cmd.spatial_gains = Some(crate::spatial::SpatialGains {
            left: 0.3,
            right: 0.9,
        });
        world.resource_mut::<PlaySoundBuffer>().push(cmd);

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(*play_count.lock().unwrap(), 1);
    }

    /// @doc: Mono-to-stereo conversion applies spatial gains separately — mono upmix with panning applied
    #[test]
    fn when_apply_spatial_gains_mono_then_stereo_samples_scaled() {
        // Arrange
        let mono = SoundData {
            samples: vec![1.0, 0.5, -0.5],
            sample_rate: 44_100,
            channels: 1,
        };
        let gains = crate::spatial::SpatialGains {
            left: 0.3,
            right: 0.7,
        };

        // Act
        let result = super::apply_spatial_gains(&mono, gains);

        // Assert — 3 mono frames → 6 stereo samples
        assert_eq!(result.channels, 2);
        assert_eq!(result.samples.len(), 6);
        assert!((result.samples[0] - 1.0 * 0.3).abs() < 1e-6); // frame 0 left
        assert!((result.samples[1] - 1.0 * 0.7).abs() < 1e-6); // frame 0 right
        assert!((result.samples[2] - 0.5 * 0.3).abs() < 1e-6); // frame 1 left
        assert!((result.samples[3] - 0.5 * 0.7).abs() < 1e-6); // frame 1 right
    }

    /// @doc: Stereo channels are scaled independently — preserves spatial direction after gain adjustment
    #[test]
    fn when_apply_spatial_gains_stereo_then_channels_scaled_independently() {
        // Arrange
        let stereo = SoundData {
            samples: vec![1.0, 0.8, 0.6, 0.4],
            sample_rate: 44_100,
            channels: 2,
        };
        let gains = crate::spatial::SpatialGains {
            left: 0.5,
            right: 0.25,
        };

        // Act
        let result = super::apply_spatial_gains(&stereo, gains);

        // Assert — 2 stereo frames → 4 samples
        assert_eq!(result.channels, 2);
        assert_eq!(result.samples.len(), 4);
        assert!((result.samples[0] - 1.0 * 0.5).abs() < 1e-6); // frame 0 left
        assert!((result.samples[1] - 0.8 * 0.25).abs() < 1e-6); // frame 0 right
        assert!((result.samples[2] - 0.6 * 0.5).abs() < 1e-6); // frame 1 left
        assert!((result.samples[3] - 0.4 * 0.25).abs() < 1e-6); // frame 1 right
    }

    /// @doc: Single-channel silence does not cull — only both gains zero triggers skip (asymmetric panning valid)
    #[test]
    fn when_one_gain_zero_other_nonzero_then_sound_still_plays() {
        // Arrange — only left is zero, right is nonzero → should NOT be culled
        let play_count = Arc::new(Mutex::new(0u32));
        let mut world = setup_world(&play_count);
        let mut library = SoundLibrary::default();
        library.register("beep", test_effect());
        world.insert_resource(library);
        let mut cmd = PlaySound::new("beep");
        cmd.spatial_gains = Some(crate::spatial::SpatialGains {
            left: 0.0,
            right: 0.5,
        });
        world.resource_mut::<PlaySoundBuffer>().push(cmd);

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(
            *play_count.lock().unwrap(),
            1,
            "should play when one channel is nonzero"
        );
    }

    /// @doc: Gain below epsilon threshold culls the sound entirely — prevents wasted mixing on inaudible sources
    #[test]
    fn when_both_gains_zero_then_play_sound_skips_backend() {
        // Arrange
        let play_count = Arc::new(Mutex::new(0u32));
        let mut world = setup_world(&play_count);
        let mut library = SoundLibrary::default();
        library.register("beep", test_effect());
        world.insert_resource(library);
        let mut cmd = PlaySound::new("beep");
        cmd.spatial_gains = Some(crate::spatial::SpatialGains {
            left: 0.0,
            right: 0.0,
        });
        world.resource_mut::<PlaySoundBuffer>().push(cmd);

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(*play_count.lock().unwrap(), 0);
    }

    /// @doc: Epsilon boundary is exclusive — gain exactly EPSILON plays, gain < EPSILON culls (precision edge case)
    #[test]
    fn when_right_gain_exactly_epsilon_and_left_zero_then_sound_not_culled() {
        // Arrange — right=EPSILON (not < EPSILON), left=0: culling condition is false → plays
        let play_count = Arc::new(Mutex::new(0u32));
        let mut world = setup_world(&play_count);
        let mut library = SoundLibrary::default();
        library.register("beep", test_effect());
        world.insert_resource(library);
        let mut cmd = PlaySound::new("beep");
        cmd.spatial_gains = Some(crate::spatial::SpatialGains {
            left: 0.0,
            right: f32::EPSILON,
        });
        world.resource_mut::<PlaySoundBuffer>().push(cmd);

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(
            *play_count.lock().unwrap(),
            1,
            "right=EPSILON should not be culled"
        );
    }

    /// @doc: Epsilon boundary is exclusive — prevents culling sounds with minimal but audible gain
    #[test]
    fn when_gain_exactly_epsilon_then_sound_not_culled() {
        // Arrange — left = EPSILON (not < EPSILON), right = 0.0 → should NOT be culled
        let play_count = Arc::new(Mutex::new(0u32));
        let mut world = setup_world(&play_count);
        let mut library = SoundLibrary::default();
        library.register("beep", test_effect());
        world.insert_resource(library);
        let mut cmd = PlaySound::new("beep");
        cmd.spatial_gains = Some(crate::spatial::SpatialGains {
            left: f32::EPSILON,
            right: 0.0,
        });
        world.resource_mut::<PlaySoundBuffer>().push(cmd);

        // Act
        run_system(&mut world);

        // Assert
        assert_eq!(
            *play_count.lock().unwrap(),
            1,
            "gain of exactly EPSILON should not be culled"
        );
    }
}
