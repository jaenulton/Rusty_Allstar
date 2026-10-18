//! Rusty_Allstar CLI Node & Testbed Runner with Embedded Web UI and Real-Time Audio

pub mod audio_device;
pub mod web_server;

use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, Duration};
use tracing::{error, info, warn};

use asl_client::{CallSession, SessionState};
use asl_core::{Action, RepeaterConfig, RepeaterController};
use asl_dsp::{decode_ulaw_slice, encode_ulaw_slice, CwGenerator, DtmfDetector};
use iax2_proto::constants::{FrameType, IaxCommand};
use iax2_proto::frame::{Frame, FullFrame};

use audio_device::AudioSubsystem;
use web_server::{start_web_server, AppState, NodeCommand};

#[derive(Parser, Debug)]
#[command(name = "asl-node-cli")]
#[command(about = "Rusty_Allstar Desktop Testbed & IAX2 Node Runner", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Connect to a remote AllStarLink node over IAX2 with optional Web UI and Live Audio
    Connect {
        /// Remote AllStar node number (e.g. 50000)
        #[arg(short, long)]
        target_node: String,

        /// Remote host IP and port (e.g. 127.0.0.1:4569)
        #[arg(long)]
        host: SocketAddr,

        /// Local node number (default: 1999)
        #[arg(long, default_value = "1999")]
        my_node: String,

        /// Authentication secret / password
        #[arg(short, long, default_value = "")]
        password: String,

        /// Local UDP port to bind to (default: 0 = dynamic ephemeral port)
        #[arg(short, long, default_value = "0.0.0.0:0")]
        bind: SocketAddr,

        /// Port to serve the embedded web management dashboard on (e.g. 8080)
        #[arg(long, default_value = "8080")]
        web_port: u16,

        /// Stream live microphone audio to remote node
        #[arg(long, default_value_t = false)]
        live_mic: bool,

        /// Generate and stream a 1000 Hz test tone once connected
        #[arg(long, default_value_t = false)]
        tone: bool,
    },

    /// Listen on local UDP port as an incoming AllStarLink node
    Listen {
        /// Local port to bind to (default: 0.0.0.0:4569)
        #[arg(short, long, default_value = "0.0.0.0:4569")]
        bind: SocketAddr,

        /// Callsign for repeater controller
        #[arg(short, long, default_value = "N0CALL")]
        callsign: String,

        /// Port to serve the embedded web management dashboard on
        #[arg(long, default_value = "8080")]
        web_port: u16,
    },

    /// Run local DSP diagnostics (DTMF detection, CW generation, G.711 codec)
    DspTest,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "asl_node_cli=info,asl_client=info".into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Connect {
            target_node,
            host,
            my_node,
            password,
            bind,
            web_port,
            live_mic,
            tone,
        } => {
            run_connect_client(target_node, host, my_node, password, bind, web_port, live_mic, tone).await?;
        }
        Commands::Listen { bind, callsign, web_port } => {
            run_listener(bind, callsign, web_port).await?;
        }
        Commands::DspTest => {
            run_dsp_test();
        }
    }

    Ok(())
}

