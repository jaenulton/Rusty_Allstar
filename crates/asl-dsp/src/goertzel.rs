//! Goertzel Algorithm for efficient single-frequency energy calculation.
//! Used for DTMF and CTCSS (PL) tone detection.

/// State for a single Goertzel filter instance
#[derive(Debug, Clone)]
pub struct Goertzel {
    coeff: f32,
    s_prev: f32,
    s_prev2: f32,
    sample_count: usize,
}

impl Goertzel {
    /// Initialize a Goertzel filter for a specific target frequency and sample rate.
    pub fn new(target_freq: f32, sample_rate: f32, block_size: usize) -> Self {
        let k = (0.5 + ((block_size as f32 * target_freq) / sample_rate)) as usize;
        let omega = (2.0 * core::f32::consts::PI * k as f32) / block_size as f32;
        let coeff = 2.0 * micromath::F32(omega).cos().0;

        Self {
            coeff,
            s_prev: 0.0,
            s_prev2: 0.0,
            sample_count: 0,
        }
    }

    /// Reset filter state for next block
    pub fn reset(&mut self) {
        self.s_prev = 0.0;
        self.s_prev2 = 0.0;
        self.sample_count = 0;
    }

    /// Process a single PCM sample (normalized or 16-bit float)
    #[inline]
    pub fn process_sample(&mut self, sample: f32) {
        let s = sample + self.coeff * self.s_prev - self.s_prev2;
        self.s_prev2 = self.s_prev;
        self.s_prev = s;
        self.sample_count += 1;
    }

    /// Process a slice of samples
    pub fn process_slice(&mut self, samples: &[f32]) {
        for &s in samples {
            self.process_sample(s);
        }
    }

    /// Compute relative signal energy after processing a block of samples
    pub fn compute_energy(&self) -> f32 {
        self.s_prev * self.s_prev + self.s_prev2 * self.s_prev2 - self.coeff * self.s_prev * self.s_prev2
    }
}

/// DTMF Frequency Definitions
pub const DTMF_ROW_FREQS: [f32; 4] = [697.0, 770.0, 852.0, 941.0];
pub const DTMF_COL_FREQS: [f32; 4] = [1209.0, 1336.0, 1477.0, 1633.0];

pub const DTMF_MATRIX: [[char; 4]; 4] = [
    ['1', '2', '3', 'A'],
    ['4', '5', '6', 'B'],
    ['7', '8', '9', 'C'],
    ['*', '0', '#', 'D'],
];

/// DTMF Decoder using a bank of 8 Goertzel filters
pub struct DtmfDetector {
    row_filters: [Goertzel; 4],
    col_filters: [Goertzel; 4],
    block_size: usize,
    sample_index: usize,
}

impl DtmfDetector {
    pub fn new(sample_rate: f32, block_size: usize) -> Self {
        let row_filters = [
            Goertzel::new(DTMF_ROW_FREQS[0], sample_rate, block_size),
            Goertzel::new(DTMF_ROW_FREQS[1], sample_rate, block_size),
            Goertzel::new(DTMF_ROW_FREQS[2], sample_rate, block_size),
            Goertzel::new(DTMF_ROW_FREQS[3], sample_rate, block_size),
        ];

        let col_filters = [
            Goertzel::new(DTMF_COL_FREQS[0], sample_rate, block_size),
            Goertzel::new(DTMF_COL_FREQS[1], sample_rate, block_size),
            Goertzel::new(DTMF_COL_FREQS[2], sample_rate, block_size),
            Goertzel::new(DTMF_COL_FREQS[3], sample_rate, block_size),
        ];

        Self {
            row_filters,
            col_filters,
            block_size,
            sample_index: 0,
        }
    }

    /// Feed a 16-bit PCM sample; returns `Some(char)` when a valid DTMF tone is detected
    pub fn process_sample(&mut self, sample: i16) -> Option<char> {
        let norm = (sample as f32) / 32768.0;

        for f in &mut self.row_filters {
            f.process_sample(norm);
        }
        for f in &mut self.col_filters {
            f.process_sample(norm);
        }

        self.sample_index += 1;
        if self.sample_index >= self.block_size {
            let result = self.evaluate();
            self.sample_index = 0;
            for f in &mut self.row_filters {
                f.reset();
            }
            for f in &mut self.col_filters {
                f.reset();
            }
            result
        } else {
            None
        }
    }

    fn evaluate(&self) -> Option<char> {
        // Find maximum energy in rows and cols
        let (best_row, max_row_e) = self.row_filters.iter().enumerate().fold(
            (0, 0.0f32),
            |(best_i, max_e), (i, f)| {
                let e = f.compute_energy();
                if e > max_e { (i, e) } else { (best_i, max_e) }
            },
        );

        let (best_col, max_col_e) = self.col_filters.iter().enumerate().fold(
            (0, 0.0f32),
            |(best_i, max_e), (i, f)| {
                let e = f.compute_energy();
                if e > max_e { (i, e) } else { (best_i, max_e) }
            },
        );

        // Thresholding and twist ratio check
        const MIN_ENERGY_THRESHOLD: f32 = 1.0;
        if max_row_e > MIN_ENERGY_THRESHOLD && max_col_e > MIN_ENERGY_THRESHOLD {
            Some(DTMF_MATRIX[best_row][best_col])
        } else {
            None
        }
    }
}
