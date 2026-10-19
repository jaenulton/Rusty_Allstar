//! Audio Conference Mixer with Soft-Clipping and Anti-Echo Muting
//!
//! Mixes multiple 8 kHz 16-bit linear PCM audio streams into a single output stream,
//! applying N-1 subtraction (or source exclusion) so participants do not hear their own echo,
//! and soft-limiting (tanh-like distortion curve) to prevent digital harsh clipping.

pub const SAMPLES_PER_FRAME: usize = 160; // 20ms @ 8 kHz

/// Soft-clip a 32-bit accumulated audio sample into the 16-bit dynamic range (-32768..=32767).
/// Uses an algebraic rational soft-clipper approximation of tanh(x):
/// f(x) = x / (1 + |x| / limit)
#[inline]
pub fn soft_clip(sample: i32) -> i16 {
    const THRESHOLD: i32 = 24000;
    const MAX_PEAK: i32 = 32000;

    let abs_s = sample.abs();
    if abs_s <= THRESHOLD {
        sample as i16
    } else {
        // Smooth knee above threshold
        let excess = abs_s - THRESHOLD;
        let compressed =
            THRESHOLD + (excess * (MAX_PEAK - THRESHOLD)) / (excess + (MAX_PEAK - THRESHOLD));
        if sample > 0 {
            compressed.min(32767) as i16
        } else {
            (-compressed).max(-32768) as i16
        }
    }
}

/// A participant stream in the conference bridge
pub struct AudioStreamSource {
    pub call_id: u32,
    pub pcm_samples: [i16; SAMPLES_PER_FRAME],
    pub active: bool,
}

/// Multi-stream audio conference mixer
pub struct AudioMixer {
    accumulator: [i32; SAMPLES_PER_FRAME],
}

impl AudioMixer {
    pub fn new() -> Self {
        Self {
            accumulator: [0i32; SAMPLES_PER_FRAME],
        }
    }

    /// Mix multiple incoming participant streams together.
    /// Writes each participant's customized mix (all active audio *except* their own)
    /// into the provided `outputs` buffer.
    #[allow(clippy::needless_range_loop)]
    pub fn mix_n_minus_one(
        &mut self,
        sources: &[AudioStreamSource],
        outputs: &mut [(u32, [i16; SAMPLES_PER_FRAME])],
    ) {
        // 1. Accumulate all active incoming streams
        self.accumulator.fill(0);
        for src in sources.iter() {
            if src.active {
                for i in 0..SAMPLES_PER_FRAME {
                    self.accumulator[i] += src.pcm_samples[i] as i32;
                }
            }
        }

        // 2. Generate customized N-1 mix for each output channel
        for (target_call_id, out_buf) in outputs.iter_mut() {
            // Find target's own input to subtract out
            let own_src = sources
                .iter()
                .find(|s| s.call_id == *target_call_id && s.active);

            for i in 0..SAMPLES_PER_FRAME {
                let mut sum = self.accumulator[i];
                if let Some(src) = own_src {
                    sum -= src.pcm_samples[i] as i32;
                }
                out_buf[i] = soft_clip(sum);
            }
        }
    }

    /// Simple master mix of all active streams (e.g. for local transmitter or monitor)
    #[allow(clippy::needless_range_loop)]
    pub fn mix_master(
        &mut self,
        sources: &[AudioStreamSource],
        master_out: &mut [i16; SAMPLES_PER_FRAME],
    ) {
        self.accumulator.fill(0);
        for src in sources.iter() {
            if src.active {
                for i in 0..SAMPLES_PER_FRAME {
                    self.accumulator[i] += src.pcm_samples[i] as i32;
                }
            }
        }
        for i in 0..SAMPLES_PER_FRAME {
            master_out[i] = soft_clip(self.accumulator[i]);
        }
    }
}

impl Default for AudioMixer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soft_clip_linear_region() {
        assert_eq!(soft_clip(0), 0);
        assert_eq!(soft_clip(10000), 10000);
        assert_eq!(soft_clip(-15000), -15000);
    }

    #[test]
    fn test_soft_clip_saturation() {
        let saturated_high = soft_clip(100_000);
        // bounded by i16
        assert!(saturated_high > 30000);

        let saturated_low = soft_clip(-100_000);
        assert!(saturated_low >= -32768);
        assert!(saturated_low < -30000);
    }

    #[test]
    fn test_n_minus_one_echo_suppression() {
        let mut mixer = AudioMixer::new();

        let node_a = AudioStreamSource {
            call_id: 101,
            pcm_samples: [1000i16; SAMPLES_PER_FRAME],
            active: true,
        };

        let node_b = AudioStreamSource {
            call_id: 102,
            pcm_samples: [2000i16; SAMPLES_PER_FRAME],
            active: true,
        };

        let sources = [node_a, node_b];
        let mut outputs = [
            (101, [0i16; SAMPLES_PER_FRAME]), // Should hear Node B only (2000)
            (102, [0i16; SAMPLES_PER_FRAME]), // Should hear Node A only (1000)
        ];

        mixer.mix_n_minus_one(&sources, &mut outputs);

        assert_eq!(outputs[0].1[0], 2000);
        assert_eq!(outputs[1].1[0], 1000);
    }

    #[test]
    fn test_master_mix_sum() {
        let mut mixer = AudioMixer::new();
        let sources = [
            AudioStreamSource {
                call_id: 1,
                pcm_samples: [5000i16; SAMPLES_PER_FRAME],
                active: true,
            },
            AudioStreamSource {
                call_id: 2,
                pcm_samples: [7000i16; SAMPLES_PER_FRAME],
                active: true,
            },
        ];

        let mut master = [0i16; SAMPLES_PER_FRAME];
        mixer.mix_master(&sources, &mut master);
        assert_eq!(master[0], 12000);
    }
}
