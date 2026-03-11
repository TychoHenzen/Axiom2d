use fundsp::prelude::AudioUnit;

use crate::sound_data::SoundData;

pub struct SoundEffect {
    factory: Box<dyn Fn() -> Box<dyn AudioUnit> + Send + Sync>,
}

impl SoundEffect {
    pub fn new(factory: impl Fn() -> Box<dyn AudioUnit> + Send + Sync + 'static) -> Self {
        Self {
            factory: Box::new(factory),
        }
    }

    pub fn synthesize(&self, sample_rate: u32, duration: f32) -> SoundData {
        let mut graph = (self.factory)();
        let frame_count = (sample_rate as f32 * duration) as usize;
        let channels = graph.outputs();
        graph.set_sample_rate(f64::from(sample_rate));
        graph.reset();

        let mut samples = Vec::with_capacity(frame_count * channels);
        let mut output = vec![0.0_f32; channels];

        for _ in 0..frame_count {
            graph.tick(&[], &mut output);
            samples.extend_from_slice(&output);
        }

        SoundData {
            samples,
            sample_rate,
            channels: channels as u16,
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use fundsp::hacker32::*;

    use super::*;

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

    #[test]
    fn when_synthesize_called_then_sample_length_equals_frame_count_times_channels() {
        // Arrange
        let effect = test_effect();

        // Act
        let sound = effect.synthesize(44_100, 1.0);

        // Assert
        assert_eq!(sound.samples.len(), 44_100);
    }

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
    fn when_synthesize_called_twice_then_each_call_returns_fresh_sound_data() {
        // Arrange
        let effect = test_effect();

        // Act
        let sound_a = effect.synthesize(44_100, 0.1);
        let sound_b = effect.synthesize(44_100, 0.1);

        // Assert
        assert_eq!(sound_a.samples.len(), sound_b.samples.len());
    }
}
