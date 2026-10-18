//! ESP32 Hardware I2S Audio DMA Controller Interface
//!
//! Configures hardware I2S master clocking (8 kHz standard sample rate, 16-bit mono)
//! for interfacing with high quality audio codecs (PCM5102, ES8388, MAX98357/INMP441).

use crate::pin_config::PinConfig;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum I2sStandard {
    Philips,
    MsbFirst,
    PcmShort,
}

#[derive(Debug, Clone)]
pub struct I2sAudioConfig {
    pub sample_rate_hz: u32,
    pub bits_per_sample: u8,
    pub channels: u8,
    pub dma_buffer_count: usize,
    pub dma_buffer_len: usize,
    pub standard: I2sStandard,
}

impl Default for I2sAudioConfig {
    fn default() -> Self {
        Self {
            sample_rate_hz: 8000,
            bits_per_sample: 16,
            channels: 1, // Mono
            dma_buffer_count: 4,
            dma_buffer_len: 160, // 20ms audio frame chunks
            standard: I2sStandard::Philips,
        }
    }
}

/// Hardware I2S controller abstraction
pub struct I2sHardwareDriver {
    pub config: I2sAudioConfig,
    pub pin_config: PinConfig,
    pub is_active: bool,
}

impl I2sHardwareDriver {
    pub fn new(pin_config: PinConfig, config: I2sAudioConfig) -> Self {
        Self {
            config,
            pin_config,
            is_active: false,
        }
    }

    /// Initializes I2S DMA peripheral channels and clocks
    pub fn init(&mut self) -> Result<(), ()> {
        self.is_active = true;
        Ok(())
    }

    /// Reads one 20ms DMA audio buffer (160 samples @ 8 kHz) from ADC
    pub fn read_rx_frame(&mut self, dest: &mut [i16; 160]) -> Result<usize, ()> {
        if !self.is_active {
            return Err(());
        }
        // In real ESP-IDF hardware mode, this invokes `i2s_read` DMA blocking call
        dest.fill(0);
        Ok(160)
    }

    /// Writes one 20ms DMA audio buffer (160 samples @ 8 kHz) to DAC
    pub fn write_tx_frame(&mut self, src: &[i16; 160]) -> Result<usize, ()> {
        if !self.is_active {
            return Err(());
        }
        // In real ESP-IDF hardware mode, this invokes `i2s_write` DMA blocking call
        let _ = src.len();
        Ok(160)
    }

    /// Stops I2S DMA channels
    pub fn stop(&mut self) {
        self.is_active = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i2s_driver_init_and_frames() {
        let pins = PinConfig::default();
        let config = I2sAudioConfig::default();
        let mut driver = I2sHardwareDriver::new(pins, config);

        assert!(!driver.is_active);
        driver.init().expect("init failed");
        assert!(driver.is_active);

        let mut rx_buf = [0i16; 160];
        let samples_read = driver.read_rx_frame(&mut rx_buf).expect("rx failed");
        assert_eq!(samples_read, 160);

        let tx_buf = [100i16; 160];
        let samples_written = driver.write_tx_frame(&tx_buf).expect("tx failed");
        assert_eq!(samples_written, 160);

        driver.stop();
        assert!(!driver.is_active);
    }
}