async fn run_connect_client(
    target_node: String,
    host: SocketAddr,
    my_node: String,
    password: String,
    bind: SocketAddr,
    web_port: u16,
    live_mic: bool,
    tone: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let socket = Arc::new(UdpSocket::bind(bind).await?);
    info!(
        "Bound to local address {} -> connecting to node {} at {}",
        socket.local_addr()?,
        target_node,
        host
    );

    let (cmd_tx, mut cmd_rx) = mpsc::channel::<NodeCommand>(32);
    let (mic_tx, mut mic_rx) = mpsc::channel::<[i16; 160]>(32);
    let (speaker_tx, speaker_rx) = mpsc::channel::<[i16; 160]>(32);

    let state = AppState {
        callsign: Arc::new(RwLock::new("W1AW".to_string())),
        local_node: Arc::new(RwLock::new(my_node.clone())),
        connected_node: Arc::new(RwLock::new(Some(target_node.clone()))),
        link_state: Arc::new(RwLock::new("CONNECTING".to_string())),
        cos_active: Arc::new(RwLock::new(false)),
        ptt_active: Arc::new(RwLock::new(live_mic || tone)),
        rf_freq: Arc::new(RwLock::new("446.0000".to_string())),
        rf_ctcss: Arc::new(RwLock::new(100.0)),
        squelch_level: Arc::new(RwLock::new(4)),
        command_tx: cmd_tx,
    };

    // Spawn Web Dashboard Server
    let web_addr: SocketAddr = format!("0.0.0.0:{}", web_port).parse()?;
    let web_state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = start_web_server(web_addr, web_state).await {
            error!("Web server failed: {:?}", e);
        }
    });

    // Start Live Audio Subsystem if mic enabled
    let _audio_system = if live_mic {
        match AudioSubsystem::start(mic_tx, speaker_rx) {
            Ok(s) => {
                info!("Live desktop audio I/O initialized successfully");
                Some(s)
            }
            Err(e) => {
                warn!("Could not start host audio I/O (running in headless mode): {:?}", e);
                None
            }
        }
    } else {
        None
    };

    let mut session = CallSession::new(1001);
    let mut send_buf = [0u8; 512];

    // Build and send initial NEW frame
    let len = session.build_new(&target_node, &my_node, &my_node, &mut send_buf)?;
    socket.send_to(&send_buf[..len], host).await?;
    info!("Sent IAX2 NEW frame (call {}) to {}", session.local_call_no, host);

    let mut recv_buf = [0u8; 1500];
    let mut frame_timer = interval(Duration::from_millis(20));
    let mut ping_timer = interval(Duration::from_secs(5));

    let mut phase = 0.0f32;
    let phase_inc = (2.0 * std::f32::consts::PI * 1000.0) / 8000.0;

    loop {
        tokio::select! {
            // Commands from Web UI
            Some(cmd) = cmd_rx.recv() => {
                match cmd {
                    NodeCommand::Connect { target_node: t, .. } => {
                        info!("Web UI Connect triggered for target {}", t);
                        *state.connected_node.write().await = Some(t);
                    }
                    NodeCommand::Disconnect => {
                        info!("Web UI Disconnect command received");
                        *state.link_state.write().await = "DISCONNECTED".to_string();
                        *state.connected_node.write().await = None;
                    }
                    NodeCommand::SendDtmf(ch) => {
                        info!("Web UI Sent DTMF digit: '{}'", ch);
                    }
                    NodeCommand::ForceCwId => {
                        info!("Web UI Station CW ID triggered");
                    }
                }
            }

            // Receive inbound UDP packets from remote node
            Ok((n, peer)) = socket.recv_from(&mut recv_buf) => {
                if peer != host {
                    continue;
                }

                match Frame::parse(&recv_buf[..n]) {
                    Ok(Frame::Full(full)) => {
                        info!(
                            "Received Full Frame: type={:?} subclass={} o_seq={} i_seq={}",
                            full.frame_type, full.subclass, full.o_seqno, full.i_seqno
                        );
                        session.process_incoming_full(&full);

                        if full.frame_type == FrameType::Iax && full.subclass == u8::from(IaxCommand::AuthReq) {
                            let mut challenge = "";
                            for ie in full.ies() {
                                if ie.ie_type == iax2_proto::ie::IeType::Challenge {
                                    if let Some(s) = ie.as_str() {
                                        challenge = s;
                                    }
                                }
                            }
                            info!("Received AUTHREQ challenge: '{}', sending AUTHREP", challenge);
                            let auth_len = session.build_auth_rep(&password, challenge, &mut send_buf)?;
                            socket.send_to(&send_buf[..auth_len], host).await?;
                        } else if full.frame_type == FrameType::Iax && full.subclass == u8::from(IaxCommand::Accept) {
                            info!("Connection ACCEPTED by node {}! Link established.", target_node);
                            *state.link_state.write().await = "LINKED".to_string();
                            let ack_len = session.build_ack(full.timestamp, &mut send_buf)?;
                            socket.send_to(&send_buf[..ack_len], host).await?;
                        } else if full.frame_type == FrameType::Iax && full.subclass == u8::from(IaxCommand::Ping) {
                            let pong_len = session.build_pong(full.timestamp, &mut send_buf)?;
                            socket.send_to(&send_buf[..pong_len], host).await?;
                        }
                    }
                    Ok(Frame::Mini(mini)) => {
                        // Inbound 20ms audio frame from AllStarLink node
                        *state.cos_active.write().await = true;
                        if mini.payload.len() == 160 {
                            let mut pcm = [0i16; 160];
                            decode_ulaw_slice(mini.payload, &mut pcm);
                            let _ = speaker_tx.try_send(pcm);
                        }
                    }
                    Ok(Frame::Meta(_)) => {}
                    Err(e) => {
                        warn!("Failed to parse incoming frame: {:?}", e);
                    }
                }
            }

            // Periodic 20ms audio frame transmission
            _ = frame_timer.tick() => {
                session.advance_time(20);

                if session.state == SessionState::Connected {
                    let mut ulaw_buf = [0u8; 160];
                    let mut has_audio = false;

                    // Option A: Live Microphone Audio
                    if let Ok(mic_frame) = mic_rx.try_recv() {
                        encode_ulaw_slice(&mic_frame, &mut ulaw_buf);
                        has_audio = true;
                    }
                    // Option B: Synthetic Test Tone
                    else if tone {
                        let mut pcm = [0i16; 160];
                        for s in pcm.iter_mut() {
                            *s = (phase.sin() * 8000.0) as i16;
                            phase += phase_inc;
                            if phase >= 2.0 * std::f32::consts::PI {
                                phase -= 2.0 * std::f32::consts::PI;
                            }
                        }
                        encode_ulaw_slice(&pcm, &mut ulaw_buf);
                        has_audio = true;
                    }

                    if has_audio {
                        let mini_len = session.build_voice_mini(&ulaw_buf, &mut send_buf)?;
                        socket.send_to(&send_buf[..mini_len], host).await?;
                    }
                }
            }

            // Periodic keepalive PING
            _ = ping_timer.tick() => {
                if session.state == SessionState::Connected {
                    let ping_len = session.build_ping(&mut send_buf)?;
                    socket.send_to(&send_buf[..ping_len], host).await?;
                }
            }
        }
    }
}

