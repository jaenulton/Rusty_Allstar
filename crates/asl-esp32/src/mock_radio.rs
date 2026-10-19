#![allow(clippy::manual_strip)]
//! Mock NiceRF SA818 Transceiver & RF Environment Emulator
//!
//! Emulates serial AT command interactions, carrier detection, and PTT actuation
//! for automated headless desktop testing and simulation.

use std::string::String;
use std::vec::Vec;

#[derive(Debug, Clone, PartialEq)]
pub struct MockRadioState {
    pub connected: bool,
    pub tx_freq: String,
    pub rx_freq: String,
    pub tx_ctcss: String,
    pub rx_ctcss: String,
    pub squelch: u8,
    pub volume: u8,
    pub ptt_active: bool,
    pub rf_carrier_detected: bool,
}

impl Default for MockRadioState {
    fn default() -> Self {
        Self {
            connected: false,
            tx_freq: "446.0000".to_string(),
            rx_freq: "446.0000".to_string(),
            tx_ctcss: "0100".to_string(),
            rx_ctcss: "0100".to_string(),
            squelch: 4,
            volume: 5,
            ptt_active: false,
            rf_carrier_detected: false,
        }
    }
}

pub struct MockSa818Emulator {
    pub state: MockRadioState,
}

impl Default for MockSa818Emulator {
    fn default() -> Self {
        Self::new()
    }
}

impl MockSa818Emulator {
    pub fn new() -> Self {
        Self {
            state: MockRadioState::default(),
        }
    }

    /// Processes an incoming serial AT command and returns the simulated response string
    pub fn process_at_command(&mut self, cmd: &str) -> String {
        let trimmed = cmd.trim();
        if trimmed == "AT+DMOCONNECT" {
            self.state.connected = true;
            "+DMOCONNECT:0\r\n".to_string()
        } else if trimmed.starts_with("AT+DMOSETGROUP=") {
            let params = &trimmed["AT+DMOSETGROUP=".len()..];
            let parts: Vec<&str> = params.split(',').collect();
            if parts.len() == 6 {
                self.state.tx_freq = parts[1].to_string();
                self.state.rx_freq = parts[2].to_string();
                self.state.tx_ctcss = parts[3].to_string();
                if let Ok(sq) = parts[4].parse::<u8>() {
                    self.state.squelch = sq;
                }
                self.state.rx_ctcss = parts[5].to_string();
                "+DMOSETGROUP:0\r\n".to_string()
            } else {
                "+DMOSETGROUP:1\r\n".to_string()
            }
        } else if trimmed.starts_with("AT+DMOSETVOLUME=") {
            let vol_str = &trimmed["AT+DMOSETVOLUME=".len()..];
            if let Ok(vol) = vol_str.parse::<u8>() {
                self.state.volume = vol;
                "+DMOSETVOLUME:0\r\n".to_string()
            } else {
                "+DMOSETVOLUME:1\r\n".to_string()
            }
        } else {
            "ERROR\r\n".to_string()
        }
    }

    pub fn set_rf_carrier(&mut self, detected: bool) {
        self.state.rf_carrier_detected = detected;
    }

    pub fn set_ptt(&mut self, keyed: bool) {
        self.state.ptt_active = keyed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_sa818_at_handshake() {
        let mut radio = MockSa818Emulator::new();
        assert!(!radio.state.connected);

        let resp = radio.process_at_command("AT+DMOCONNECT\r\n");
        assert_eq!(resp, "+DMOCONNECT:0\r\n");
        assert!(radio.state.connected);

        let group_resp =
            radio.process_at_command("AT+DMOSETGROUP=0,146.5200,146.5200,0100,3,0100\r\n");
        assert_eq!(group_resp, "+DMOSETGROUP:0\r\n");
        assert_eq!(radio.state.tx_freq, "146.5200");
        assert_eq!(radio.state.squelch, 3);
    }

    #[test]
    fn test_mock_rf_carrier_and_ptt() {
        let mut radio = MockSa818Emulator::new();
        assert!(!radio.state.rf_carrier_detected);
        assert!(!radio.state.ptt_active);

        radio.set_rf_carrier(true);
        assert!(radio.state.rf_carrier_detected);

        radio.set_ptt(true);
        assert!(radio.state.ptt_active);
    }
}
