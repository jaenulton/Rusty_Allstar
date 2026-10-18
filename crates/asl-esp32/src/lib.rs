//! # asl-esp32
//!
//! ESP32 embedded firmware components for Rusty_Allstar.
//! Coordinates dual-core FreeRTOS tasks:
//! - Core 0: WiFi networking & IAX2 protocol streaming
//! - Core 1: I2S DMA audio, Goertzel DSP, and repeater GPIO controller

pub mod audio_queue;
pub mod pin_config;
pub mod radio_driver;
pub mod task_audio;

pub use audio_queue::{AudioFrame, AudioRingBuffer, FRAME_SIZE};
pub use pin_config::PinConfig;
pub use radio_driver::{MockRadioDriver, RadioGpio};
pub use task_audio::AudioDspEngine;
