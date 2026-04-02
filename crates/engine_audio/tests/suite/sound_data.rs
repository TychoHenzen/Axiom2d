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
    assert_eq!(frames, 4);
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
    assert_eq!(frames, 4);
}
