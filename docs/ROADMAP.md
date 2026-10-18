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
  - [x] Synthetic 1000 Hz tone generator for live loopback testing.

## Phase 3: ESP32 Port & Embedded Integration
- [ ] **ESP32 Firmware Target (`crates/asl-esp32`)**
  - [ ] ESP-IDF FreeRTOS dual-core task assignment:
    - Core 0: WiFi / LwIP UDP stack & IAX2 call session.
    - Core 1: I2S DMA audio pipeline, DSP tone filters, and GPIO repeater controller.
  - [ ] Hardware I2S codec driver (ES8388, PCM5102, or WM8960).
  - [ ] Hardware GPIO interrupts for Carrier Operated Squelch (COS) and Push-To-Talk (PTT).
  - [ ] Non-volatile configuration storage (NVS) for WiFi SSID, node credentials, and audio levels.

## Phase 4: Hardware & Web Configuration
- [ ] Embedded web configuration UI served directly from ESP32.
- [ ] Direct UART control for embedded NiceRF SA818 / SR-FRS transceiver modules.
- [ ] Open hardware reference schematic (KiCad) for an all-in-one ESP32 micro-node.
