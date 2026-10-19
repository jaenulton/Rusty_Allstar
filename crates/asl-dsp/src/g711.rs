//! ITU-T G.711 μ-law (ulaw) and A-law audio codecs
//! Optimized for embedded execution on microcontrollers like the ESP32.

const BIAS: i32 = 0x84;
const CLIP: i32 = 32635;

/// Linear 16-bit PCM to 8-bit μ-law (ulaw)
pub fn linear_to_ulaw(pcm_val: i16) -> u8 {
    let mut pcm = pcm_val as i32;
    let sign = if pcm < 0 {
        pcm = -pcm;
        0x7F
    } else {
        0xFF
    };

    if pcm > CLIP {
        pcm = CLIP;
    }
    pcm += BIAS;

    let exponent = match pcm {
        0x4000..=i32::MAX => 7,
        0x2000..=0x3FFF => 6,
        0x1000..=0x1FFF => 5,
        0x0800..=0x0FFF => 4,
        0x0400..=0x07FF => 3,
        0x0200..=0x03FF => 2,
        0x0100..=0x01FF => 1,
        _ => 0,
    };

    let mantissa = (pcm >> (exponent + 3)) & 0x0F;
    let ulaw_byte = ((exponent << 4) | mantissa) as u8;

    ulaw_byte ^ sign
}

/// 8-bit μ-law to Linear 16-bit PCM
pub fn ulaw_to_linear(ulaw_byte: u8) -> i16 {
    let ulaw = (!ulaw_byte) as i32;
    let sign = ulaw & 0x80;
    let exponent = (ulaw >> 4) & 0x07;
    let mantissa = ulaw & 0x0F;

    let mut sample = ((mantissa << 3) + BIAS) << exponent;
    sample -= BIAS;

    if sign != 0 {
        (-sample) as i16
    } else {
        sample as i16
    }
}

/// Batch encode a slice of PCM samples to μ-law
pub fn encode_ulaw_slice(pcm: &[i16], out: &mut [u8]) {
    let count = pcm.len().min(out.len());
    for i in 0..count {
        out[i] = linear_to_ulaw(pcm[i]);
    }
}

/// Batch decode a slice of μ-law samples to PCM
pub fn decode_ulaw_slice(ulaw: &[u8], out: &mut [i16]) {
    let count = ulaw.len().min(out.len());
    for i in 0..count {
        out[i] = ulaw_to_linear(ulaw[i]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ulaw_roundtrip() {
        // Zero
        let enc0 = linear_to_ulaw(0);
        let dec0 = ulaw_to_linear(enc0);
        assert!(dec0.abs() < 10);

        // Typical voice amplitude
        let val = 12000i16;
        let enc = linear_to_ulaw(val);
        let dec = ulaw_to_linear(enc);
        // G.711 is lossy, should be within quantization step
        assert!((dec - val).abs() < 256);

        // Negative amplitude
        let neg_val = -15000i16;
        let enc_neg = linear_to_ulaw(neg_val);
        let dec_neg = ulaw_to_linear(enc_neg);
        assert!((dec_neg - neg_val).abs() < 256);
    }
}
