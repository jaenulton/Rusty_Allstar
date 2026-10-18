//! NiceRF SA818 / SR-FRS Transceiver Module UART Driver
//! Enables ESP32 to program VHF (134-174 MHz) or UHF (400-480 MHz) frequencies,
//! channel spacing, squelch levels, and CTCSS/CDCSS sub-audible tones over serial.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bandwidth {
    Narrow = 0, // 12.5 kHz
    Wide = 1,   // 25.0 kHz
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmphasisFilter {
    Bypass = 1,
    Enable = 0,
}

#[derive(Debug, Clone)]
pub struct Sa818Config {
    pub bandwidth: Bandwidth,
    /// Transmit Frequency in MHz (e.g. "146.5200" or "446.0000")
    pub tx_freq_mhz: &'static str,
    /// Receive Frequency in MHz
    pub rx_freq_mhz: &'static str,
    /// Squelch level (0 to 8)
    pub squelch_level: u8,
    /// TX CTCSS tone in Hz (0.0 for none)
    pub tx_ctcss: f32,
    /// RX CTCSS tone in Hz (0.0 for none)
    pub rx_ctcss: f32,
    /// Audio volume (1 to 8)
    pub volume: u8,
}

impl Default for Sa818Config {
    fn default() -> Self {
        Self {
            bandwidth: Bandwidth::Narrow,
            tx_freq_mhz: "446.0000",
            rx_freq_mhz: "446.0000",
            squelch_level: 4,
            tx_ctcss: 100.0,
            rx_ctcss: 100.0,
            volume: 5,
        }
    }
}

pub struct Sa818Driver;

impl Sa818Driver {
    /// Generate handshake command: `AT+DMOCONNECT\r\n`
    pub fn cmd_connect(out: &mut [u8]) -> usize {
        const CMD: &[u8] = b"AT+DMOCONNECT\r\n";
        out[..CMD.len()].copy_from_slice(CMD);
        CMD.len()
    }

    /// Generate group tuning command:
    /// Format: `AT+DMOSETGROUP=BW,TFV,RFV,TxCX,SQ,RxCX\r\n`
    pub fn cmd_set_group(config: &Sa818Config, out: &mut [u8]) -> usize {
        let bw = config.bandwidth as u8;
        let sq = config.squelch_level.min(8);

        // Format CTCSS as 4-digit fixed point string (e.g. "0100" for 100.0 Hz or "0000" for none)
        let tx_cx = format_ctcss(config.tx_ctcss);
        let rx_cx = format_ctcss(config.rx_ctcss);

        let s = format!(
            "AT+DMOSETGROUP={},{},{},{},{},{}\r\n",
            bw, config.tx_freq_mhz, config.rx_freq_mhz, tx_cx, sq, rx_cx
        );
        let bytes = s.as_bytes();
        out[..bytes.len()].copy_from_slice(bytes);
        bytes.len()
    }

    /// Generate volume command: `AT+DMOSETVOLUME=x\r\n` (1-8)
    pub fn cmd_set_volume(level: u8, out: &mut [u8]) -> usize {
        let vol = level.clamp(1, 8);
        let s = format!("AT+DMOSETVOLUME={}\r\n", vol);
        let bytes = s.as_bytes();
        out[..bytes.len()].copy_from_slice(bytes);
        bytes.len()
    }

    /// Generate filter command: `AT+SETFILTER=EMPH,HPF,LPF\r\n`
    pub fn cmd_set_filter(emphasis: EmphasisFilter, out: &mut [u8]) -> usize {
        let emph = emphasis as u8;
        // Bypassing internal HPF/LPF is ideal when ASL software DSP handles filtering
        let s = format!("AT+SETFILTER={},1,1\r\n", emph);
        let bytes = s.as_bytes();
        out[..bytes.len()].copy_from_slice(bytes);
        bytes.len()
    }

    /// Parse SA818 module response:
    /// "+DMOCONNECT:0" -> Success (0 = ok, 1 = fail)
    /// "+DMOSETGROUP:0" -> Success
    pub fn parse_response(resp: &str) -> bool {
        resp.contains(":0")
    }
}

fn format_ctcss(freq: f32) -> String {
    if freq < 60.0 {
        "0000".to_string()
    } else {
        // e.g. 100.0 Hz -> "0100" or 67.0 Hz -> "0067"
        let val = freq as u32;
        format!("{:04}", val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sa818_commands() {
        let mut buf = [0u8; 128];

        // 1. Handshake
        let len = Sa818Driver::cmd_connect(&mut buf);
        let s = core::str::from_utf8(&buf[..len]).unwrap();
        assert_eq!(s, "AT+DMOCONNECT\r\n");

        // 2. Set Group
        let config = Sa818Config {
            bandwidth: Bandwidth::Narrow,
            tx_freq_mhz: "146.5200",
            rx_freq_mhz: "146.5200",
            squelch_level: 3,
            tx_ctcss: 100.0,
            rx_ctcss: 100.0,
            volume: 6,
        };
        let len = Sa818Driver::cmd_set_group(&config, &mut buf);
        let s = core::str::from_utf8(&buf[..len]).unwrap();
        assert_eq!(s, "AT+DMOSETGROUP=0,146.5200,146.5200,0100,3,0100\r\n");

        // 3. Response validation
        assert!(Sa818Driver::parse_response("+DMOCONNECT:0\r\n"));
        assert!(!Sa818Driver::parse_response("+DMOCONNECT:1\r\n"));
    }
}
