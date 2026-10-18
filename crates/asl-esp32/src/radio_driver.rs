//! Radio Hardware Driver Interface (PTT Keying & COS Squelch Sensing)

pub trait RadioGpio {
    /// Read Carrier Operated Squelch (COS) state. Returns true if carrier present.
    fn read_cos(&mut self) -> bool;

    /// Set Push-To-Talk (PTT) line state.
    fn set_ptt(&mut self, active: bool);

    /// Set Status LED state.
    fn set_led(&mut self, active: bool);
}

/// In-memory mock radio driver for testing and desktop simulation
pub struct MockRadioDriver {
    pub cos_state: bool,
    pub ptt_state: bool,
    pub led_state: bool,
}

impl MockRadioDriver {
    pub fn new() -> Self {
        Self {
            cos_state: false,
            ptt_state: false,
            led_state: false,
        }
    }
}

impl Default for MockRadioDriver {
    fn default() -> Self {
        Self::new()
    }
}

impl RadioGpio for MockRadioDriver {
    fn read_cos(&mut self) -> bool {
        self.cos_state
    }

    fn set_ptt(&mut self, active: bool) {
        self.ptt_state = active;
    }

    fn set_led(&mut self, active: bool) {
        self.led_state = active;
    }
}
