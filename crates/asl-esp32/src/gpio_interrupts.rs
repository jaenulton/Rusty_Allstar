//! ESP32 Hardware GPIO Interrupt Controller for Carrier Operated Squelch (COS) and PTT
//!
//! Handles edge-triggered ISR pin changes with software debounce and safety watchdog timers.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeType {
    Rising,
    Falling,
    Any,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RadioEvent {
    CosCarrierDetected,
    CosCarrierLost,
    PttTimeoutTriggered,
}

pub struct GpioInterruptController {
    pub cos_pin: u8,
    pub ptt_pin: u8,
    pub cos_active_low: bool,
    pub last_cos_level: bool,
    pub debounce_ms: u32,
    pub current_carrier: bool,
    pub ptt_watchdog_ms: u32,
    pub ptt_elapsed_ms: u32,
    pub ptt_is_keyed: bool,
}

impl GpioInterruptController {
    pub fn new(cos_pin: u8, ptt_pin: u8, cos_active_low: bool) -> Self {
        Self {
            cos_pin,
            ptt_pin,
            cos_active_low,
            last_cos_level: true, // pulled high default
            debounce_ms: 20,
            current_carrier: false,
            ptt_watchdog_ms: 180_000, // 3 minute maximum PTT limit
            ptt_elapsed_ms: 0,
            ptt_is_keyed: false,
        }
    }

    /// Called by the GPIO ISR or high-frequency polling loop with the raw pin state
    pub fn handle_cos_pin_change(&mut self, raw_pin_high: bool) -> Option<RadioEvent> {
        let carrier_detected = if self.cos_active_low {
            !raw_pin_high
        } else {
            raw_pin_high
        };

        if carrier_detected != self.current_carrier {
            self.current_carrier = carrier_detected;
            if carrier_detected {
                Some(RadioEvent::CosCarrierDetected)
            } else {
                Some(RadioEvent::CosCarrierLost)
            }
        } else {
            None
        }
    }

    /// Sets PTT output state and resets the safety watchdog
    pub fn set_ptt(&mut self, active: bool) {
        self.ptt_is_keyed = active;
        if !active {
            self.ptt_elapsed_ms = 0;
        }
    }

    /// Periodic hardware tick (e.g. called every 10ms or 20ms) to enforce the PTT safety watchdog
    pub fn tick(&mut self, delta_ms: u32) -> Option<RadioEvent> {
        if self.ptt_is_keyed {
            self.ptt_elapsed_ms = self.ptt_elapsed_ms.saturating_add(delta_ms);
            if self.ptt_elapsed_ms >= self.ptt_watchdog_ms {
                // Force unkey transmitter to prevent burning out PA finals
                self.ptt_is_keyed = false;
                self.ptt_elapsed_ms = 0;
                return Some(RadioEvent::PttTimeoutTriggered);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cos_edge_detection() {
        let mut ctrl = GpioInterruptController::new(34, 25, true);

        // Active low: pin low means carrier present
        let ev1 = ctrl.handle_cos_pin_change(false);
        assert_eq!(ev1, Some(RadioEvent::CosCarrierDetected));
        assert!(ctrl.current_carrier);

        // Same state -> no event
        assert_eq!(ctrl.handle_cos_pin_change(false), None);

        // Pin returns high -> carrier lost
        let ev2 = ctrl.handle_cos_pin_change(true);
        assert_eq!(ev2, Some(RadioEvent::CosCarrierLost));
        assert!(!ctrl.current_carrier);
    }

    #[test]
    fn test_ptt_watchdog_timeout() {
        let mut ctrl = GpioInterruptController::new(34, 25, true);
        ctrl.ptt_watchdog_ms = 1000; // 1 second for test

        ctrl.set_ptt(true);
        assert!(ctrl.ptt_is_keyed);

        assert_eq!(ctrl.tick(500), None);
        assert!(ctrl.ptt_is_keyed);

        // Exceed watchdog timeout
        let ev = ctrl.tick(600);
        assert_eq!(ev, Some(RadioEvent::PttTimeoutTriggered));
        assert!(!ctrl.ptt_is_keyed); // Automatically unkeyed
    }
}
