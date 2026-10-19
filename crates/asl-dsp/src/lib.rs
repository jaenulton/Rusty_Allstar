//! # asl-dsp
//!
//! Digital Signal Processing components for AllStarLink nodes and repeater controllers.
//! Includes G.711 μ-law / A-law codecs, Goertzel tone detection, CTCSS squelch, CW synthesizer,
//! adaptive VoIP jitter buffer, and multi-stream audio conference mixer.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod audio_mixer;
pub mod ctcss;
pub mod cw;
pub mod g711;
pub mod goertzel;
pub mod jitter_buffer;

pub use audio_mixer::{soft_clip, AudioMixer, AudioStreamSource, SAMPLES_PER_FRAME};
pub use ctcss::{CtcssDetector, CtcssGenerator, STANDARD_CTCSS_TONES};
pub use cw::CwGenerator;
pub use g711::{decode_ulaw_slice, encode_ulaw_slice, linear_to_ulaw, ulaw_to_linear};
pub use goertzel::{DtmfDetector, Goertzel};
pub use jitter_buffer::{JitterBuffer, JitterBufferConfig, JitterPacket};
