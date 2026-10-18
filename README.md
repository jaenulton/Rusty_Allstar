# Rusty_Allstar 📻🦀

**A high-performance, memory-safe, embedded-ready rewrite of the AllStarLink amateur radio repeater and node engine in Rust.**

[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![GitHub](https://img.shields.io/badge/github-jaenulton%2FRusty__Allstar-blue.svg)](https://github.com/jaenulton/Rusty_Allstar)

---

## What is Rusty_Allstar?

AllStarLink (ASL) is a popular amateur radio VoIP linking network based on Asterisk PBX with the `app_rpt` channel driver. While groundbreaking for its time, Asterisk was designed as a corporate telephony switch, forcing repeaters to act as "telephone calls" wrapped in complex dialplans, DAHDI kernel modules, and decades of monolithic C code with POSIX mutex locks.

**Rusty_Allstar** frees AllStarLink from the legacy Asterisk PBX architecture:
* **Embedded-first**: Designed to run directly on an inexpensive **ESP32 microcontroller** (240 MHz dual-core, ~320 KB heap) as a complete standalone micro-node or repeater controller.
* **Pure Rust**: Memory safe, zero buffer overflows, fearless concurrency without deadlocks, and zero garbage-collection jitter.
* **No DAHDI Required**: Audio timing is handled in user space with high-resolution async timers or hardware I2S DMA.
* **Direct Hardware Audio**: Direct connection to I2S codecs (ES8388, PCM5102, etc.) or radio modules (SA818) with sub-millisecond audio latency.

---

## Workspace Architecture

```
Rusty_Allstar/
├── crates/
│   ├── iax2-proto/        # RFC 5456 IAX2 protocol frame parser/serializer (no_std)
│   ├── asl-dsp/           # G.711 codecs, Goertzel DTMF & CTCSS detectors, CW synthesizer (no_std)
│   ├── asl-core/          # Repeater controller state machine, CW ID, courtesy tones (no_std)
│   ├── asl-client/        # Async IAX2 client, MD5 authentication, call session & registration
│   ├── asl-node-cli/      # Desktop testbed runner (Tokio async UDP, connect/listen commands)
│   └── asl-esp32/         # ESP32 firmware target (dual-core task scheduling, I2S audio, GPIO)
├── docs/
│   ├── ARCHITECTURE.md    # Architectural specification and memory/CPU budgets
│   ├── HARDWARE_ESP32.md  # ESP32 pinout, I2S codec wiring, and radio interfacing schematics
│   └── ROADMAP.md         # Phased project roadmap and milestone tracker
└── Cargo.toml
```

---

## Desktop CLI Runner (`asl-node-cli`)

The workspace includes a desktop testbed CLI for testing DSP algorithms, listening for inbound node links, or connecting directly to an AllStarLink node:

```bash
# 1. Run local DSP diagnostics (DTMF detection, CW generator, G.711 codec)
cargo run --bin asl-node-cli -- dsp-test

# 2. Listen on port 4569 as an inbound AllStarLink node
cargo run --bin asl-node-cli -- listen --bind 0.0.0.0:4569 --callsign W1AW

# 3. Connect to a remote node over IAX2 and stream audio
cargo run --bin asl-node-cli -- connect --host 192.168.1.100:4569 --target-node 50000 --my-node 50001 --tone
```

---

## ESP32 Embedded Architecture

```
                      +---------------------------------------+
                      |               ESP32                   |
                      |                                       |
                      |  [Core 0: Network & IAX2]             |
                      |  - WiFi Station (LwIP UDP)            |
                      |  - IAX2 Client Session                |
                      |                    | (Audio Queues)   |
                      |  [Core 1: DSP & Controller]           |
                      |  - G.711 / Goertzel DTMF / CTCSS      |
                      |  - Repeater Controller State Machine  |
                      +-------+--------------------+----------+
                              | I2S DMA            | GPIO
                              v                    v
                     +-----------------+  +-----------------+
                     |   Audio Codec   |  | Radio Interface |
                     | (ES8388/PCM5102)|  | (Opto / BJT)    |
                     +--------+--------+  +--------+--------+
                              |                    |
                              v                    v
                     [RX / TX Audio]          [COS / PTT]
                              \                    /
                             +----------------------+
                             | Transceiver / SA818  |
                             +----------------------+
```

See [docs/HARDWARE_ESP32.md](docs/HARDWARE_ESP32.md) for complete pinout and circuit schematics.

---

## Testing

Run all unit and integration tests across the workspace:
```bash
cargo test --workspace
```

---

## License
Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
