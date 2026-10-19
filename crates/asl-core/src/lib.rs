//! # asl-core
//!
//! Core repeater controller state machine, timer handling, conference bridge, and telemetry logic for Rusty_Allstar.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod conference_bridge;
pub mod dtmf_command;
pub mod repeater;

pub use conference_bridge::{ConferenceBridge, ConnectedPeer, LinkMode};
pub use dtmf_command::{DtmfCommand, DtmfCommandProcessor};
pub use repeater::{Action, ControllerState, RepeaterConfig, RepeaterController};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repeater_cycle() {
        let config = RepeaterConfig {
            timeout_timer_ms: 10_000,
            hang_time_ms: 1_000,
            courtesy_tone_delay_ms: 100,
            callsign: "W1AW",
            id_interval_ms: 60_000,
        };

        let mut controller = RepeaterController::new(config);
        assert_eq!(controller.state(), ControllerState::Idle);
        assert_eq!(controller.is_ptt_active(), false);

        // User keys up
        let action = controller.on_cos_event(true);
        assert_eq!(controller.state(), ControllerState::LocalRx);
        assert_eq!(controller.is_ptt_active(), true);
        match action {
            Some(Action::SetPtt(true)) => {}
            _ => panic!("Expected SetPtt(true)"),
        }

        // Advance 5 seconds
        controller.tick(5000);
        assert_eq!(controller.state(), ControllerState::LocalRx);

        // User unkeys
        let action = controller.on_cos_event(false);
        assert_eq!(controller.state(), ControllerState::CourtesyTone);
        match action {
            Some(Action::PlayCourtesyTone) => {}
            _ => panic!("Expected PlayCourtesyTone"),
        }

        // Advance through courtesy tone into hang time
        controller.tick(150);
        assert_eq!(controller.state(), ControllerState::HangTime);
        assert_eq!(controller.is_ptt_active(), true);

        // Advance beyond hang time
        let action = controller.tick(1100);
        assert_eq!(controller.state(), ControllerState::Idle);
        assert_eq!(controller.is_ptt_active(), false);
        match action {
            Some(Action::SetPtt(false)) => {}
            _ => panic!("Expected SetPtt(false)"),
        }
    }
}
