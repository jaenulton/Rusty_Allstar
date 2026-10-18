//! # asl-client
//!
//! Async IAX2 client and session controller for AllStarLink linking and registration.

pub mod auth;
pub mod registration;
pub mod session;

pub use auth::compute_md5_response;
pub use registration::{RegAction, RegState, RegistrationClient};
pub use session::{CallSession, SessionState};

#[cfg(test)]
mod tests {
    use super::*;
    use iax2_proto::constants::{FrameType, IaxCommand};
    use iax2_proto::frame::FullFrame;
    use iax2_proto::ie::IeType;

    #[test]
    fn test_call_session_setup_and_auth() {
        let mut session = CallSession::new(1001);
        let mut buf = [0u8; 256];

        // 1. Build NEW
        let len = session.build_new("50000", "50001", "node50001", &mut buf).unwrap();
        assert_eq!(session.state, SessionState::Connecting);
        assert_eq!(session.o_seqno, 1);

        let parsed_new = FullFrame::parse(&buf[..len]).unwrap();
        assert_eq!(parsed_new.source_call, 1001);
        assert_eq!(parsed_new.dest_call, 0);
        assert_eq!(parsed_new.frame_type, FrameType::Iax);
        assert_eq!(parsed_new.subclass, IaxCommand::New.into());

        // 2. Build AUTHREP
        let challenge = "abcdef123456";
        let secret = "supersecret";
        let len = session.build_auth_rep(secret, challenge, &mut buf).unwrap();
        assert_eq!(session.state, SessionState::Authenticating);

        let parsed_auth = FullFrame::parse(&buf[..len]).unwrap();
        assert_eq!(parsed_auth.subclass, IaxCommand::AuthRep.into());

        // Verify MD5 IE in payload
        let mut found_md5 = false;
        for ie in parsed_auth.ies() {
            if ie.ie_type == IeType::Md5Result {
                found_md5 = true;
                assert_eq!(ie.data.len(), 32);
            }
        }
        assert!(found_md5);
    }

    #[test]
    fn test_voice_mini_frame_generation() {
        let session = CallSession::new(1001);
        let audio_ulaw = [0xFFu8; 160]; // 20ms of silence/max in ulaw
        let mut buf = [0u8; 200];

        let len = session.build_voice_mini(&audio_ulaw, &mut buf).unwrap();
        assert_eq!(len, 4 + 160);

        let mini = iax2_proto::frame::MiniFrame::parse(&buf[..len]).unwrap();
        assert_eq!(mini.source_call, 1001);
        assert_eq!(mini.payload.len(), 160);
    }
}
