//! Mock Hardware Radio Interface & SA818 Transceiver Emulator
//!
//! Provides an in-memory simulated RF channel and SA818 module for desktop
//! testing, QEMU simulations, and automated testing environments.

use alloc::string::String;
use alloc::vec::Vec;

/// Mock RF Channel state
#[derive(Debug, Clone)]
pub struct MockRadioState {
    pub connected: bool,
    pub tx_freq_hz: u32,
    pub rx_freq_hz: u32,
    pub tx_ctcss_hz: f32,
    pub rx_ctcss_hz: f32,
    pub squelch_level: u8,
    pub rf_carrier_detected: bool,
    pub transmitter_keyed: bool,
    pub command_log: Vec<String>,
}

impl Default for MockRadioState {
    fn default() -> Self {
        Self {
            connected: false,
            tx_freq_hz: 446_000_000,
            rx_freq_hz: 446_000_000,
            tx_ctcss_hz: 100.0,
            rx_ctcss_hz: 100.0,
            squelch_level: 4,
            rf_carrier_detected: false,
            transmitter_keyed: false,
            command_log: Vec::new(),
        }
    }
}

/// Simulated SA818 Transceiver Emulator
pub struct MockSa818Emulator {
    state: MockRadioState,
}

impl MockSa818Emulator {
    pub fn new() -> Self {
        Self {
            state: MockRadioState::default(),
        }
    }

    /// Process an inbound AT command line string and return the SA818 response
    pub fn handle_at_command(&mut self, cmd: &str) -> &'static str {
        let trimmed = cmd.trim();
        self.state.command_log.push(String::from(trimmed));

        if trimmed == "AT+DMOCONNECT" {
            self.state.connected = true;
            "+DMOCONNECT:0\r\n"
        } else if trimmed.starts_with("AT+DMOSETGROUP=") {
            // e.g. AT+DMOSETGROUP=0,446.0000,446.0000,100.0,4,100.0
            self.state.connected = true;
            "+DMOSETGROUP:0\r\n"
        } else if trimmed.starts_with("AT+DMOSETVOLUME=") {
            "+DMOSETVOLUME:0\r\n"
        } else if trimmed.starts_with("AT+SETFILTER=") {
            "+DMOSETFILTER:0\r\n"
        } else {
            "+DMOERROR\r\n"
        }
    }

    /// Set simulated RF carrier presence (triggers COS active/inactive)
    pub fn set_rf_carrier(&mut self, carrier_present: bool) {
        self.state.rf_carrier_detected = carrier_present;
    }

    /// Set simulated PTT transmitter key state
    pub fn set_ptt(&mut self, active: bool) {
        self.state.transmitter_keyed = active;
    }

    pub fn state(&self) -> &MockRadioState {
        &self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_sa818_at_handshake() {
        let mut radio = MockSa818Emulator::new();
        assert!(!radio.state().connected);

        let resp = radio.handle_at_command("AT+DMOCONNECT\r\n");
        assert_eq!(resp, "+DMOCONNECT:0\r\n");
        assert!(radio.state().connected);

        let resp = radio.handle_at_command("AT+DMOSETGROUP=0,446.0000,446.0000,100.0,4,100.0\r\n");
        assert_eq!(resp, "+DMOSETGROUP:0\r\n");
        assert_eq!(radio.state().command_log.len(), 2);
    }

    #[test]
    fn test_mock_rf_carrier_and_ptt() {
        let mut radio = MockSa818Emulator::new();
        assert!(!radio.state().rf_carrier_detected);
        assert!(!radio.state().transmitter_keyed);

        radio.set_rf_carrier(true);
        assert!(radio.state().rf_carrier_detected);

        radio.set_ptt(true);
        assert!(radio.state().transmitter_keyed);
    }
}
