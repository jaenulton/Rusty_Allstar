//! Repeater Controller State Machine
//! Replaces Asterisk app_rpt monolithic logic with an event-driven,
//! deterministic finite state machine suitable for embedded microcontrollers.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerState {
    Idle,
    LocalRx,
    CourtesyTone,
    HangTime,
    TimedOut,
    NetworkRx,
}

#[derive(Debug, Clone)]
pub struct RepeaterConfig {
    /// Maximum transmission duration in milliseconds before TOT kicks in
    pub timeout_timer_ms: u32,
    /// Hang time duration in milliseconds after unkey before dropping PTT
    pub hang_time_ms: u32,
    /// Courtesy tone delay in milliseconds after unkey
    pub courtesy_tone_delay_ms: u32,
    /// Station callsign for CW ID
    pub callsign: &'static str,
    /// Periodic ID interval in milliseconds (e.g. 10 minutes = 600,000 ms)
    pub id_interval_ms: u32,
}

impl Default for RepeaterConfig {
    fn default() -> Self {
        Self {
            timeout_timer_ms: 180_000,    // 3 minutes
            hang_time_ms: 1_500,          // 1.5 seconds
            courtesy_tone_delay_ms: 200,  // 200 ms
            callsign: "N0CALL",
            id_interval_ms: 600_000,      // 10 minutes
        }
    }
}

pub enum Action {
    SetPtt(bool),
    PlayCourtesyTone,
    PlayCwId(&'static str),
}

pub struct RepeaterController {
    config: RepeaterConfig,
    state: ControllerState,
    state_elapsed_ms: u32,
    last_id_elapsed_ms: u32,
    cos_active: bool,
    network_rx_active: bool,
}

impl RepeaterController {
    pub fn new(config: RepeaterConfig) -> Self {
        Self {
            config,
            state: ControllerState::Idle,
            state_elapsed_ms: 0,
            last_id_elapsed_ms: 0,
            cos_active: false,
            network_rx_active: false,
        }
    }

    pub fn state(&self) -> ControllerState {
        self.state
    }

    pub fn is_ptt_active(&self) -> bool {
        match self.state {
            ControllerState::LocalRx
            | ControllerState::CourtesyTone
            | ControllerState::HangTime
            | ControllerState::NetworkRx => true,
            ControllerState::Idle | ControllerState::TimedOut => false,
        }
    }

    /// Event: Carrier Operated Squelch (COS) pin state change
    pub fn on_cos_event(&mut self, active: bool) -> Option<Action> {
        self.cos_active = active;

        match (self.state, active) {
            (ControllerState::Idle, true) => {
                self.state = ControllerState::LocalRx;
                self.state_elapsed_ms = 0;
                Some(Action::SetPtt(true))
            }
            (ControllerState::HangTime, true) | (ControllerState::CourtesyTone, true) => {
                self.state = ControllerState::LocalRx;
                self.state_elapsed_ms = 0;
                None
            }
            (ControllerState::LocalRx, false) => {
                self.state = ControllerState::CourtesyTone;
                self.state_elapsed_ms = 0;
                Some(Action::PlayCourtesyTone)
            }
            (ControllerState::TimedOut, false) => {
                // User unkeyed after timeout; return to idle
                self.state = ControllerState::Idle;
                self.state_elapsed_ms = 0;
                None
            }
            _ => None,
        }
    }

    /// Event: Network audio start/stop from IAX2
    pub fn on_network_rx_event(&mut self, active: bool) -> Option<Action> {
        self.network_rx_active = active;

        if active && self.state == ControllerState::Idle {
            self.state = ControllerState::NetworkRx;
            self.state_elapsed_ms = 0;
            Some(Action::SetPtt(true))
        } else if !active && self.state == ControllerState::NetworkRx {
            self.state = ControllerState::HangTime;
            self.state_elapsed_ms = 0;
            None
        } else {
            None
        }
    }

    /// Advance time by `delta_ms` (called periodically by timer tick or loop)
    pub fn tick(&mut self, delta_ms: u32) -> Option<Action> {
        self.state_elapsed_ms += delta_ms;
        self.last_id_elapsed_ms += delta_ms;

        // Check periodic station CW ID
        if self.last_id_elapsed_ms >= self.config.id_interval_ms {
            self.last_id_elapsed_ms = 0;
            return Some(Action::PlayCwId(self.config.callsign));
        }

        match self.state {
            ControllerState::LocalRx => {
                if self.state_elapsed_ms >= self.config.timeout_timer_ms {
                    self.state = ControllerState::TimedOut;
                    return Some(Action::SetPtt(false));
                }
            }
            ControllerState::CourtesyTone => {
                if self.state_elapsed_ms >= self.config.courtesy_tone_delay_ms {
                    self.state = ControllerState::HangTime;
                    self.state_elapsed_ms = 0;
                }
            }
            ControllerState::HangTime => {
                if self.state_elapsed_ms >= self.config.hang_time_ms {
                    self.state = ControllerState::Idle;
                    self.state_elapsed_ms = 0;
                    return Some(Action::SetPtt(false));
                }
            }
            _ => {}
        }

        None
    }
}
