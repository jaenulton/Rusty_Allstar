//! AllStarLink Node Registration Client (REGREQ / REGAUTH / REGACK)

use crate::auth::compute_md5_response;
use iax2_proto::constants::{FrameType, IaxCommand};
use iax2_proto::frame::{FullFrame, ParseError};
use iax2_proto::ie::IeType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegState {
    Unregistered,
    RequestSent,
    Authenticating,
    Registered,
    Rejected,
}

pub struct RegistrationClient {
    pub node_number: &'static str,
    pub password: &'static str,
    pub refresh_secs: u16,
    pub local_call_no: u16,
    pub remote_call_no: u16,
    pub state: RegState,
    pub o_seqno: u8,
    pub i_seqno: u8,
    pub timestamp_ms: u32,
}

impl RegistrationClient {
    pub fn new(
        node_number: &'static str,
        password: &'static str,
        refresh_secs: u16,
        local_call_no: u16,
    ) -> Self {
        Self {
            node_number,
            password,
            refresh_secs,
            local_call_no,
            remote_call_no: 0,
            state: RegState::Unregistered,
            o_seqno: 0,
            i_seqno: 0,
            timestamp_ms: 0,
        }
    }

    /// Build initial `REGREQ` frame
    pub fn build_reg_req(&mut self, out: &mut [u8]) -> Result<usize, ParseError> {
        let mut ie_buf = [0u8; 64];
        let mut ie_len = 0;

        // USERNAME = node number
        ie_len += write_ie(
            IeType::Username,
            self.node_number.as_bytes(),
            &mut ie_buf[ie_len..],
        )?;
        // REFRESH = refresh_secs (2 bytes BE)
        let refresh_bytes = self.refresh_secs.to_be_bytes();
        ie_len += write_ie(IeType::Refresh, &refresh_bytes, &mut ie_buf[ie_len..])?;

        let frame = FullFrame {
            source_call: self.local_call_no,
            dest_call: self.remote_call_no,
            retransmission: false,
            timestamp: self.timestamp_ms,
            o_seqno: self.o_seqno,
            i_seqno: self.i_seqno,
            frame_type: FrameType::Iax,
            subclass: IaxCommand::RegReq.into(),
            c_bit: false,
            payload: &ie_buf[..ie_len],
        };

        self.o_seqno = self.o_seqno.wrapping_add(1);
        self.state = RegState::RequestSent;
        frame.serialize(out)
    }

    /// Build authenticated `REGREQ` response after receiving `REGAUTH` challenge
    pub fn build_reg_auth_response(
        &mut self,
        challenge: &str,
        out: &mut [u8],
    ) -> Result<usize, ParseError> {
        let md5_hash = compute_md5_response(challenge, self.password);

        let mut ie_buf = [0u8; 96];
        let mut ie_len = 0;

        ie_len += write_ie(
            IeType::Username,
            self.node_number.as_bytes(),
            &mut ie_buf[ie_len..],
        )?;
        let refresh_bytes = self.refresh_secs.to_be_bytes();
        ie_len += write_ie(IeType::Refresh, &refresh_bytes, &mut ie_buf[ie_len..])?;
        ie_len += write_ie(IeType::Md5Result, &md5_hash, &mut ie_buf[ie_len..])?;

        let frame = FullFrame {
            source_call: self.local_call_no,
            dest_call: self.remote_call_no,
            retransmission: false,
            timestamp: self.timestamp_ms,
            o_seqno: self.o_seqno,
            i_seqno: self.i_seqno,
            frame_type: FrameType::Iax,
            subclass: IaxCommand::RegReq.into(),
            c_bit: false,
            payload: &ie_buf[..ie_len],
        };

        self.o_seqno = self.o_seqno.wrapping_add(1);
        self.state = RegState::Authenticating;
        frame.serialize(out)
    }

    /// Process incoming registration responses from ASL directory server
    pub fn handle_response(&mut self, frame: &FullFrame) -> Option<RegAction> {
        self.remote_call_no = frame.source_call;
        self.i_seqno = frame.o_seqno.wrapping_add(1);

        if frame.frame_type != FrameType::Iax {
            return None;
        }

        match IaxCommand::from(frame.subclass) {
            IaxCommand::RegAuth => {
                // Extract challenge string from IEs
                for ie in frame.ies() {
                    if ie.ie_type == IeType::Challenge {
                        if let Some(challenge_str) = ie.as_str() {
                            return Some(RegAction::RespondToChallenge(challenge_str.into()));
                        }
                    }
                }
                None
            }
            IaxCommand::RegAck => {
                self.state = RegState::Registered;
                Some(RegAction::Registered)
            }
            IaxCommand::RegRej => {
                self.state = RegState::Rejected;
                Some(RegAction::Rejected)
            }
            _ => None,
        }
    }
}

pub enum RegAction {
    RespondToChallenge(std::string::String),
    Registered,
    Rejected,
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
