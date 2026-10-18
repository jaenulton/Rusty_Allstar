# Rusty_Allstar ESP32 Hardware Guide

This document describes how to connect an **ESP32** (or **ESP32-S3**) microcontroller to radio equipment, audio codecs, and RF modules.

---

## 1. System Overview

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
                               \                  /
                             +----------------------+
                             | Transceiver / SA818  |
                             +----------------------+
```

---

## 2. Audio Interfacing (I2S DMA)

Standard AllStarLink uses 8,000 Hz, 16-bit linear PCM audio.
The ESP32 I2S hardware peripheral provides dedicated DMA buffers, streaming samples without CPU intervention.

### Option A: ES8388 Audio Codec (Full-Duplex Stereo ADC + DAC)
Common on boards like the **ESP32-Audio-Kit / LyraT**:
* **I2S Master Clock (MCLK)**: GPIO 0
* **I2S Bit Clock (BCLK)**: GPIO 27
* **I2S Word Select (LRCK / WS)**: GPIO 25
* **I2S Data Out (TX to Radio)**: GPIO 26
* **I2S Data In (RX from Radio)**: GPIO 35
* **I2C Control (SDA / SCL)**: GPIO 18 / GPIO 23

### Option B: Separate DAC + ADC Modules
* **DAC (Audio Out to Transmitter)**: PCM5102A or MAX98357A I2S DAC
* **ADC (Audio In from Receiver)**: PCM1808 I2S ADC or INMP441 MEMS microphone (for acoustic testing)

---

## 3. Radio Control Lines (COS & PTT)

### Push-To-Talk (PTT Output)
* **Pin**: GPIO 4
* **Drive Circuit**: NPN Transistor (2N2222 / 2N3904) or N-channel MOSFET (2N7000):
  * ESP32 GPIO 4 -> 1 kΩ Resistor -> Base of 2N2222.
  * Emitter -> Radio Ground.
  * Collector -> Radio PTT line (pulled up internally in radio).
  * Diode protection (1N4148) across collector/emitter if switching inductive relay lines.

### Carrier Operated Squelch (COS / COR Input)
* **Pin**: GPIO 5 (Configured with internal pull-down or pull-up and hardware interrupt)
* **Drive Circuit**: PC817 Optocoupler or 2N3904 inverter:
  * Radio COS line (0V / 5V or 0V / 12V) drives optocoupler LED via current limiting resistor.
  * Phototransistor pulls ESP32 GPIO 5 to Ground when carrier is present.

---

## 4. Integrated RF Module (NiceRF SA818 / SR-FRS)

For a self-contained, pocket-sized AllStarLink micro-node (RF Hotspot), you can wire an **SA818 VHF or UHF module** directly:
* **ESP32 UART TX (GPIO 16)** -> SA818 RXD (Frequency & CTCSS programming commands)
* **ESP32 UART RX (GPIO 17)** -> SA818 TXD
* **ESP32 GPIO 4** -> SA818 PTT
* **SA818 Squelch (SQ)** -> ESP32 GPIO 5 (COS)
* **ESP32 I2S DAC Audio Out** -> SA818 MIC_IN
* **SA818 AUDIO_OUT** -> ESP32 I2S ADC Audio In
