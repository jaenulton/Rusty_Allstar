//! IAX2 Call Session State Machine and Frame Builder

use crate::auth::compute_md5_response;
use iax2_proto::constants::{audio_format, FrameType, IaxCommand};
use iax2_proto::frame::{FullFrame, MiniFrame, ParseError};
use iax2_proto::ie::IeType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Idle,
    Connecting,
    Authenticating,
    Connected,
    HangingUp,
    Terminated,
}

pub struct CallSession {
    pub local_call_no: u16,
    pub remote_call_no: u16,
    pub state: SessionState,
    pub o_seqno: u8,
    pub i_seqno: u8,
    pub timestamp_ms: u32,
}

impl CallSession {
    pub fn new(local_call_no: u16) -> Self {
        Self {
            local_call_no,
            remote_call_no: 0,
            state: SessionState::Idle,
            o_seqno: 0,
            i_seqno: 0,
            timestamp_ms: 0,
        }
    }

    /// Advance timestamp by elapsed milliseconds
    pub fn advance_time(&mut self, elapsed_ms: u32) {
        self.timestamp_ms = self.timestamp_ms.wrapping_add(elapsed_ms);
    }

    /// Build a `NEW` call full frame to initiate linking to a remote AllStar node
    pub fn build_new(
        &mut self,
        called_node: &str,
        calling_node: &str,
        username: &str,
        out: &mut [u8],
    ) -> Result<usize, ParseError> {
        let mut ie_buf = [0u8; 128];
        let mut ie_len = 0;

        // CALLED_NUMBER
        ie_len += write_ie(IeType::CalledNumber, called_node.as_bytes(), &mut ie_buf[ie_len..])?;
        // CALLING_NUMBER
        ie_len += write_ie(IeType::CallingNumber, calling_node.as_bytes(), &mut ie_buf[ie_len..])?;
        // USERNAME
        ie_len += write_ie(IeType::Username, username.as_bytes(), &mut ie_buf[ie_len..])?;
        // FORMAT = G.711 u-law (4 bytes)
        let format_bytes = audio_format::ULAW.to_be_bytes();
        ie_len += write_ie(IeType::Format, &format_bytes, &mut ie_buf[ie_len..])?;
        // CAPABILITY = G.711 u-law (4 bytes)
        ie_len += write_ie(IeType::Capability, &format_bytes, &mut ie_buf[ie_len..])?;

        let frame = FullFrame {
            source_call: self.local_call_no,
            dest_call: 0,
            retransmission: false,
            timestamp: self.timestamp_ms,
            o_seqno: self.o_seqno,
            i_seqno: self.i_seqno,
            frame_type: FrameType::Iax,
            subclass: IaxCommand::New.into(),
            c_bit: false,
            payload: &ie_buf[..ie_len],
        };

        self.o_seqno = self.o_seqno.wrapping_add(1);
        self.state = SessionState::Connecting;
        frame.serialize(out)
    }

    /// Build an `AUTHREP` full frame responding to an `AUTHREQ` MD5 challenge
    pub fn build_auth_rep(
        &mut self,
        secret: &str,
        challenge: &str,
        out: &mut [u8],
    ) -> Result<usize, ParseError> {
        let md5_hash = compute_md5_response(challenge, secret);

        let mut ie_buf = [0u8; 64];
        let mut ie_len = 0;
        ie_len += write_ie(IeType::Md5Result, &md5_hash, &mut ie_buf[ie_len..])?;

        let frame = FullFrame {
            source_call: self.local_call_no,
            dest_call: self.remote_call_no,
            retransmission: false,
            timestamp: self.timestamp_ms,
            o_seqno: self.o_seqno,
            i_seqno: self.i_seqno,
            frame_type: FrameType::Iax,
            subclass: IaxCommand::AuthRep.into(),
            c_bit: false,
            payload: &ie_buf[..ie_len],
        };

        self.o_seqno = self.o_seqno.wrapping_add(1);
        self.state = SessionState::Authenticating;
        frame.serialize(out)
    }

