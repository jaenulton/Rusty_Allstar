#![allow(clippy::field_reassign_with_default)]
//! Interactive Terminal User Interface (TUI) Dashboard for Rusty_Allstar Node
//!
//! Renders real-time telemetry, COS / PTT indicators, audio VU meter, and node state
//! directly in terminal consoles.

pub struct TuiDashboardState {
    pub callsign: String,
    pub local_node: String,
    pub connected_node: Option<String>,
    pub link_state: String,
    pub cos_active: bool,
    pub ptt_active: bool,
    pub rf_freq: String,
    pub squelch_level: u8,
    pub audio_peak_pct: u8,
}

impl Default for TuiDashboardState {
    fn default() -> Self {
        Self {
            callsign: "W1AW".to_string(),
            local_node: "1999".to_string(),
            connected_node: None,
            link_state: "IDLE".to_string(),
            cos_active: false,
            ptt_active: false,
            rf_freq: "446.0000".to_string(),
            squelch_level: 4,
            audio_peak_pct: 0,
        }
    }
}

pub struct TerminalRenderer;

impl TerminalRenderer {
    /// Renders an ANSI formatted text console dashboard snapshot
    pub fn render_frame(state: &TuiDashboardState) -> String {
        let mut out = String::new();

        let cos_badge = if state.cos_active {
            "\x1b[1;32m[● COS ACTIVE (RX)]\x1b[0m"
        } else {
            "\x1b[90m[○ COS IDLE]\x1b[0m"
        };

        let ptt_badge = if state.ptt_active {
            "\x1b[1;31m[● PTT KEYED (TX)]\x1b[0m"
        } else {
            "\x1b[90m[○ PTT UNKEYED]\x1b[0m"
        };

        let target_str = state.connected_node.as_deref().unwrap_or("DISCONNECTED");

        // Build VU meter bar (20 characters)
        let filled_bars = (state.audio_peak_pct as usize * 20) / 100;
        let mut vu_meter = String::with_capacity(20);
        for i in 0..20 {
            if i < filled_bars {
                if i < 14 {
                    vu_meter.push_str("\x1b[32m█\x1b[0m"); // Green
                } else if i < 18 {
                    vu_meter.push_str("\x1b[33m█\x1b[0m"); // Yellow
                } else {
                    vu_meter.push_str("\x1b[31m█\x1b[0m"); // Red
                }
            } else {
                vu_meter.push_str("\x1b[90m░\x1b[0m");
            }
        }

        out.push_str("┌────────────────────────────────────────────────────────┐\n");
        out.push_str("│          📻  RUSTY_ALLSTAR CONSOLE DASHBOARD           │\n");
        out.push_str("├────────────────────────────┬───────────────────────────┤\n");
        out.push_str(&format!(
            "│ Callsign:   {:<14} │ Local Node: {:<13} │\n",
            state.callsign, state.local_node
        ));
        out.push_str(&format!(
            "│ Link State: {:<14} │ Target:     {:<13} │\n",
            state.link_state, target_str
        ));
        out.push_str(&format!(
            "│ Transceiver:{:<14} │ Squelch:    Level {:<7} │\n",
            state.rf_freq, state.squelch_level
        ));
        out.push_str("├────────────────────────────┴───────────────────────────┤\n");
        out.push_str(&format!("│ Status: {}   {} │\n", cos_badge, ptt_badge));
        out.push_str(&format!(
            "│ Audio:  [{}] {:>3}%                      │\n",
            vu_meter, state.audio_peak_pct
        ));
        out.push_str("└────────────────────────────────────────────────────────┘\n");

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_renderer_snapshot() {
        let mut state = TuiDashboardState::default();
        state.cos_active = true;
        state.audio_peak_pct = 75;
        state.connected_node = Some("50000".to_string());

        let frame = TerminalRenderer::render_frame(&state);
        assert!(frame.contains("RUSTY_ALLSTAR CONSOLE DASHBOARD"));
        assert!(frame.contains("COS ACTIVE"));
        assert!(frame.contains("50000"));
    }
}
