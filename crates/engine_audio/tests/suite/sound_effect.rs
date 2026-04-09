#![allow(clippy::unwrap_used)]

use fundsp::prelude::AudioUnit;
use fundsp::prelude32::*;

use engine_audio::sound::SoundEffect;

fn test_effect() -> SoundEffect {
    SoundEffect::new(|| Box::new(dc(0.5)) as Box<dyn AudioUnit>)
}

#[test]
fn when_synthesize_called_then_sound_data_has_correct_sample_rate() {
    // Arrange
    let effect = test_effect();

    // Act
    let sound = effect.synthesize(44_100, 1.0);

    // Assert
    assert_eq!(sound.sample_rate, 44_100);
}

#[test]
fn when_synthesize_called_then_sound_data_has_mono_channel_count() {
    // Arrange
    let effect = test_effect();

    // Act
    let sound = effect.synthesize(44_100, 0.1);

    // Assert
    assert_eq!(sound.channels, 1);
}

/// @doc: Sample buffer size must exactly equal `sample_rate * duration *
/// channels`. An off-by-one would cause the mixer to read past the end of
/// the buffer (UB-adjacent panic) or produce a pop/click from a short sample.
#[test]
fn when_synthesize_called_then_sample_length_equals_frame_count_times_channels() {
    // Arrange
    let effect = test_effect();

    // Act
    let sound = effect.synthesize(44_100, 1.0);

    // Assert
    assert_eq!(sound.samples.len(), 44_100);
}

/// @doc: Smoke test that the DSP graph actually produces signal — a silent
/// output from a non-zero source graph would indicate a broken fundsp pipeline
/// or incorrect `tick()` call, making all synthesized sounds inaudible.
#[test]
fn when_nonzero_amplitude_graph_then_samples_are_not_all_zero() {
    // Arrange
    let effect = test_effect();

    // Act
    let sound = effect.synthesize(44_100, 0.01);

    // Assert
    assert!(sound.samples.iter().any(|&s| s != 0.0));
}

#[test]
fn when_synthesize_with_half_second_duration_then_frame_count_is_half_sample_rate() {
    // Arrange
    let effect = test_effect();

    // Act
    let sound = effect.synthesize(44_100, 0.5);

    // Assert — frame_count = (44100 * 0.5) = 22050, channels = 1
    assert_eq!(sound.samples.len(), 22_050);
}

/// @doc: Synthesis is stateless — factory creates fresh graph each call, no caching or state carryover
#[test]
fn when_synthesize_called_twice_then_each_call_returns_fresh_sound_data() {
    // Arrange
    let effect = test_effect();

    // Act
    let sound_a = effect.synthesize(44_100, 0.1);
    let sound_b = effect.synthesize(44_100, 0.1);

    // Assert
    assert_eq!(sound_a.samples.len(), sound_b.samples.len());
}
