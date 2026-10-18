//! ESP32 Pin Assignment Configurations for Audio I2S and Radio Lines

#[derive(Debug, Clone, Copy)]
pub struct PinConfig {
    // I2S Audio Bus
    pub i2s_bclk: i32,
    pub i2s_lrck: i32,
    pub i2s_dout: i32,
    pub i2s_din: i32,
    pub i2s_mclk: Option<i32>,

    // Radio Hardware Lines
    pub gpio_ptt: i32,
    pub gpio_cos: i32,
    pub cos_active_low: bool,
    pub ptt_active_high: bool,

    // Status LED
    pub gpio_led: Option<i32>,
}

impl Default for PinConfig {
    /// Default pin assignment matching common ESP32 audio / radio boards
    fn default() -> Self {
        Self {
            i2s_bclk: 27,
            i2s_lrck: 25,
            i2s_dout: 26,
            i2s_din: 35,
            i2s_mclk: Some(0),
            gpio_ptt: 4,
            gpio_cos: 5,
            cos_active_low: true,
            ptt_active_high: true,
            gpio_led: Some(2),
        }
    }
}
