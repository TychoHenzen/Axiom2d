#![allow(clippy::unwrap_used)]

use std::sync::{Arc, Mutex};

use bevy_ecs::prelude::{Schedule, World};

use engine_audio::audio_res::AudioRes;
use engine_audio::backend::AudioBackend;
use engine_audio::mixer::{MixerState, MixerTrack};
use engine_audio::playback::{PlaySound, PlaybackId, SpatialGains, play_sound_system};
use engine_audio::sound::{SoundData, SoundEffect, SoundLibrary};
use engine_core::prelude::EventBus;

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
    use fundsp::prelude::AudioUnit;
    use fundsp::prelude32::*;
    SoundEffect::new(|| Box::new(dc(0.5)) as Box<dyn AudioUnit>)
}

fn setup_world(play_count: &Arc<Mutex<u32>>) -> World {
    let mut world = World::new();
    world.insert_resource(EventBus::<PlaySound>::default());
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
        .resource_mut::<EventBus<PlaySound>>()
        .push(PlaySound::new("beep"));

    // Act
    run_system(&mut world);

    // Assert
    assert_eq!(*play_count.lock().unwrap(), 1);
}

#[test]
fn when_known_sound_then_bus_is_drained() {
    // Arrange
    let play_count = Arc::new(Mutex::new(0u32));
    let mut world = setup_world(&play_count);
    let mut library = SoundLibrary::default();
    library.register("beep", test_effect());
    world.insert_resource(library);
    world
        .resource_mut::<EventBus<PlaySound>>()
        .push(PlaySound::new("beep"));

    // Act
    run_system(&mut world);

    // Assert
    let remaining: Vec<_> = world
        .resource_mut::<EventBus<PlaySound>>()
        .drain()
        .collect();
    assert!(remaining.is_empty());
}

#[test]
fn when_no_sound_library_then_audio_not_called() {
    // Arrange
    let play_count = Arc::new(Mutex::new(0u32));
    let mut world = setup_world(&play_count);
    // No SoundLibrary inserted
    world
        .resource_mut::<EventBus<PlaySound>>()
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
        .resource_mut::<EventBus<PlaySound>>()
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
    world.insert_resource(EventBus::<PlaySound>::default());
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
        .resource_mut::<EventBus<PlaySound>>()
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
        .resource_mut::<EventBus<PlaySound>>()
        .push(PlaySound::new("beep"));

    // Act
    run_system(&mut world);

    // Assert
    assert_eq!(*play_count.lock().unwrap(), 1);
}

#[test]
fn when_unknown_sound_name_then_bus_is_still_drained() {
    // Arrange
    let play_count = Arc::new(Mutex::new(0u32));
    let mut world = setup_world(&play_count);
    let mut library = SoundLibrary::default();
    library.register("beep", test_effect());
    world.insert_resource(library);
    world
        .resource_mut::<EventBus<PlaySound>>()
        .push(PlaySound::new("missing"));

    // Act
    run_system(&mut world);

    // Assert
    let remaining: Vec<_> = world
        .resource_mut::<EventBus<PlaySound>>()
        .drain()
        .collect();
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
    cmd.spatial_gains = Some(SpatialGains {
        left: 0.3,
        right: 0.9,
    });
    world.resource_mut::<EventBus<PlaySound>>().push(cmd);

    // Act
    run_system(&mut world);

    // Assert
    assert_eq!(*play_count.lock().unwrap(), 1);
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
    cmd.spatial_gains = Some(SpatialGains {
        left: 0.0,
        right: 0.5,
    });
    world.resource_mut::<EventBus<PlaySound>>().push(cmd);

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
    cmd.spatial_gains = Some(SpatialGains {
        left: 0.0,
        right: 0.0,
    });
    world.resource_mut::<EventBus<PlaySound>>().push(cmd);

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
    cmd.spatial_gains = Some(SpatialGains {
        left: 0.0,
        right: f32::EPSILON,
    });
    world.resource_mut::<EventBus<PlaySound>>().push(cmd);

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
    cmd.spatial_gains = Some(SpatialGains {
        left: f32::EPSILON,
        right: 0.0,
    });
    world.resource_mut::<EventBus<PlaySound>>().push(cmd);

    // Act
    run_system(&mut world);

    // Assert
    assert_eq!(
        *play_count.lock().unwrap(),
        1,
        "gain of exactly EPSILON should not be culled"
    );
}
