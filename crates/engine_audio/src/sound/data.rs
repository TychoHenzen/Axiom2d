#[derive(Debug, Clone)]
pub struct SoundData {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
}

impl SoundData {
    #[must_use]
    pub fn frame_count(&self) -> usize {
        self.samples.len() / self.channels as usize
    }
}
