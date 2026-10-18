# ESP32 Build, Flashing & Hardware Guide

This document details how to set up the Rust ESP-IDF toolchain, compile `asl-esp32`, wire the peripheral radio hardware (NiceRF SA818 & I2S Codec), and flash physical boards.

---

## 1. Prerequisites & Toolchain Setup

ESP32 microcontrollers use either the Xtensa architecture (ESP32, ESP32-S2, ESP32-S3) or RISC-V (ESP32-C3, ESP32-C6). Rust provides first-class support via [`espup`](https://github.com/esp-rs/espup) and `espflash`.

### Install ESP-RS Toolchain Installer & Flasher:
```bash
cargo install espup
cargo install espflash
cargo install ldproxy
```

### Install Xtensa Rust Toolchain:
```bash
espup install
# Source the environment variables configured by espup (or add to your ~/.bashrc or ~/.zshrc)
source $HOME/export-esp.sh
```

---

## 2. Hardware Architecture & Pinout Reference

`asl-esp32` is configured to interface with:
1. **NiceRF SA818 / SA818S Transceiver Module** (VHF 134–174 MHz or UHF 400–480 MHz).
2. **I2S Audio ADC/DAC Codec** (e.g. PCM5102A DAC + MAX9814/INMP441 ADC, or ES8388 Audio Codec).
3. **Discrete Radio GPIO Lines** (COS, PTT, Audio Power Enable).

### Default Pin Assignments

| Signal Name | ESP32 GPIO | SA818 / Codec Pin | Description |
|---|---|---|---|
| **UART TX** | `GPIO 17` | SA818 `RXD` | Sends serial AT commands (9600 baud) |
| **UART RX** | `GPIO 16` | SA818 `TXD` | Receives command responses from SA818 |
| **COS / SQL** | `GPIO 34` | SA818 `SQ` / COS | Active-low squelch / carrier detect input |
| **PTT OUT** | `GPIO 25` | SA818 `PTT` | Active-low transmitter key output |
| **PDN / Sleep** | `GPIO 26` | SA818 `PDN` | Power-down line (HIGH = active, LOW = sleep) |
| **I2S BCLK** | `GPIO 27` | Codec `BCK` | Bit clock for 8 kHz / 16-bit audio |
| **I2S WS / LRC** | `GPIO 22` | Codec `LRC` | Left/Right word select clock (8 kHz) |
| **I2S DOUT** | `GPIO 19` | Codec `DIN` | Transmit audio out to SA818 MIC IN |
| **I2S DIN** | `GPIO 35` | Codec `DOUT` | Receive audio in from SA818 AUDIO OUT |

---

## 3. Dual-Core Task Allocation

`asl-esp32` uses FreeRTOS task pinning to guarantee zero jitter for real-time DSP and IAX2 streaming:

- **Core 0 (Network & Control)**:
  - FreeRTOS priority 5: WiFi association & LwIP UDP socket management.
  - IAX2 protocol session handling (Full Frame parsing, Mini Frame sequencing, Auth MD5, Ping/Pong).
  - Embedded HTTP REST API & Web Dashboard (`asl-web`).
- **Core 1 (Real-Time Audio DSP)**:
  - FreeRTOS priority 10: I2S DMA ISR buffer transfers.
  - DSP Pipeline: Goertzel CTCSS tone decode / encode, μ-law G.711 compression, DTMF generation.
  - GPIO State Machine: COS debounce and PTT watchdog.

---

## 4. Building & Flashing

### Check Target Configuration
Ensure target settings in `.cargo/config.toml` match your MCU (default is Xtensa `xtensa-esp32-espidf`).

### Build Firmware:
```bash
# Set target for Xtensa ESP32
cargo build --target xtensa-esp32-espidf --package asl-esp32 --release
```

### Flash to Physical Board:
Connect your ESP32 module via USB (e.g., `/dev/ttyUSB0` or `/dev/ttyACM0`):
```bash
espflash flash target/xtensa-esp32-espidf/release/asl-esp32 --monitor
```

---

## 5. First Boot & Verification

1. Once booted, the ESP32 logs boot messages and initial handshake with the NiceRF SA818:
   ```text
   [INFO] Radio driver initialized. SA818 handshake...
   [INFO] SA818 response: +DMOCONNECT:0
   [INFO] SA818 frequency programmed: TX=446.0000MHz, RX=446.0000MHz, CTCSS=100.0Hz
   [INFO] WiFi connected. IP: 192.168.1.150
   [INFO] Web dashboard listening on http://192.168.1.150
   ```
2. Open a browser to `http://192.168.1.150` to inspect real-time COS/PTT status, transmit DTMF commands, or connect to an AllStarLink node.
