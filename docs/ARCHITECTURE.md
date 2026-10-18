# Rusty_Allstar Architecture Specification

## 1. Vision & Purpose
AllStarLink (ASL) is a popular amateur radio VoIP linking network historically based on Asterisk PBX with the `app_rpt` channel driver. While robust, the Asterisk + DAHDI C codebase carries over two decades of legacy telephony complexity, high memory consumption, kernel module fragility, and concurrency lockups.

**Rusty_Allstar** is a clean-room Rust implementation of the AllStarLink radio node logic, designed from the ground up for:
1. **Low Resource Footprint**: Capable of running on an embedded **ESP32 microcontroller** (240 MHz Xtensa/RISC-V with ~320 KB heap + optional PSRAM).
2. **Fearless Concurrency & Clean State Machines**: Replacing Asterisk POSIX mutexes and dialplans with typed Rust actor channels and deterministic repeater states.
3. **No DAHDI Kernel Dependency**: User-space timing and direct hardware audio peripheral integration (I2S DMA on ESP32, ALSA/cpal on Linux).
4. **Modularity**: Separation of the IAX2 protocol, DSP pipeline, and repeater controller into reusable, testable crates.

---

## 2. Multi-Crate Architecture

```
Rusty_Allstar/
├── crates/
│   ├── iax2-proto/        # [no_std + alloc] RFC 5456 IAX2 packet parser & serializer
│   ├── asl-dsp/           # [no_std + alloc] G.711 μ/A-law, Goertzel DTMF/CTCSS, biquad filters
│   ├── asl-core/          # [no_std + alloc] Repeater controller: PTT/COS state, CW ID, courtesy tones
│   ├── asl-node-cli/      # Desktop test runner & bridge using Tokio & cpal/ALSA
│   └── asl-esp32/         # ESP32 firmare target (esp-idf-svc / esp-hal, I2S audio, WiFi)
└── docs/                  # Architectural specs, hardware guides, and protocol docs
```

### 2.1 `iax2-proto`
- Pure, zero-copy (or zero unnecessary allocation) binary parser for IAX2 (RFC 5456).
- Supports:
  - **Full Frames** (Protocol control, call setup, audio headers, sequence numbers, timestamps).
  - **Mini Frames** (Low-overhead 4-byte header for consecutive voice packets).
  - **Information Elements (IEs)**: CALLED_NUMBER, CALLING_NUMBER, USERNAME, MD5_CHALLENGE, MD5_RESULT, FORMAT, etc.
- No network I/O; pure state machine and byte buffer transformations suitable for unit testing with Wireshark PCAPs.

### 2.2 `asl-dsp`
- **Audio Codecs**:
  - G.711 μ-law & A-law (standard ASL voice format; fast table lookups, minimal CPU).
  - Optional Speex / GSM / Opus for bandwidth-constrained links.
- **Tone Processing**:
  - **Goertzel Algorithm**: Dual-Tone Multi-Frequency (DTMF) detection and Continuous Tone-Coded Squelch System (CTCSS / PL) detection.
  - **Tone Generation**: Single/dual tone generators for courtesy tones, repeater telemetry, and CW (Morse code) station identification.
- **Filters**:
  - 300 Hz – 3000 Hz audio bandpass.
  - De-emphasis and pre-emphasis biquad IIR filters for direct discriminator/mic audio interfacing.

### 2.3 `asl-core`
- **Repeater Controller State Machine**:
  - Carrier Operated Squelch (COS) & Continuous Tone Squelch (CTCSS) inputs.
  - Push-To-Talk (PTT) output with configurable anti-kerchunking, hang time, and timeout timers (TOT).
  - CW Identifier with priority queue over audio channels.
  - Courtesy tone sequencer.
  - Command interpreter for DTMF radio control (*1, *2, *3 link commands).

---

## 3. ESP32 Resource Budget

| Subsystem | Estimated Compute | Estimated Memory |
| :--- | :--- | :--- |
| **IAX2 Streaming (G.711)** | ~2% Core 0 | ~15 KB (buffers & call state) |
| **Goertzel DTMF/CTCSS** | ~3–5% Core 1 | < 2 KB |
| **I2S Audio DMA Buffers** | Hardware DMA | ~4 KB |
| **Repeater Logic & Timers**| < 1% Core 1 | ~5 KB |
| **LwIP WiFi / Network** | Handled by ESP-IDF | ~40–60 KB |
| **Total Headroom** | **> 85% CPU Free** | **Comfortably < 100 KB total heap** |

---

## 4. Hardware Interfacing Options

1. **Direct I2S Audio Codec**:
   - ESP32 connects via I2S to an audio codec (e.g. ES8388, PCM5102 DAC + INMP441 ADC, or WM8960).
   - Audio input attaches to receiver discriminator / flat audio.
   - Audio output attaches to transmitter mic input.
2. **Integrated RF Module**:
   - ESP32 controls an onboard transceiver module like the NiceRF SA818 / SR-FRS via UART + I2S/analog for an ultra-compact single-board node.
3. **GPIO PTT & COS**:
   - COS/COR input with optocoupler or level shifter to ESP32 interrupt pin.
   - PTT output via transistor / MOSFET to transmitter PTT line.
