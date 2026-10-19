# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-10-19

### Added
- **Pure-Rust IAX2 Protocol Engine (`crates/iax2-proto`)**:
  - Full Frame, Mini Frame, and Meta Frame binary parser & serializer compliant with RFC 5456.
  - Information Element (IE) encoder/decoder (MD5 challenge/response, call numbers, codec formats).
  - Outbound call setup (`NEW` -> `AUTHREQ` -> `AUTHREP` -> `ACCEPT` -> `ACK`), audio streaming, and keepalive heartbeat (`PING` / `PONG`).
- **AllStarLink Client & Directory Resolver (`crates/asl-client`)**:
  - ASTDB (`astdb.txt`) parser and fast in-memory lookup cache for node numbers, locations, and frequencies.
  - Automatic node IP/port resolution via AllStarLink DNS convention (`<node>.nodes.allstarlink.org`).
- **Audio DSP & Signal Processing (`crates/asl-dsp`)**:
  - High-performance G.711 μ-law / A-law encoder & decoder with lookup tables.
  - Goertzel algorithm tone detector for DTMF digits (0-9, *, #, A-D) and all 50 EIA standard CTCSS sub-audible tones.
  - Sub-audible CTCSS continuous tone synthesizer and Morse code (CW) ID tone generator with raised-cosine anti-click envelope shaping.
  - Adaptive VoIP jitter buffer with out-of-order packet resequencing.
  - Multi-stream audio conference mixer with $N-1$ echo suppression and soft-knee limiting.
- **Repeater Controller & Conference Bridge (`crates/asl-core`)**:
  - Core repeater state machine with configurable hang timers, transmitter time-out protection, courtesy tones, and CW ID scheduling.
  - AllStarLink DTMF command parser (`*1<node>` disconnect, `*2<node>` monitor, `*3<node>` link, `*70` status, `*80` force CW ID).
  - Multi-node conference bridge for multi-link routing.
- **ESP32 Firmware Target (`crates/asl-esp32`)**:
  - Dual-core FreeRTOS firmware architecture (Core 0: network/web, Core 1: I2S DMA audio & DSP).
  - Hardware I2S audio driver abstraction for PCM5102 / ES8388 codecs.
  - NiceRF SA818 / SA818S serial UART AT driver for programming VHF/UHF frequencies, squelch, and power.
  - Hardware GPIO interrupt controller with debounce and transmitter safety watchdog.
  - Binary NVS storage serialization for Wi-Fi credentials and radio parameters.
  - Mock radio & SA818 emulator for headless desktop automated simulation.
- **Embedded Web Dashboard & REST API (`crates/asl-web`)**:
  - Modern dark-mode responsive HTML5/CSS dashboard with live glowing COS and PTT LED indicators.
  - Interactive DTMF touch keypad, manual CW ID trigger, and node linking controls.
  - Browser Web Audio API monitor and virtual Push-To-Talk microphone.
  - Low-latency WebSocket telemetry stream (`/ws`) providing 50ms status broadcasts.
- **Desktop CLI Node (`crates/asl-node-cli`)**:
  - Tokio async UDP runner supporting `connect`, `listen`, and `dsp-test` modes.
  - Live soundcard microphone and speaker I/O via CPAL.
  - Console Terminal User Interface (TUI) with real-time colorized VU meters.
  - Automated two-node loopback integration test suite.
- **Hardware Reference Design (`hardware/kicad/`)**:
  - Complete engineering specification, BOM, and KiCad 8 netlist for a standalone ESP32 + NiceRF SA818 pocket micro-node.
  - 5-pole Chebyshev low-pass harmonic suppression filter layout guidelines.
