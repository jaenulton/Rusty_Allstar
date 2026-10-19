//! # asl-esp32
//!
//! ESP32 embedded firmware components for Rusty_Allstar.
//! Coordinates dual-core FreeRTOS tasks:
//! - Core 0: WiFi networking, NVS config, & IAX2 protocol streaming
//! - Core 1: I2S DMA audio, Goertzel DSP, and repeater GPIO controller

#![allow(clippy::result_unit_err, clippy::field_reassign_with_default)]

pub mod audio_queue;
pub mod gpio_interrupts;
pub mod i2s_audio;
pub mod mock_radio;
pub mod nvs_config;
pub mod pin_config;
pub mod radio_driver;
pub mod sa818;
pub mod task_audio;

pub use audio_queue::{AudioFrame, AudioRingBuffer, FRAME_SIZE};
pub use gpio_interrupts::{EdgeType, GpioInterruptController, RadioEvent};
pub use i2s_audio::{I2sAudioConfig, I2sHardwareDriver, I2sStandard};
pub use mock_radio::{MockRadioState, MockSa818Emulator};
pub use nvs_config::NodeNvsConfig;
pub use pin_config::PinConfig;
pub use radio_driver::{MockRadioDriver, RadioGpio};
pub use sa818::{Bandwidth, EmphasisFilter, Sa818Config, Sa818Driver};
pub use task_audio::AudioDspEngine;
