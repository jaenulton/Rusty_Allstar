# Rusty_Allstar Development Roadmap

## Phase 1: Core Protocol & DSP Foundations (Desktop/Unit-Tested)
- [x] **IAX2 Protocol Parser (`crates/iax2-proto`)**
  - [x] Binary frame parsing: Full Frames, Mini Frames, Meta Frames (RFC 5456).
  - [x] Information Element (IE) encoder and decoder.
  - [x] IAX2 Control Frame & Command definitions (NEW, ACCEPT, ACK, HANGUP, PING, PONG, VOICE).
  - [x] Zero-copy deserializer and unit tests.
- [x] **Audio & DSP Library (`crates/asl-dsp`)**
  - [x] G.711 μ-law / A-law encoder/decoder with fast lookup tables.
  - [x] Goertzel algorithm implementation for real-time DTMF decoding.
  - [x] Goertzel CTCSS sub-audible tone detector (50 standard EIA tones).
  - [x] Sub-audible CTCSS continuous tone encoder.
  - [x] Sine wave & Morse code (CW) tone generator with raised cosine envelope shaping.
- [x] **Repeater State Controller (`crates/asl-core`)**
  - [x] COS / PTT state machine (Hang timer, Time-out timer, Kerchunk filter).
  - [x] Courtesy tone player & Morse code ID sequencer.

## Phase 2: Desktop Prototype & Network Interop
- [x] **IAX2 Client Session Engine (`crates/asl-client`)**
  - [x] RFC 5456 MD5 challenge-response authentication.
  - [x] Outbound call setup (`NEW` -> `AUTHREQ` -> `AUTHREP` -> `ACCEPT` -> `ACK`).
  - [x] 20ms G.711 audio packet streamer (`MiniFrame`).
  - [x] Keepalive heartbeat (`PING` / `PONG`).
  - [x] AllStarLink registration client (`REGREQ`, `REGAUTH`, `REGACK`).
- [x] **Desktop CLI Node (`crates/asl-node-cli`)**
  - [x] Async UDP network transport using Tokio.
  - [x] CLI interface with `connect`, `listen`, and `dsp-test` commands.
  - [x] Embedded HTTP Web Management Dashboard (`axum`, `asl-web`).
  - [x] Live desktop sound card microphone/speaker I/O (`cpal`).
  - [x] Synthetic 1000 Hz tone generator for live loopback testing.

## Phase 3: ESP32 Port & Embedded Integration
- [x] **ESP32 Firmware Target (`crates/asl-esp32`)**
  - [x] ESP-IDF FreeRTOS dual-core task assignment:
    - Core 0: WiFi / LwIP UDP stack & IAX2 call session.
    - Core 1: I2S DMA audio pipeline, DSP tone filters, and GPIO repeater controller.
  - [x] Hardware I2S codec driver abstraction (`I2sHardwareDriver`).
  - [x] Hardware GPIO interrupts & safety watchdog for COS & PTT (`GpioInterruptController`).
  - [x] Non-volatile configuration storage (`NodeNvsConfig` for Wi-Fi, node ID, and radio frequencies).
  - [x] Direct UART AT driver for NiceRF SA818 / SA818S VHF/UHF transceiver modules.

## Phase 4: Hardware & Web Configuration
- [x] **Embedded Web UI (`crates/asl-web`)**
  - [x] High-tech dark-mode dashboard with live COS/PTT glowing LEDs, DTMF keypad, and CW ID button.
  - [x] JSON REST API endpoints (`/api/status`, `/api/connect`, `/api/disconnect`, `/api/dtmf`, `/api/cwid`).
- [x] **Open Hardware Reference Design (`hardware/kicad/`)**
  - [x] Engineering specification & Bill of Materials (`HARDWARE_DESIGN_SPEC.md`).
  - [x] KiCad project & netlist schema (`rusty_allstar_micronode.kicad_sch`).
  - [x] 5-pole Chebyshev low-pass harmonic suppression filter & 50Ω coplanar waveguide layout guidelines.
