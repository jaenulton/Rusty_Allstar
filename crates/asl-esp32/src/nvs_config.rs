//! Non-Volatile Storage (NVS) Configuration for ESP32 Micro-Nodes
//!
//! Stores Wi-Fi credentials, AllStarLink node registration, and radio RF parameters.

#[derive(Debug, Clone, PartialEq)]
pub struct NodeNvsConfig {
    pub wifi_ssid: [u8; 32],
    pub wifi_ssid_len: usize,
    pub wifi_pass: [u8; 64],
    pub wifi_pass_len: usize,
    pub asl_node: u32,
    pub asl_password: [u8; 32],
    pub asl_pass_len: usize,
    pub tx_frequency_khz: u32,
    pub rx_frequency_khz: u32,
    pub ctcss_tone_tenths_hz: u16, // e.g. 1000 = 100.0 Hz
    pub squelch_level: u8,
    pub ptt_timeout_seconds: u16,
}

impl Default for NodeNvsConfig {
    fn default() -> Self {
        let mut cfg = Self {
            wifi_ssid: [0; 32],
            wifi_ssid_len: 0,
            wifi_pass: [0; 64],
            wifi_pass_len: 0,
            asl_node: 1999,
            asl_password: [0; 32],
            asl_pass_len: 0,
            tx_frequency_khz: 446_000,
            rx_frequency_khz: 446_000,
            ctcss_tone_tenths_hz: 1000,
            squelch_level: 4,
            ptt_timeout_seconds: 180,
        };
        cfg.set_wifi_ssid("AllStar-WiFi");
        cfg.set_asl_password("secret123");
        cfg
    }
}

impl NodeNvsConfig {
    pub fn set_wifi_ssid(&mut self, ssid: &str) {
        let bytes = ssid.as_bytes();
        let len = bytes.len().min(32);
        self.wifi_ssid[..len].copy_from_slice(&bytes[..len]);
        self.wifi_ssid_len = len;
    }

    pub fn wifi_ssid_str(&self) -> &str {
        core::str::from_utf8(&self.wifi_ssid[..self.wifi_ssid_len]).unwrap_or("")
    }

    pub fn set_wifi_pass(&mut self, pass: &str) {
        let bytes = pass.as_bytes();
        let len = bytes.len().min(64);
        self.wifi_pass[..len].copy_from_slice(&bytes[..len]);
        self.wifi_pass_len = len;
    }

    pub fn set_asl_password(&mut self, pass: &str) {
        let bytes = pass.as_bytes();
        let len = bytes.len().min(32);
        self.asl_password[..len].copy_from_slice(&bytes[..len]);
        self.asl_pass_len = len;
    }

    /// Serializes configuration into a binary blob for NVS storage partition
    pub fn serialize(&self, buf: &mut [u8]) -> Result<usize, ()> {
        if buf.len() < 160 {
            return Err(());
        }
        buf[0..4].copy_from_slice(b"ASL1"); // Magic marker
        buf[4..8].copy_from_slice(&self.asl_node.to_le_bytes());
        buf[8..12].copy_from_slice(&self.tx_frequency_khz.to_le_bytes());
        buf[12..16].copy_from_slice(&self.rx_frequency_khz.to_le_bytes());
        buf[16..18].copy_from_slice(&self.ctcss_tone_tenths_hz.to_le_bytes());
        buf[18] = self.squelch_level;
        buf[19..21].copy_from_slice(&self.ptt_timeout_seconds.to_le_bytes());
        buf[21] = self.wifi_ssid_len as u8;
        buf[22..54].copy_from_slice(&self.wifi_ssid);
        buf[54] = self.wifi_pass_len as u8;
        buf[55..119].copy_from_slice(&self.wifi_pass);
        buf[119] = self.asl_pass_len as u8;
        buf[120..152].copy_from_slice(&self.asl_password);
        Ok(152)
    }

    /// Deserializes configuration from binary NVS storage blob
    pub fn deserialize(buf: &[u8]) -> Result<Self, ()> {
        if buf.len() < 152 || &buf[0..4] != b"ASL1" {
            return Err(());
        }

        let asl_node = u32::from_le_bytes(buf[4..8].try_into().unwrap());
        let tx_frequency_khz = u32::from_le_bytes(buf[8..12].try_into().unwrap());
        let rx_frequency_khz = u32::from_le_bytes(buf[12..16].try_into().unwrap());
        let ctcss_tone_tenths_hz = u16::from_le_bytes(buf[16..18].try_into().unwrap());
        let squelch_level = buf[18];
        let ptt_timeout_seconds = u16::from_le_bytes(buf[19..21].try_into().unwrap());

        let wifi_ssid_len = (buf[21] as usize).min(32);
        let mut wifi_ssid = [0u8; 32];
        wifi_ssid.copy_from_slice(&buf[22..54]);

        let wifi_pass_len = (buf[54] as usize).min(64);
        let mut wifi_pass = [0u8; 64];
        wifi_pass.copy_from_slice(&buf[55..119]);

        let asl_pass_len = (buf[119] as usize).min(32);
        let mut asl_password = [0u8; 32];
        asl_password.copy_from_slice(&buf[120..152]);

        Ok(Self {
            wifi_ssid,
            wifi_ssid_len,
            wifi_pass,
            wifi_pass_len,
            asl_node,
            asl_password,
            asl_pass_len,
            tx_frequency_khz,
            rx_frequency_khz,
            ctcss_tone_tenths_hz,
            squelch_level,
            ptt_timeout_seconds,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nvs_serialization_roundtrip() {
        let mut orig = NodeNvsConfig::default();
        orig.set_wifi_ssid("MyRepeaterHub");
        orig.set_wifi_pass("super-secret-pass");
        orig.asl_node = 50012;
        orig.tx_frequency_khz = 146_520;
        orig.rx_frequency_khz = 146_520;
        orig.ctcss_tone_tenths_hz = 885; // 88.5 Hz

        let mut buf = [0u8; 256];
        let len = orig.serialize(&mut buf).expect("serialization failed");
        assert_eq!(len, 152);

        let recovered = NodeNvsConfig::deserialize(&buf[..len]).expect("deserialization failed");
        assert_eq!(recovered.wifi_ssid_str(), "MyRepeaterHub");
        assert_eq!(recovered.asl_node, 50012);
        assert_eq!(recovered.tx_frequency_khz, 146_520);
        assert_eq!(recovered.ctcss_tone_tenths_hz, 885);
    }
}
