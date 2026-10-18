# Rusty_Allstar Development Roadmap

## Phase 1: Core Protocol & DSP Foundations (Desktop/Unit-Tested)
- [ ] **IAX2 Protocol Parser (`crates/iax2-proto`)**
  - [ ] Binary frame parsing: Full Frames, Mini Frames, Meta Frames (RFC 5456).
  - [ ] Information Element (IE) encoder and decoder.
  - [ ] IAX2 Control Frame & Command definitions (NEW, ACCEPT, ACK, HANGUP, PING, PONG, VOICE).
  - [ ] Unit tests matching standard Asterisk IAX2 packet captures.
- [ ] **Audio & DSP Library (`crates/asl-dsp`)**
  - [ ] G.711 μ-law / A-law encoder/decoder with fast lookup tables.
  - [ ] Goertzel algorithm implementation for real-time DTMF decoding.
  - [ ] Goertzel CTCSS sub-audible tone detector (38 standard EIA tones).
  - [ ] Sine wave & Morse code (CW) tone generator with audio envelope shaping.
- [ ] **Repeater State Controller (`crates/asl-core`)**
  - [ ] COS / PTT state machine (Hang timer, Time-out timer, Kerchunk filter).
  - [ ] Audio mixer / matrix (Local RF audio + Network IAX2 audio).
  - [ ] Courtesy tone player & Morse code ID sequencer.

## Phase 2: Desktop Prototype & Network Interop
- [ ] **Desktop CLI Node (`crates/asl-node-cli`)**
  - [ ] Async UDP network transport using Tokio.
  - [ ] Audio I/O using CPAL / ALSA.
  - [ ] AllStarLink directory registration client (`register.allstarlink.org`).
  - [ ] Live audio loopback and connecting to an active AllStar node or echo reflector.

## Phase 3: ESP32 Port & Embedded Integration
- [ ] **ESP32 Firmware Base (`crates/asl-esp32`)**
  - [ ] ESP-IDF FreeRTOS dual-core task assignment (Core 0: WiFi/IAX2, Core 1: DSP/Audio/State).
  - [ ] Hardware I2S audio driver (ES8388 or PCM5102/INMP441 codec).
  - [ ] Hardware GPIO interrupts for COS and outputs for PTT.
  - [ ] WiFi station configuration (Captive portal or NVRAM/NVS settings).

## Phase 4: Polish, Web Management & Field Deployment
- [ ] Embedded web configuration UI (served directly from ESP32).
- [ ] Support for direct SA818 RF module tuning over UART.
- [ ] Open hardware reference PCB design.
