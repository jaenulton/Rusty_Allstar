# Rusty_Allstar 📻🦀

**A high-performance, memory-safe, embedded-ready rewrite of the AllStarLink amateur radio repeater and node engine in Rust.**

[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)
[![CI](https://github.com/jaenulton/Rusty_Allstar/actions/workflows/ci.yml/badge.svg)](https://github.com/jaenulton/Rusty_Allstar/actions)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![GitHub](https://img.shields.io/badge/github-jaenulton%2FRusty__Allstar-blue.svg)](https://github.com/jaenulton/Rusty_Allstar)

---

## What is Rusty_Allstar?

AllStarLink (ASL) is a popular amateur radio VoIP linking network based on Asterisk PBX with the `app_rpt` channel driver. While groundbreaking for its time, Asterisk was designed as a corporate telephony switch, forcing repeaters to act as "telephone calls" wrapped in complex dialplans, DAHDI kernel modules, and decades of monolithic C code with POSIX mutex locks.

**Rusty_Allstar** frees AllStarLink from the legacy Asterisk PBX architecture:
* **Embedded-first**: Designed to run directly on an inexpensive **ESP32 microcontroller** (240 MHz dual-core, ~320 KB heap) as a complete standalone micro-node or repeater controller.
* **Pure Rust**: Memory safe, zero buffer overflows, fearless concurrency without deadlocks, and zero garbage-collection jitter.
* **No DAHDI Required**: Audio timing is handled in user space with high-resolution async timers or hardware I2S DMA.
* **Direct Hardware Audio**: Direct connection to I2S codecs (ES8388, PCM5102, etc.) or radio modules (NiceRF SA818) with sub-millisecond audio latency.
* **Modern Web Dashboard**: Real-time browser control panel with WebSocket telemetry, Web Audio monitor, and live DTMF keypad.

---

## Workspace Architecture

```
Rusty_Allstar/
├── crates/
│   ├── iax2-proto/        # RFC 5456 IAX2 protocol frame parser/serializer (no_std)
│   ├── asl-dsp/           # G.711 codecs, Goertzel DTMF & CTCSS, CW synthesizer, Jitter Buffer, Audio Mixer (no_std)
│   ├── asl-core/          # Repeater controller state machine, DTMF command parser, Conference Bridge (no_std)
│   ├── asl-client/        # Async IAX2 client, MD5 auth, call session, ASL DNS resolver & ASTDB directory
│   ├── asl-web/           # Embedded Web Dashboard, WebSocket telemetry, and REST API definitions
│   ├── asl-node-cli/      # Desktop testbed runner (Tokio async UDP, CPAL live audio, Axum web server, TUI)
│   └── asl-esp32/         # ESP32 firmware target (Dual-core FreeRTOS, I2S audio DMA, GPIO ISRs, SA818 UART)
├── docs/
│   ├── ARCHITECTURE.md    # Architectural specification and memory/CPU budgets
│   ├── ESP_BUILD_GUIDE.md # ESP-IDF toolchain setup, hardware wiring, and flashing guide
│   ├── HARDWARE_ESP32.md  # ESP32 pinout, I2S codec wiring, and radio interfacing schematics
│   └── ROADMAP.md         # Phased project roadmap and milestone tracker
├── hardware/
│   └── kicad/             # Hardware reference design, BOM, schematics, and KiCad netlist
└── scripts/
    └── install_esp_toolchain.sh # Automated Espressif Xtensa toolchain installer
```

---

## Quick Start (Desktop Testbed)

### 1. Run Tests & Diagnostics
```bash
# Run unit & integration test suite (39 tests)
cargo test --workspace

# Run local DSP diagnostics (DTMF detection, CW generator, G.711 codec)
cargo run --bin asl-node-cli -- dsp-test
```

### 2. Launch Local Listening Node & Web Dashboard
```bash
cargo run --bin asl-node-cli -- listen --bind 0.0.0.0:4569 --callsign W1AW --web-port 8080
```
Open **`http://localhost:8080`** in any browser to access the control panel:
* **Live Glowing LEDs**: Real-time COS (Carrier Detect) and PTT (Push-To-Talk) indicators powered by WebSockets.
* **Browser Web Audio**: Listen directly to repeater traffic in your browser or key the virtual PTT microphone.
* **DTMF Touch Keypad**: Send commands (`*1`, `*2`, `*3`, `*70`) directly from the browser.

### 3. Connect to a Remote AllStarLink Node
```bash
# Connect with live microphone & speaker audio:
cargo run --bin asl-node-cli -- connect --target-node 50000 --live-mic

# Or connect with synthetic 1000 Hz test tone:
cargo run --bin asl-node-cli -- connect --target-node 50000 --tone
```

---

## ESP32 Embedded Micro-Node

`asl-esp32` coordinates dual-core FreeRTOS execution:
* **Core 0 (Network & Web)**: WiFi association, IAX2 UDP audio streaming, and embedded Web server.
* **Core 1 (Real-Time Audio DSP)**: I2S DMA transfers (8 kHz 16-bit mono), Goertzel CTCSS tone decode/encode, and GPIO repeater state machine.

### Flashing Physical Hardware
```bash
# 1. Install toolchain:
./scripts/install_esp_toolchain.sh
source $HOME/export-esp.sh

# 2. Build and Flash:
cargo build --target xtensa-esp32-espidf --package asl-esp32 --release
espflash flash target/xtensa-esp32-espidf/release/asl-esp32 --monitor
```

See [`docs/ESP_BUILD_GUIDE.md`](docs/ESP_BUILD_GUIDE.md) and [`hardware/kicad/HARDWARE_DESIGN_SPEC.md`](hardware/kicad/HARDWARE_DESIGN_SPEC.md) for complete hardware wiring and PCB schematics.

---

## License

Dual-licensed under either:
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
