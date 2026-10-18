//! IAX2 Frame Definitions and Zero-Copy Parsers (RFC 5456)

use crate::constants::FrameType;
use crate::ie::IeIterator;

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    BufferTooShort,
    InvalidFullFrameMarker,
    MalformedData,
}

/// IAX2 Full Frame (RFC 5456 Section 8.1)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FullFrame<'a> {
    pub source_call: u16,
    pub dest_call: u16,
    pub retransmission: bool,
    pub timestamp: u32,
    pub o_seqno: u8,
    pub i_seqno: u8,
    pub frame_type: FrameType,
    pub subclass: u8,
    pub c_bit: bool,
    pub payload: &'a [u8],
}

impl<'a> FullFrame<'a> {
    pub const HEADER_SIZE: usize = 12;

    /// Parse a full frame from a raw UDP byte buffer
    pub fn parse(buffer: &'a [u8]) -> Result<Self, ParseError> {
        if buffer.len() < Self::HEADER_SIZE {
            return Err(ParseError::BufferTooShort);
        }

        let first_word = u16::from_be_bytes([buffer[0], buffer[1]]);
        if (first_word & 0x8000) == 0 {
            return Err(ParseError::InvalidFullFrameMarker);
        }
        let source_call = first_word & 0x7FFF;

        let second_word = u16::from_be_bytes([buffer[2], buffer[3]]);
        let retransmission = (second_word & 0x8000) != 0;
        let dest_call = second_word & 0x7FFF;

        let timestamp = u32::from_be_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]);
        let o_seqno = buffer[8];
        let i_seqno = buffer[9];
        let frame_type = FrameType::from(buffer[10]);

        let subclass_byte = buffer[11];
        let c_bit = (subclass_byte & 0x80) != 0;
        let subclass = subclass_byte & 0x7F;

        Ok(FullFrame {
            source_call,
            dest_call,
            retransmission,
            timestamp,
            o_seqno,
            i_seqno,
            frame_type,
            subclass,
            c_bit,
            payload: &buffer[Self::HEADER_SIZE..],
        })
    }

    /// Serialize this full frame into a output buffer.
    /// Returns number of bytes written.
    pub fn serialize(&self, out: &mut [u8]) -> Result<usize, ParseError> {
        let total_size = Self::HEADER_SIZE + self.payload.len();
        if out.len() < total_size {
            return Err(ParseError::BufferTooShort);
        }

        // Word 0: [1 (1 bit)][source_call (15 bits)]
        let first_word = 0x8000 | (self.source_call & 0x7FFF);
        out[0..2].copy_from_slice(&first_word.to_be_bytes());

        // Word 1: [R (1 bit)][dest_call (15 bits)]
        let r_flag = if self.retransmission { 0x8000 } else { 0 };
        let second_word = r_flag | (self.dest_call & 0x7FFF);
        out[2..4].copy_from_slice(&second_word.to_be_bytes());

        // Timestamp (32-bit big endian)
        out[4..8].copy_from_slice(&self.timestamp.to_be_bytes());

        // OSeqno and ISeqno
        out[8] = self.o_seqno;
        out[9] = self.i_seqno;

        // Frame Type
        out[10] = self.frame_type.into();

        // Subclass and C-bit
        let c_flag = if self.c_bit { 0x80 } else { 0 };
        out[11] = c_flag | (self.subclass & 0x7F);

        // Payload
        if !self.payload.is_empty() {
            out[Self::HEADER_SIZE..total_size].copy_from_slice(self.payload);
        }

        Ok(total_size)
    }

    /// Return an iterator over Information Elements in the payload
    pub fn ies(&self) -> IeIterator<'a> {
        IeIterator::new(self.payload)
    }
}

/// IAX2 Mini Frame (RFC 5456 Section 8.3)
/// High-efficiency 4-byte header used for sequential voice packets after call establishment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MiniFrame<'a> {
    pub source_call: u16,
    pub timestamp_16: u16,
    pub payload: &'a [u8],
}

impl<'a> MiniFrame<'a> {
    pub const HEADER_SIZE: usize = 4;

    pub fn parse(buffer: &'a [u8]) -> Result<Self, ParseError> {
        if buffer.len() < Self::HEADER_SIZE {
            return Err(ParseError::BufferTooShort);
        }

        let first_word = u16::from_be_bytes([buffer[0], buffer[1]]);
        if (first_word & 0x8000) != 0 {
            return Err(ParseError::InvalidFullFrameMarker);
        }
        let source_call = first_word & 0x7FFF;
        let timestamp_16 = u16::from_be_bytes([buffer[2], buffer[3]]);

        Ok(MiniFrame {
            source_call,
            timestamp_16,
            payload: &buffer[Self::HEADER_SIZE..],
        })
    }

    pub fn serialize(&self, out: &mut [u8]) -> Result<usize, ParseError> {
        let total_size = Self::HEADER_SIZE + self.payload.len();
        if out.len() < total_size {
            return Err(ParseError::BufferTooShort);
        }

        // Word 0: [0 (1 bit)][source_call (15 bits)]
        let first_word = self.source_call & 0x7FFF;
        out[0..2].copy_from_slice(&first_word.to_be_bytes());
        out[2..4].copy_from_slice(&self.timestamp_16.to_be_bytes());

        if !self.payload.is_empty() {
            out[Self::HEADER_SIZE..total_size].copy_from_slice(self.payload);
        }

        Ok(total_size)
    }
}

/// Top-level parsed IAX2 Frame
#[derive(Debug, PartialEq, Eq)]
pub enum Frame<'a> {
    Full(FullFrame<'a>),
    Mini(MiniFrame<'a>),
    Meta(&'a [u8]),
}

impl<'a> Frame<'a> {
    pub fn parse(buffer: &'a [u8]) -> Result<Self, ParseError> {
        if buffer.len() < 4 {
            return Err(ParseError::BufferTooShort);
        }

        let first_word = u16::from_be_bytes([buffer[0], buffer[1]]);
        if (first_word & 0x8000) != 0 {
            FullFrame::parse(buffer).map(Frame::Full)
        } else if first_word == 0 {
            Ok(Frame::Meta(buffer))
        } else {
            MiniFrame::parse(buffer).map(Frame::Mini)
        }
    }
}