    /// Build an `ACK` response acknowledging a received full frame
    pub fn build_ack(&self, target_timestamp: u32, out: &mut [u8]) -> Result<usize, ParseError> {
        let frame = FullFrame {
            source_call: self.local_call_no,
            dest_call: self.remote_call_no,
            retransmission: false,
            timestamp: target_timestamp,
            o_seqno: self.o_seqno,
            i_seqno: self.i_seqno,
            frame_type: FrameType::Iax,
            subclass: IaxCommand::Ack.into(),
            c_bit: false,
            payload: &[],
        };
        frame.serialize(out)
    }

    /// Build a `HANGUP` frame to close the connection
    pub fn build_hangup(&mut self, cause_code: u8, out: &mut [u8]) -> Result<usize, ParseError> {
        let mut ie_buf = [0u8; 8];
        let ie_len = write_ie(IeType::Cause, &[cause_code], &mut ie_buf)?;

        let frame = FullFrame {
            source_call: self.local_call_no,
            dest_call: self.remote_call_no,
            retransmission: false,
            timestamp: self.timestamp_ms,
            o_seqno: self.o_seqno,
            i_seqno: self.i_seqno,
            frame_type: FrameType::Iax,
            subclass: IaxCommand::Hangup.into(),
            c_bit: false,
            payload: &ie_buf[..ie_len],
        };

        self.o_seqno = self.o_seqno.wrapping_add(1);
        self.state = SessionState::HangingUp;
        frame.serialize(out)
    }

    /// Build a `PING` keepalive frame
    pub fn build_ping(&mut self, out: &mut [u8]) -> Result<usize, ParseError> {
        let frame = FullFrame {
            source_call: self.local_call_no,
            dest_call: self.remote_call_no,
            retransmission: false,
            timestamp: self.timestamp_ms,
            o_seqno: self.o_seqno,
            i_seqno: self.i_seqno,
            frame_type: FrameType::Iax,
            subclass: IaxCommand::Ping.into(),
            c_bit: false,
            payload: &[],
        };
        self.o_seqno = self.o_seqno.wrapping_add(1);
        frame.serialize(out)
    }

    /// Build a `PONG` response
    pub fn build_pong(&self, target_timestamp: u32, out: &mut [u8]) -> Result<usize, ParseError> {
        let frame = FullFrame {
            source_call: self.local_call_no,
            dest_call: self.remote_call_no,
            retransmission: false,
            timestamp: target_timestamp,
            o_seqno: self.o_seqno,
            i_seqno: self.i_seqno,
            frame_type: FrameType::Iax,
            subclass: IaxCommand::Pong.into(),
            c_bit: false,
            payload: &[],
        };
        frame.serialize(out)
    }

    /// Build a high-efficiency 4-byte `MiniFrame` containing voice samples (e.g. 160 bytes G.711)
    pub fn build_voice_mini(
        &self,
        audio_payload: &[u8],
        out: &mut [u8],
    ) -> Result<usize, ParseError> {
        let mini = MiniFrame {
            source_call: self.local_call_no,
            timestamp_16: (self.timestamp_ms & 0xFFFF) as u16,
            payload: audio_payload,
        };
        mini.serialize(out)
    }

    /// Process an incoming Full Frame from remote node
    pub fn process_incoming_full(&mut self, frame: &FullFrame) {
        if self.remote_call_no == 0 {
            self.remote_call_no = frame.source_call;
        }

        // Update incoming sequence count
        self.i_seqno = frame.o_seqno.wrapping_add(1);

        if frame.frame_type == FrameType::Iax {
            match IaxCommand::from(frame.subclass) {
                IaxCommand::Accept => {
                    self.state = SessionState::Connected;
                }
                IaxCommand::Hangup | IaxCommand::Reject => {
                    self.state = SessionState::Terminated;
                }
                _ => {}
            }
        }
    }
}

fn write_ie(ie_type: IeType, data: &[u8], out: &mut [u8]) -> Result<usize, ParseError> {
    if out.len() < 2 + data.len() {
        return Err(ParseError::BufferTooShort);
    }
    out[0] = ie_type.into();
    out[1] = data.len() as u8;
    out[2..2 + data.len()].copy_from_slice(data);
    Ok(2 + data.len())
}
