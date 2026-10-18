//! # asl-dsp
//!
//! Digital Signal Processing components for AllStarLink nodes and repeater controllers.
//! Includes G.711 μ-law / A-law codecs, Goertzel tone detection, CTCSS squelch, and CW synthesizer.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod ctcss;
pub mod cw;
pub mod g711;
pub mod goertzel;

pub use ctcss::{CtcssDetector, CtcssGenerator, STANDARD_CTCSS_TONES};
pub use cw::CwGenerator;
pub use g711::{decode_ulaw_slice, encode_ulaw_slice, linear_to_ulaw, ulaw_to_linear};
pub use goertzel::{DtmfDetector, Goertzel};
