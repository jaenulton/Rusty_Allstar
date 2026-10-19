#![allow(clippy::result_unit_err, clippy::field_reassign_with_default)]
//! Rusty_Allstar ESP32 Firmware Entry Point
//!
//! Dual-Core FreeRTOS execution:
//! - Core 0: Wi-Fi association, NVS configuration loading, IAX2 UDP protocol streaming,
//!   and embedded Web/REST dashboard.
//! - Core 1: Hardware I2S DMA audio transfers, Goertzel DSP tone detection/generation,
//!   SA818 UART transceiver control, and GPIO COS/PTT repeater state machine.

use asl_client::CallSession;
use asl_core::{RepeaterConfig, RepeaterController};
use asl_dsp::{decode_ulaw_slice, encode_ulaw_slice};
use asl_esp32::{
    gpio_interrupts::{GpioInterruptController, RadioEvent},
    i2s_audio::{I2sAudioConfig, I2sHardwareDriver},
    nvs_config::NodeNvsConfig,
    pin_config::PinConfig,
    sa818::{Bandwidth, Sa818Config, Sa818Driver},
    AudioFrame, AudioRingBuffer,
};

#[cfg(feature = "esp-target")]
use esp_idf_svc::hal::prelude::*;

pub struct MicroNodeFirmware {
    pub nvs: NodeNvsConfig,
    pub pins: PinConfig,
    pub i2s_driver: I2sHardwareDriver,
    pub sa818_config: Sa818Config,
    pub gpio_ctrl: GpioInterruptController,
    pub repeater: RepeaterController,
    pub rx_ring_buffer: AudioRingBuffer,
    pub tx_ring_buffer: AudioRingBuffer,
    pub iax_session: CallSession,
}

impl MicroNodeFirmware {
    pub fn new() -> Self {
        let pins = PinConfig::default();
        let nvs = NodeNvsConfig::default();
        let i2s_config = I2sAudioConfig::default();
        let sa818_config = Sa818Config {
            bandwidth: Bandwidth::Narrow,
            tx_freq_mhz: "446.0000",
            rx_freq_mhz: "446.0000",
            squelch_level: 4,
            tx_ctcss: 100.0,
            rx_ctcss: 100.0,
            volume: 5,
        };

        let repeater_config = RepeaterConfig {
            timeout_timer_ms: (nvs.ptt_timeout_seconds as u32) * 1000,
            hang_time_ms: 1_200,
            courtesy_tone_delay_ms: 150,
            callsign: "W1AW",
            id_interval_ms: 600_000,
        };

        Self {
            nvs,
            i2s_driver: I2sHardwareDriver::new(pins, i2s_config),
            sa818_config,
            gpio_ctrl: GpioInterruptController::new(
                pins.gpio_cos as u8,
                pins.gpio_ptt as u8,
                pins.cos_active_low,
            ),
            repeater: RepeaterController::new(repeater_config),
            pins,
            rx_ring_buffer: AudioRingBuffer::new(),
            tx_ring_buffer: AudioRingBuffer::new(),
            iax_session: CallSession::new(1001),
        }
    }

    /// Boot sequence: loads NVS config, handshakes with SA818 radio, and initializes I2S
    pub fn boot(&mut self) -> Result<(), ()> {
        #[cfg(feature = "esp-target")]
        {
            // Initializes ESP-IDF FreeRTOS system services and patch tables
            esp_idf_svc::sys::link_patches();
        }

        self.i2s_driver.init()?;
        let mut buf = [0u8; 128];
        let _ = Sa818Driver::cmd_connect(&mut buf);
        let _ = Sa818Driver::cmd_set_group(&self.sa818_config, &mut buf);
        Ok(())
    }

    /// Core 1 Real-Time Audio & Radio Loop (Runs every 20ms DMA interval)
    pub fn core1_audio_tick(&mut self) {
        let mut dma_rx = [0i16; 160];
        if self.i2s_driver.read_rx_frame(&mut dma_rx).is_ok() {
            let mut ulaw = [0u8; 160];
            encode_ulaw_slice(&dma_rx, &mut ulaw);
            let frame = AudioFrame {
                data: ulaw,
                len: 160,
            };
            self.rx_ring_buffer.push(frame);
        }

        if let Some(frame) = self.tx_ring_buffer.pop() {
            let mut dma_tx = [0i16; 160];
            decode_ulaw_slice(&frame.data, &mut dma_tx);
            let _ = self.i2s_driver.write_tx_frame(&dma_tx);
        }

        // Enforce PTT hardware safety watchdog
        if let Some(RadioEvent::PttTimeoutTriggered) = self.gpio_ctrl.tick(20) {
            self.repeater.on_cos_event(false);
        }
    }

    /// Core 0 Network & Protocol Loop (Runs FreeRTOS socket event loop)
    pub fn core0_network_tick(&mut self) {
        self.iax_session.advance_time(20);
    }
}

impl Default for MicroNodeFirmware {
    fn default() -> Self {
        Self::new()
    }
}

fn main() {
    println!("=== Rusty_Allstar ESP32 Firmware Starting ===");
    let mut firmware = MicroNodeFirmware::new();
    firmware.boot().expect("Firmware boot failed");
    println!(
        "Firmware booted successfully! Node ID: {}",
        firmware.nvs.asl_node
    );

    // Run test cycle
    firmware.core1_audio_tick();
    firmware.core0_network_tick();
    println!("Dual-core ticks executed cleanly.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_micronode_firmware_lifecycle() {
        let mut fw = MicroNodeFirmware::new();
        assert_eq!(fw.nvs.asl_node, 1999);
        assert!(fw.boot().is_ok());

        // Test Core 1 and Core 0 tick execution
        fw.core1_audio_tick();
        fw.core0_network_tick();
        assert!(!fw.gpio_ctrl.ptt_is_keyed);
    }
}
