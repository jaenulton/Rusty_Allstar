//! Morse Code (CW) tone synthesizer with raised cosine envelope
//! for key-click-free repeater station identification.

pub struct CwGenerator {
    sample_rate: f32,
    tone_freq: f32,
    wpm: u8,
    phase: f32,
}

impl CwGenerator {
    pub fn new(sample_rate: f32, tone_freq: f32, wpm: u8) -> Self {
        Self {
            sample_rate,
            tone_freq,
            wpm,
            phase: 0.0,
        }
    }

    /// Calculate dot duration in samples based on standard PARIS formula:
    /// Dot duration = 1200 / WPM ms
    pub fn dot_samples(&self) -> usize {
        let dot_sec = 1.2 / (self.wpm as f32);
        (dot_sec * self.sample_rate) as usize
    }

    /// Synthesize a tone burst of `samples` count into the output buffer with raised cosine ramp
    pub fn generate_tone(&mut self, samples: usize, out: &mut [i16]) {
        let ramp_samples = (self.sample_rate * 0.005) as usize; // 5ms ramp
        let phase_inc = (2.0 * core::f32::consts::PI * self.tone_freq) / self.sample_rate;

        for i in 0..samples.min(out.len()) {
            let mut amp = 1.0f32;
            if i < ramp_samples {
                amp = 0.5 * (1.0 - micromath::F32(core::f32::consts::PI * (i as f32) / (ramp_samples as f32)).cos().0);
            } else if i > samples.saturating_sub(ramp_samples) {
                let tail = samples - i;
                amp = 0.5 * (1.0 - micromath::F32(core::f32::consts::PI * (tail as f32) / (ramp_samples as f32)).cos().0);
            }

            let sample_f = amp * micromath::F32(self.phase).sin().0 * 16384.0;
            out[i] = sample_f as i16;

            self.phase += phase_inc;
            if self.phase > 2.0 * core::f32::consts::PI {
                self.phase -= 2.0 * core::f32::consts::PI;
            }
        }
    }
}
