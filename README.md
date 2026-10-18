# Rusty_Allstar 📻🦀

**A high-performance, memory-safe, embedded-ready rewrite of the AllStarLink amateur radio repeater and node engine in Rust.**

[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

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
│   ├── iax2-proto/        # Zero-copy IAX2 protocol frame parser/serializer (RFC 5456)
│   ├── asl-dsp/           # G.711 codecs, Goertzel DTMF/CTCSS detectors, tone generators
│   ├── asl-core/          # Repeater controller state machine, CW ID, courtesy tones
│   ├── asl-node-cli/      # Desktop testbed runner (Linux/macOS)
│   └── asl-esp32/         # ESP32 firmware target (esp-idf-svc / esp-hal)
├── docs/                  # Architecture, protocol, and hardware documentation
└── Cargo.toml
```

For full architecture details and design specifications, see [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).
For project milestones and upcoming work, see [docs/ROADMAP.md](docs/ROADMAP.md).

---

## Getting Started

### Prerequisites
* Rust toolchain (1.75+ recommended):
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

### Building the Project
```bash
git clone https://github.com/jaenulton/Rusty_Allstar.git
cd Rusty_Allstar
cargo check --workspace
```

---

## License
Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
