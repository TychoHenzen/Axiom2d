use fundsp::prelude::AudioUnit;

use super::data::SoundData;

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
