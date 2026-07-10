#![allow(clippy::unwrap_used)]

use engine_audio::sound::SoundData;

/// @doc: For mono audio, each sample is one frame. `frame_count()` divides
/// by channel count — getting this wrong would cause the mixer to read half
/// the buffer (stereo assumption) or double it, producing glitchy playback.
#[test]
fn when_mono_then_frame_count_equals_sample_len() {
    // Arrange
    let sound = SoundData {
        samples: vec![0.1, 0.2, 0.3, 0.4],
        sample_rate: 44_100,
        channels: 1,
    };

    // Act
    let frames = sound.frame_count();

    // Assert
    assert_eq!(frames, 4, "mono frame_count should equal sample length");
}

/// @doc: Stereo audio interleaves L/R samples, so frame count is half the
/// sample count. The mixer uses frame count to determine playback duration —
/// a wrong value would cause sounds to end at half length or play garbage.
#[test]
fn when_stereo_then_frame_count_is_half_sample_len() {
    // Arrange
    let sound = SoundData {
        samples: vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8],
        sample_rate: 44_100,
        channels: 2,
    };

    // Act
    let frames = sound.frame_count();

    // Assert
    assert_eq!(
        frames, 4,
        "stereo frame_count should be half sample length (8 samples / 2 channels)"
    );
}

/// @doc: A `SoundData` with no samples has zero frames regardless of channel count.
#[test]
fn when_empty_samples_then_frame_count_zero() {
    // Arrange
    let sound = SoundData {
        samples: vec![],
        sample_rate: 44_100,
        channels: 1,
    };

    // Act
    let frames = sound.frame_count();

    // Assert
    assert_eq!(frames, 0, "frame_count should be 0 for empty samples");
}

/// @doc: When stereo samples aren't evenly divisible, `frame_count` truncates via integer division.
#[test]
fn when_stereo_odd_sample_count_then_frame_count_truncates() {
    // Arrange — 5 stereo frames would be 10 samples, but we have 5 → truncates to 2
    let sound = SoundData {
        samples: vec![0.0; 5],
        sample_rate: 44_100,
        channels: 2,
    };

    // Act
    let frames = sound.frame_count();

    // Assert
    assert_eq!(
        frames, 2,
        "stereo frame_count truncates: 5 samples / 2 channels = 2"
    );
}

/// @doc: Multi-channel audio (4 channels) correctly divides total samples by channel count.
#[test]
fn when_quad_audio_then_frame_count_is_quarter_sample_len() {
    // Arrange
    let sound = SoundData {
        samples: vec![0.0; 8],
        sample_rate: 44_100,
        channels: 4,
    };

    // Act
    let frames = sound.frame_count();

    // Assert
    assert_eq!(frames, 2, "quad frame_count should be 8 / 4 = 2");
}

/// @doc: A single stereo sample (odd count) truncates to zero frames — edge case of integer division.
#[test]
fn when_one_sample_stereo_then_frame_count_zero() {
    // Arrange
    let sound = SoundData {
        samples: vec![0.5],
        sample_rate: 44_100,
        channels: 2,
    };

    // Act
    let frames = sound.frame_count();

    // Assert
    assert_eq!(
        frames, 0,
        "stereo frame_count with 1 sample should be 0 (1 / 2)"
    );
}

/// @doc: Mono audio with a single sample still produces one frame.
#[test]
fn when_one_sample_mono_then_frame_count_one() {
    // Arrange
    let sound = SoundData {
        samples: vec![-0.3],
        sample_rate: 22_050,
        channels: 1,
    };

    // Act
    let frames = sound.frame_count();

    // Assert
    assert_eq!(frames, 1, "mono frame_count with 1 sample should be 1");
}

/// @doc: Zero channels causes division by zero panic — callers must ensure channels > 0.
#[test]
#[should_panic(expected = "divide by zero")]
fn when_zero_channels_then_frame_count_panics() {
    // Arrange
    let sound = SoundData {
        samples: vec![0.0; 4],
        sample_rate: 44_100,
        channels: 0,
    };

    // Act — division by zero panics
    let _ = sound.frame_count();
}
