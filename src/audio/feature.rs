#[derive(Debug, Clone, Copy)]
pub struct AudioFeature {
    pub rms: f32,
    pub zcr: f32
}

impl AudioFeature {
    // TODO: calculate features
    pub fn calculate(&mut self, _samples: &[f32]) {
        self.rms = (_samples
            .iter()
            .map(|s| s * s)
            .sum::<f32>() / _samples.len() as f32)
            .sqrt();
        
        // zero crossing rate currently for simplisitc impl
        // usemore advanced for future use
        self.zcr = _samples
            .windows(2)
            .map(|w| (w[0] * w[1] < 0.0) as u32)
            .sum::<u32>() as f32 / (_samples.len() - 1) as f32;
    }
}