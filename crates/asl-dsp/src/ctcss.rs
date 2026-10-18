//! Continuous Tone-Coded Squelch System (CTCSS / PL Tone)
//! Detection and Generation for sub-audible radio squelch control.

use crate::goertzel::Goertzel;

/// Standard EIA/TIA CTCSS Frequencies (in Hz * 10 to allow exact integer representation)
pub const STANDARD_CTCSS_TONES: [f32; 50] = [
    67.0, 69.3, 71.9, 74.4, 77.0, 79.7, 82.5, 85.4, 88.5, 91.5,
    94.8, 97.4, 100.0, 103.5, 107.2, 110.9, 114.8, 118.8, 123.0, 127.3,
    131.8, 136.5, 141.3, 146.2, 151.4, 156.7, 159.8, 162.2, 165.5, 167.9,
    171.3, 173.8, 177.3, 179.9, 183.5, 186.2, 189.9, 192.8, 196.6, 199.5,
    203.5, 206.5, 210.7, 218.1, 225.7, 229.1, 233.6, 241.8, 250.3, 254.1,
];

/// CTCSS Detector tuned to a single specified PL tone frequency
pub struct CtcssDetector {
    filter: Goertzel,
    target_freq: f32,
    block_size: usize,
    sample_index: usize,
    detected: bool,
    threshold: f32,
}

impl CtcssDetector {
    /// Create a detector for a specific PL tone (e.g. 100.0 Hz) at a given sample rate (e.g. 8000 Hz)
    pub fn new(target_freq: f32, sample_rate: f32, block_size: usize, threshold: f32) -> Self {
        let filter = Goertzel::new(target_freq, sample_rate, block_size);
        Self {
            filter,
            target_freq,
            block_size,
            sample_index: 0,
            detected: false,
            threshold,
        }
    }

    pub fn target_freq(&self) -> f32 {
        self.target_freq
    }

    pub fn is_detected(&self) -> bool {
        self.detected
    }

    /// Feed a normalized audio sample (-1.0 to 1.0).
    /// Returns true if the tone detection status just changed.
    pub fn process_sample(&mut self, sample: f32) -> bool {
        self.filter.process_sample(sample);
        self.sample_index += 1;

        if self.sample_index >= self.block_size {
            let energy = self.filter.compute_energy();
            let new_detected = energy >= self.threshold;
            let changed = new_detected != self.detected;

            self.detected = new_detected;
            self.sample_index = 0;
            self.filter.reset();

            changed
        } else {
            false
        }
    }
}

/// Continuous sub-audible tone generator for transmitter CTCSS encoding
pub struct CtcssGenerator {
    sample_rate: f32,
    tone_freq: f32,
    amplitude: f32,
    phase: f32,
}

impl CtcssGenerator {
    /// Create CTCSS tone generator (standard amplitude is ~15% of full deviation, e.g. 0.15)
    pub fn new(sample_rate: f32, tone_freq: f32, amplitude: f32) -> Self {
        Self {
            sample_rate,
            tone_freq,
            amplitude,
            phase: 0.0,
        }
    }

    /// Mix CTCSS tone into existing PCM samples in-place
    pub fn mix_into_pcm(&mut self, samples: &mut [i16]) {
        let phase_inc = (2.0 * core::f32::consts::PI * self.tone_freq) / self.sample_rate;
        let scale = self.amplitude * 32767.0;

        for s in samples.iter_mut() {
            let tone_sample = (micromath::F32(self.phase).sin().0 * scale) as i32;
            let mixed = (*s as i32) + tone_sample;
            *s = mixed.clamp(i16::MIN as i32, i16::MAX as i32) as i16;

            self.phase += phase_inc;
            if self.phase >= 2.0 * core::f32::consts::PI {
                self.phase -= 2.0 * core::f32::consts::PI;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ctcss_detection() {
        let sample_rate = 8000.0;
        let target_tone = 100.0;
        let block_size = 800; // 100ms window
        let threshold = 50.0;

        let mut detector = CtcssDetector::new(target_tone, sample_rate, block_size, threshold);

        // Feed silence -> should not detect
        for _ in 0..block_size {
            detector.process_sample(0.0);
        }
        assert_eq!(detector.is_detected(), false);

        // Feed pure 100 Hz tone
        let phase_inc = (2.0 * core::f32::consts::PI * target_tone) / sample_rate;
        let mut phase = 0.0f32;
        for _ in 0..block_size {
            let sample = micromath::F32(phase).sin().0 * 0.5;
            detector.process_sample(sample);
            phase += phase_inc;
        }
        assert_eq!(detector.is_detected(), true);
    }
}
