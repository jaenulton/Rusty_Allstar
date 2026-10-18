# Rusty_Allstar Micro-Node Hardware Reference Design Specification

This document contains the complete engineering specification, Bill of Materials (BOM), KiCad netlist interconnects, and PCB layout guidelines for the **Rusty_Allstar ESP32 Pocket Micro-Node**.

---

## 1. System Overview

The Rusty_Allstar Micro-Node is an open-hardware, standalone AllStarLink radio node combining:
- **Espressif ESP32-WROOM-32E / ESP32-S3** microcontroller (dual-core 240 MHz, 8 MB flash, Wi-Fi 802.11 b/g/n).
- **NiceRF SA818S** transceiver module (VHF 134–174 MHz or UHF 400–480 MHz, 1 Watt RF output).
- **I2S Audio Codec**: PCM5102A DAC (Transmit Audio) + ES8388 or MAX9814/ADC (Receive Audio) with clean ground plane.
- **Power Subsystem**: USB-C 5V input, ultra-low-noise 3.3V LDO for the analog audio / RF section, separate digital 3.3V rail.
- **RF Output**: 50Ω SMA female connector with 5-pole Chebyshev low-pass harmonic suppression filter.

```
                      +---------------------------------------+
                      |               USB-C 5V                |
                      +-------------------+-------------------+
                                          |
                      +-------------------+-------------------+
                      |   AP2112K 3.3V LDO (Ultra Low Noise)  |
                      +-------------------+-------------------+
                                          |
        +---------------------------------+---------------------------------+
        |                                 |                                 |
+-------v-------+                 +-------v-------+                 +-------v-------+
|  ESP32-WROOM  |<-- UART / AT -->| NiceRF SA818S |<--- RF 50Ω ---->| SMA Connector |
|  Dual-Core    |<-- GPIO COS/PTT-| 1W VHF/UHF    |                 |   Antenna     |
|  Controller   |<-- I2S Audio -->| Audio In/Out  |                 +---------------+
+---------------+                 +---------------+
```

---

## 2. Bill of Materials (BOM)

| Ref Des | Component | Description / Footprint | Value / Part Number |
|---|---|---|---|
| **U1** | Microcontroller | ESP32-WROOM-32E (16MB Flash) | Espressif ESP32-WROOM-32E-N16 |
| **U2** | RF Transceiver | NiceRF SA818S (UHF or VHF module) | NiceRF SA818S-U / SA818S-V |
| **U3** | Audio DAC | 32-bit I2S Stereo Audio DAC, TSSOP-20 | TI PCM5102A |
| **U4** | 3.3V Digital LDO | 600mA Low Dropout Linear Regulator, SOT-23-5 | Diodes Inc AP2112K-3.3TRG1 |
| **U5** | 3.3V Analog LDO | Ultra-low noise RF LDO, SOT-23-5 | Microchip MIC5219-3.3YM5 |
| **Q1** | PTT Buffer | NPN BJT or N-Channel MOSFET, SOT-23 | 2N7002 / MMBT3904 |
| **J1** | Power / Serial | USB-C 16-pin receptacle, Receptacle SMT | USB4105-GF-A |
| **J2** | RF Output | SMA Female Edge-Mount Receptacle, 1.6mm PCB | Amphenol 132134 |
| **L1, L2**| RF LPF | 0805 High-Q RF Inductors | 22 nH (UHF) / 56 nH (VHF) |
| **C1–C5**| RF LPF | 0603 C0G/NP0 RF Capacitors | 10 pF, 18 pF, 10 pF (UHF) |
| **D1, D2**| Status LEDs | 0603 SMD LED | Green (COS) / Red (PTT) |
| **R1, R2**| Pull-ups | 0603 Resistors | 10 kΩ 1% |

---

## 3. KiCad Netlist & Wiring Table

### 3.1 ESP32 to SA818S Transceiver
| Net Name | ESP32 GPIO | SA818S Pin | Notes |
|---|---|---|---|
| `SA818_TXD` | `GPIO 17` (UART2 TX) | `RXD` (Pin 17) | 9600 Baud AT command TX |
| `SA818_RXD` | `GPIO 16` (UART2 RX) | `TXD` (Pin 16) | 9600 Baud AT command RX |
| `RADIO_COS` | `GPIO 34` (GPI input) | `SQ` / COS (Pin 13) | Active-low squelch with 10k pull-up to 3V3 |
| `RADIO_PTT` | `GPIO 25` (GPO output) | `PTT` (Pin 15) | Active-low keying via Q1 buffer |
| `RADIO_PDN` | `GPIO 26` (GPO output) | `PDN` (Pin 18) | Sleep/Power control (HIGH = run, LOW = sleep) |

### 3.2 ESP32 to I2S Audio DAC (PCM5102A)
| Net Name | ESP32 GPIO | PCM5102A Pin | Notes |
|---|---|---|---|
| `I2S_BCLK` | `GPIO 27` | `BCK` (Pin 12) | Bit Clock (256 kHz @ 8 kHz / 16-bit mono) |
| `I2S_WS` | `GPIO 22` | `LRCK` (Pin 11) | Word Select / Frame Sync (8 kHz) |
| `I2S_DOUT` | `GPIO 19` | `DIN` (Pin 13) | Audio data out -> filtered -> SA818 MIC_IN |
| `I2S_DIN` | `GPIO 35` | ADC / Codec DOUT | Filtered SA818 AUDIO_OUT -> ADC -> ESP32 |

---

## 4. PCB Layout & Grounding Guidelines

1. **4-Layer Stackup**:
   - Layer 1 (Top): High-frequency RF traces (50Ω coplanar waveguide) & I2S digital signals.
   - Layer 2 (Ground Plane): Solid, unbroken GND plane serving as RF return.
   - Layer 3 (Power Plane): Split 5V and isolated 3.3V analog/digital power islands.
   - Layer 4 (Bottom): Non-critical DC control traces and LED status lines.
2. **RF Guard Ring**:
   - Provide via stitching (0.5mm drill, 1.5mm pitch) along the 50Ω antenna feedline from the SA818S RF OUT pin through the low-pass filter to the SMA connector.
3. **Decoupling**:
   - Place 0.1 μF and 10 μF MLCC ceramic capacitors directly adjacent to the SA818 power pins to absorb RF amplifier transmission transients (up to 750mA peak during 1W TX).