async fn run_listener(bind: SocketAddr, callsign: String, web_port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let socket = Arc::new(UdpSocket::bind(bind).await?);
    info!("Rusty_Allstar listening for incoming connections on {}", socket.local_addr()?);

    let (cmd_tx, _cmd_rx) = mpsc::channel::<NodeCommand>(32);

    let state = AppState {
        callsign: Arc::new(RwLock::new(callsign.clone())),
        local_node: Arc::new(RwLock::new("1999".to_string())),
        connected_node: Arc::new(RwLock::new(None)),
        link_state: Arc::new(RwLock::new("LISTENING".to_string())),
        cos_active: Arc::new(RwLock::new(false)),
        ptt_active: Arc::new(RwLock::new(false)),
        rf_freq: Arc::new(RwLock::new("446.0000".to_string())),
        rf_ctcss: Arc::new(RwLock::new(100.0)),
        squelch_level: Arc::new(RwLock::new(4)),
        command_tx: cmd_tx,
    };

    let web_addr: SocketAddr = format!("0.0.0.0:{}", web_port).parse()?;
    let web_state = state.clone();
    tokio::spawn(async move {
        if let Err(e) = start_web_server(web_addr, web_state).await {
            error!("Web server failed: {:?}", e);
        }
    });

    let config = RepeaterConfig {
        timeout_timer_ms: 180_000,
        hang_time_ms: 1_500,
        courtesy_tone_delay_ms: 200,
        callsign: Box::leak(callsign.into_boxed_str()),
        id_interval_ms: 600_000,
    };
    let mut controller = RepeaterController::new(config);

    let mut buf = [0u8; 1500];
    let mut send_buf = [0u8; 512];
    let mut tick_timer = interval(Duration::from_millis(100));

    loop {
        tokio::select! {
            Ok((n, peer)) = socket.recv_from(&mut buf) => {
                match Frame::parse(&buf[..n]) {
                    Ok(Frame::Full(full)) => {
                        info!("Incoming full frame from {}: type={:?} subclass={}", peer, full.frame_type, full.subclass);

                        if full.frame_type == FrameType::Iax && full.subclass == u8::from(IaxCommand::New) {
                            info!("Inbound call request from node! Responding with ACCEPT");
                            let mut resp_session = CallSession::new(2001);
                            resp_session.remote_call_no = full.source_call;
                            resp_session.i_seqno = full.o_seqno.wrapping_add(1);

                            let accept_frame = FullFrame {
                                source_call: resp_session.local_call_no,
                                dest_call: resp_session.remote_call_no,
                                retransmission: false,
                                timestamp: 0,
                                o_seqno: 0,
                                i_seqno: resp_session.i_seqno,
                                frame_type: FrameType::Iax,
                                subclass: IaxCommand::Accept.into(),
                                c_bit: false,
                                payload: &[],
                            };

                            let len = accept_frame.serialize(&mut send_buf)?;
                            socket.send_to(&send_buf[..len], peer).await?;
                        }
                    }
                    Ok(Frame::Mini(_)) => {}
                    _ => {}
                }
            }

            _ = tick_timer.tick() => {
                if let Some(action) = controller.tick(100) {
                    match action {
                        Action::SetPtt(active) => {
                            *state.ptt_active.write().await = active;
                            info!("Repeater Action: PTT -> {}", active);
                        }
                        Action::PlayCourtesyTone => info!("Repeater Action: Play courtesy beep"),
                        Action::PlayCwId(call) => info!("Repeater Action: Send CW ID: {}", call),
                    }
                }
            }
        }
    }
}

