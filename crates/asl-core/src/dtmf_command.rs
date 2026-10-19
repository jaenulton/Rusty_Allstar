//! AllStarLink / app_rpt DTMF Command Processor & Telemetry Dispatcher
//!
//! Parses incoming DTMF digit streams into AllStarLink control commands:
//! - `*1<node>`: Disconnect link
//! - `*2<node>`: Connect link (monitor mode / receive-only)
//! - `*3<node>`: Connect link (transceive mode)
//! - `*70`: Node status & connection query
//! - `*80`: Force station CW ID
//! - `*81`: Time / telemetry report

use heapless::String as HString;

/// Standard AllStarLink command parsed from DTMF stream
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DtmfCommand {
    /// Disconnect remote link (*1<node> or *10 for disconnect all)
    Disconnect { target_node: Option<u32> },
    /// Connect in monitor / listen-only mode (*2<node>)
    ConnectMonitor { target_node: u32 },
    /// Connect in normal transceive mode (*3<node>)
    ConnectTransceive { target_node: u32 },
    /// Status check (*70)
    SystemStatus,
    /// Manually trigger Morse code station ID (*80)
    ForceCwId,
    /// Announce current time / telemetry (*81)
    SayTime,
    /// Custom / user-defined command
    Custom(HString<16>),
}

/// DTMF Command Processor with configurable inter-digit timeout and buffer management
#[derive(Debug)]
pub struct DtmfCommandProcessor {
    buffer: HString<16>,
    inter_digit_timeout_ms: u32,
    current_digit_idle_ms: u32,
}

impl Default for DtmfCommandProcessor {
    fn default() -> Self {
        Self::new(3000) // 3-second inter-digit timeout
    }
}

impl DtmfCommandProcessor {
    pub fn new(inter_digit_timeout_ms: u32) -> Self {
        Self {
            buffer: HString::new(),
            inter_digit_timeout_ms,
            current_digit_idle_ms: 0,
        }
    }

    /// Feeds a detected DTMF digit into the command processor buffer.
    /// Returns Some(DtmfCommand) if a complete command was recognized immediately.
    pub fn push_digit(&mut self, digit: char) -> Option<DtmfCommand> {
        self.current_digit_idle_ms = 0;

        // '#' clears the current command buffer
        if digit == '#' {
            self.buffer.clear();
            return None;
        }

        // Add digit to buffer if capacity permits
        let _ = self.buffer.push(digit);

        // Check if buffer matches known zero-argument commands immediately
        if self.buffer.as_str() == "*70" {
            self.buffer.clear();
            return Some(DtmfCommand::SystemStatus);
        } else if self.buffer.as_str() == "*80" {
            self.buffer.clear();
            return Some(DtmfCommand::ForceCwId);
        } else if self.buffer.as_str() == "*81" {
            self.buffer.clear();
            return Some(DtmfCommand::SayTime);
        } else if self.buffer.as_str() == "*10" {
            self.buffer.clear();
            return Some(DtmfCommand::Disconnect { target_node: None });
        }

        None
    }

    /// Advance time by elapsed milliseconds.
    /// Evaluates variable-length node commands (e.g. `*350000`, `*11999`) on inter-digit timeout.
    pub fn tick(&mut self, elapsed_ms: u32) -> Option<DtmfCommand> {
        if self.buffer.is_empty() {
            return None;
        }

        self.current_digit_idle_ms = self.current_digit_idle_ms.saturating_add(elapsed_ms);

        if self.current_digit_idle_ms >= self.inter_digit_timeout_ms {
            let cmd = self.parse_buffer();
            self.buffer.clear();
            self.current_digit_idle_ms = 0;
            return cmd;
        }

        None
    }

    /// Current unparsed buffer contents
    pub fn current_buffer(&self) -> &str {
        self.buffer.as_str()
    }

    /// Force parse and clear current buffer
    pub fn flush(&mut self) -> Option<DtmfCommand> {
        if self.buffer.is_empty() {
            return None;
        }
        let cmd = self.parse_buffer();
        self.buffer.clear();
        self.current_digit_idle_ms = 0;
        cmd
    }

    fn parse_buffer(&self) -> Option<DtmfCommand> {
        let s = self.buffer.as_str();

        // Disconnect: *1<node>
        if s.starts_with("*1") {
            let remainder = &s[2..];
            if remainder.is_empty() || remainder == "0" {
                return Some(DtmfCommand::Disconnect { target_node: None });
            } else if let Ok(node) = remainder.parse::<u32>() {
                return Some(DtmfCommand::Disconnect { target_node: Some(node) });
            }
        }
        // Monitor connect: *2<node>
        else if s.starts_with("*2") {
            let remainder = &s[2..];
            if let Ok(node) = remainder.parse::<u32>() {
                return Some(DtmfCommand::ConnectMonitor { target_node: node });
            }
        }
        // Transceive connect: *3<node>
        else if s.starts_with("*3") {
            let remainder = &s[2..];
            if let Ok(node) = remainder.parse::<u32>() {
                return Some(DtmfCommand::ConnectTransceive { target_node: node });
            }
        }

        if !s.is_empty() {
            let mut custom = HString::new();
            let _ = custom.push_str(s);
            Some(DtmfCommand::Custom(custom))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_immediate_commands() {
        let mut proc = DtmfCommandProcessor::default();

        assert_eq!(proc.push_digit('*'), None);
        assert_eq!(proc.push_digit('7'), None);
        assert_eq!(proc.push_digit('0'), Some(DtmfCommand::SystemStatus));
        assert_eq!(proc.current_buffer(), "");

        assert_eq!(proc.push_digit('*'), None);
        assert_eq!(proc.push_digit('8'), None);
        assert_eq!(proc.push_digit('0'), Some(DtmfCommand::ForceCwId));

        assert_eq!(proc.push_digit('*'), None);
        assert_eq!(proc.push_digit('1'), None);
        assert_eq!(proc.push_digit('0'), Some(DtmfCommand::Disconnect { target_node: None }));
    }

    #[test]
    fn test_connect_with_timeout() {
        let mut proc = DtmfCommandProcessor::new(1000);

        for ch in "*350000".chars() {
            assert_eq!(proc.push_digit(ch), None);
        }
        assert_eq!(proc.current_buffer(), "*350000");

        // Advance 500ms (not yet timeout)
        assert_eq!(proc.tick(500), None);

        // Advance another 600ms (total 1100ms >= 1000ms timeout)
        let cmd = proc.tick(600);
        assert_eq!(cmd, Some(DtmfCommand::ConnectTransceive { target_node: 50000 }));
        assert_eq!(proc.current_buffer(), "");
    }

    #[test]
    fn test_disconnect_target_node() {
        let mut proc = DtmfCommandProcessor::new(500);

        for ch in "*12000".chars() {
            proc.push_digit(ch);
        }

        let cmd = proc.tick(600);
        assert_eq!(cmd, Some(DtmfCommand::Disconnect { target_node: Some(2000) }));
    }

    #[test]
    fn test_pound_clears_buffer() {
        let mut proc = DtmfCommandProcessor::new(1000);
        proc.push_digit('*');
        proc.push_digit('3');
        proc.push_digit('#');
        assert_eq!(proc.current_buffer(), "");
        assert_eq!(proc.tick(1200), None);
    }
}
