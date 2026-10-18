//! IAX2 Authentication helpers (MD5 challenge response)

use md5::{Digest, Md5};

/// Compute IAX2 MD5 response for a challenge and node secret/password:
/// RFC 5456 Section 8.6.15: MD5(challenge string + password string)
/// Returned as a lowercase 32-character hexadecimal string.
pub fn compute_md5_response(challenge: &str, secret: &str) -> [u8; 32] {
    let mut hasher = Md5::new();
    hasher.update(challenge.as_bytes());
    hasher.update(secret.as_bytes());
    let result = hasher.finalize();

    let mut hex_buf = [0u8; 32];
    hex::encode_to_slice(result, &mut hex_buf).expect("hex encode into 32-byte buffer");
    hex_buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_md5_response() {
        let challenge = "12345678";
        let secret = "secret_password";
        let resp = compute_md5_response(challenge, secret);
        assert_eq!(resp.len(), 32);

        // Verify known hash using digest
        let mut hasher = Md5::new();
        hasher.update(b"12345678secret_password");
        let expected = hasher.finalize();
        let mut expected_hex = [0u8; 32];
        hex::encode_to_slice(expected, &mut expected_hex).unwrap();

        assert_eq!(resp, expected_hex);
    }
}
