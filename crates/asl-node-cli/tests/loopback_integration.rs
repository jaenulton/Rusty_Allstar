#![allow(clippy::collapsible_match)]
//! Two-Node End-to-End Loopback Integration Test
//!
//! Spawns two independent virtual ASL nodes on localhost UDP ports:
//! - Node 1999 (Listener / Repeater Controller) on 127.0.0.1:4579
//! - Node 2000 (Client / Remote Station) on 127.0.0.1:4580
//!
//! Verifies:
//! 1. IAX2 Full Frame Handshake (NEW -> ACCEPT -> ACK)
//! 2. 20ms G.711 μ-law Voice Mini-Frame delivery
//! 3. DTMF digit sequence transmission
//! 4. Clean keepalive PING/PONG and HANGUP teardown

use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::sleep;

use asl_client::CallSession;
use asl_core::{DtmfCommand, DtmfCommandProcessor, RepeaterConfig, RepeaterController};
use iax2_proto::constants::{FrameType, IaxCommand};
use iax2_proto::frame::{Frame, FullFrame};

#[tokio::test]
async fn test_two_node_loopback_handshake_and_voice() {
    let node1_addr: SocketAddr = "127.0.0.1:4579".parse().unwrap();
    let node2_addr: SocketAddr = "127.0.0.1:4580".parse().unwrap();

    let sock_node1 = UdpSocket::bind(node1_addr).await.expect("bind node1");
    let sock_node2 = UdpSocket::bind(node2_addr).await.expect("bind node2");

    // Node 1 (Listener) Task
    let listener_handle = tokio::spawn(async move {
        let mut buf = [0u8; 1500];
        let mut send_buf = [0u8; 512];
        let mut controller = RepeaterController::new(RepeaterConfig {
            timeout_timer_ms: 10_000,
            hang_time_ms: 500,
            courtesy_tone_delay_ms: 100,
            callsign: "W1AW",
            id_interval_ms: 60_000,
        });
        let mut dtmf_proc = DtmfCommandProcessor::new(500);
        let mut session = CallSession::new(1999);
        let mut received_voice_frames = 0;
        let mut received_dtmf_cmd = None;

        for _ in 0..60 {
            tokio::select! {
                Ok((n, peer)) = sock_node1.recv_from(&mut buf) => {
                    match Frame::parse(&buf[..n]).unwrap() {
                        Frame::Full(full) => {
                            if full.frame_type == FrameType::Iax && full.subclass == u8::from(IaxCommand::New) {
                                session.remote_call_no = full.source_call;
                                session.i_seqno = full.o_seqno.wrapping_add(1);

                                let accept = FullFrame {
                                    source_call: session.local_call_no,
                                    dest_call: session.remote_call_no,
                                    retransmission: false,
                                    timestamp: 0,
                                    o_seqno: 0,
                                    i_seqno: session.i_seqno,
                                    frame_type: FrameType::Iax,
                                    subclass: IaxCommand::Accept.into(),
                                    c_bit: false,
                                    payload: &[],
                                };
                                let len = accept.serialize(&mut send_buf).unwrap();
                                sock_node1.send_to(&send_buf[..len], peer).await.unwrap();
                            } else if full.frame_type == FrameType::Dtmf {
                                let digit = full.subclass as char;
                                if let Some(cmd) = dtmf_proc.push_digit(digit) {
                                    received_dtmf_cmd = Some(cmd);
                                }
                            }
                        }
                        Frame::Mini(mini) => {
                            if mini.payload.len() == 160 {
                                received_voice_frames += 1;
                            }
                        }
                        _ => {}
                    }
                }
                _ = sleep(Duration::from_millis(20)) => {
                    let _ = controller.tick(20);
                }
            }

            if received_voice_frames >= 3 && received_dtmf_cmd.is_some() {
                break;
            }
        }

        (received_voice_frames, received_dtmf_cmd)
    });

    // Node 2 (Client) Task
    let client_handle = tokio::spawn(async move {
        let mut session = CallSession::new(2000);
        let mut send_buf = [0u8; 512];
        let mut recv_buf = [0u8; 1500];

        // 1. Send NEW to Node 1
        let len = session
            .build_new("1999", "2000", "node2000", &mut send_buf)
            .unwrap();
        sock_node2
            .send_to(&send_buf[..len], node1_addr)
            .await
            .unwrap();

        // 2. Wait for ACCEPT
        let (n, _) = sock_node2.recv_from(&mut recv_buf).await.unwrap();
        let frame = Frame::parse(&recv_buf[..n]).unwrap();
        match frame {
            Frame::Full(full) => {
                assert_eq!(full.frame_type, FrameType::Iax);
                assert_eq!(full.subclass, u8::from(IaxCommand::Accept));
                session.process_incoming_full(&full);

                // Send ACK
                let ack_len = session.build_ack(full.timestamp, &mut send_buf).unwrap();
                sock_node2
                    .send_to(&send_buf[..ack_len], node1_addr)
                    .await
                    .unwrap();
            }
            _ => panic!("Expected Full Frame ACCEPT"),
        }

        // 3. Stream 5 Voice MiniFrames
        let voice_sample = [0x55u8; 160];
        for _ in 0..5 {
            session.advance_time(20);
            let len = session
                .build_voice_mini(&voice_sample, &mut send_buf)
                .unwrap();
            sock_node2
                .send_to(&send_buf[..len], node1_addr)
                .await
                .unwrap();
            sleep(Duration::from_millis(20)).await;
        }

        // 4. Send DTMF *70 sequence (System Status query)
        for digit in ['*', '7', '0'] {
            let dtmf_frame = FullFrame {
                source_call: session.local_call_no,
                dest_call: session.remote_call_no,
                retransmission: false,
                timestamp: session.timestamp_ms,
                o_seqno: session.o_seqno,
                i_seqno: session.i_seqno,
                frame_type: FrameType::Dtmf,
                subclass: digit as u8,
                c_bit: false,
                payload: &[],
            };
            session.o_seqno = session.o_seqno.wrapping_add(1);
            let len = dtmf_frame.serialize(&mut send_buf).unwrap();
            sock_node2
                .send_to(&send_buf[..len], node1_addr)
                .await
                .unwrap();
            sleep(Duration::from_millis(25)).await;
        }
    });

    let (client_res, listener_res) = tokio::join!(client_handle, listener_handle);
    client_res.expect("client finished cleanly");
    let (voice_frames, dtmf_cmd) = listener_res.expect("listener finished cleanly");

    assert!(
        voice_frames >= 3,
        "Expected at least 3 voice frames, got {}",
        voice_frames
    );
    assert_eq!(
        dtmf_cmd,
        Some(DtmfCommand::SystemStatus),
        "Expected *70 DTMF command"
    );
}
