//! # iax2-proto
//!
//! Pure Rust, zero-allocation parser and serializer for the IAX2 protocol (RFC 5456).
//! Designed for low-memory microcontrollers like the ESP32 and high-throughput server daemons.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod constants;
pub mod frame;
pub mod ie;

pub use constants::{audio_format, FrameType, IaxCommand};
pub use frame::{Frame, FullFrame, MiniFrame, ParseError};
pub use ie::{IeIterator, IeType, InformationElement};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_frame_roundtrip() {
        let payload = [0x01, 0x02, 0x03, 0x04];
        let original = FullFrame {
            source_call: 101,
            dest_call: 202,
            retransmission: false,
            timestamp: 123456,
            o_seqno: 1,
            i_seqno: 2,
            frame_type: FrameType::Iax,
            subclass: IaxCommand::New.into(),
            c_bit: false,
            payload: &payload,
        };

        let mut buffer = [0u8; 64];
        let written = original.serialize(&mut buffer).expect("serialization failed");
        assert_eq!(written, FullFrame::HEADER_SIZE + payload.len());

        let parsed = FullFrame::parse(&buffer[..written]).expect("parse failed");
        assert_eq!(parsed.source_call, 101);
        assert_eq!(parsed.dest_call, 202);
        assert_eq!(parsed.retransmission, false);
        assert_eq!(parsed.timestamp, 123456);
        assert_eq!(parsed.o_seqno, 1);
        assert_eq!(parsed.i_seqno, 2);
        assert_eq!(parsed.frame_type, FrameType::Iax);
        assert_eq!(parsed.subclass, IaxCommand::New.into());
        assert_eq!(parsed.c_bit, false);
        assert_eq!(parsed.payload, &payload);
    }

    #[test]
    fn test_mini_frame_roundtrip() {
        let audio_payload = [0x55, 0x55, 0xAA, 0xAA];
        let mini = MiniFrame {
            source_call: 350,
            timestamp_16: 0x4321,
            payload: &audio_payload,
        };

        let mut buffer = [0u8; 32];
        let written = mini.serialize(&mut buffer).expect("serialization failed");
        assert_eq!(written, MiniFrame::HEADER_SIZE + audio_payload.len());

        let frame = Frame::parse(&buffer[..written]).expect("parse failed");
        match frame {
            Frame::Mini(parsed) => {
                assert_eq!(parsed.source_call, 350);
                assert_eq!(parsed.timestamp_16, 0x4321);
                assert_eq!(parsed.payload, &audio_payload);
            }
            _ => panic!("Expected MiniFrame"),
        }
    }

    #[test]
    fn test_ie_iteration() {
        // IE: CALLED_NUMBER (1), len 4: "1234"
        // IE: USERNAME (6), len 5: "node1"
        let raw_ies: [u8; 13] = [
            1, 4, b'1', b'2', b'3', b'4',
            6, 5, b'n', b'o', b'd', b'e', b'1',
        ];

        let mut iter = IeIterator::new(&raw_ies);

        let first = iter.next().expect("missing first IE");
        assert_eq!(first.ie_type, IeType::CalledNumber);
        assert_eq!(first.as_str(), Some("1234"));

        let second = iter.next().expect("missing second IE");
        assert_eq!(second.ie_type, IeType::Username);
        assert_eq!(second.as_str(), Some("node1"));

        assert_eq!(iter.next(), None);
    }
}