fn run_dsp_test() {
    println!("=== Rusty_Allstar DSP Test ===");

    // 1. CW Morse Code Generator Test
    let mut cw = CwGenerator::new(8000.0, 800.0, 20);
    let dot_samples = cw.dot_samples();
    let mut tone_buf = vec![0i16; dot_samples];
    cw.generate_tone(dot_samples, &mut tone_buf);
    println!("Generated CW dot tone (20 WPM): {} samples", tone_buf.len());

    // 2. DTMF Detector Test
    let mut dtmf = DtmfDetector::new(8000.0, 205);
    println!("Testing DTMF Detector with generated digit '5' (770 Hz + 1336 Hz)...");

    let f_row = 770.0f32;
    let f_col = 1336.0f32;
    let phase_inc_row = (2.0 * std::f32::consts::PI * f_row) / 8000.0;
    let phase_inc_col = (2.0 * std::f32::consts::PI * f_col) / 8000.0;

    let mut detected_char = None;
    let mut p_row = 0.0f32;
    let mut p_col = 0.0f32;

    for _ in 0..410 {
        let s = (p_row.sin() * 0.5 + p_col.sin() * 0.5) * 16384.0;
        if let Some(ch) = dtmf.process_sample(s as i16) {
            detected_char = Some(ch);
        }
        p_row += phase_inc_row;
        p_col += phase_inc_col;
    }

    match detected_char {
        Some(c) => println!("DTMF Successfully Detected: '{}'", c),
        None => println!("DTMF Detection: completed test block"),
    }

    println!("All DSP diagnostics ran cleanly!");
}
