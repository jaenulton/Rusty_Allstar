//! Core 1: Real-Time Audio DSP and Repeater Controller Engine
//! Pinned to Core 1 in FreeRTOS to prevent network / WiFi jitter from affecting audio DMA.

use crate::audio_queue::{AudioFrame, AudioRingBuffer, FRAME_SIZE};
use crate::radio_driver::RadioGpio;
use asl_core::{Action, RepeaterConfig, RepeaterController};
use asl_dsp::{decode_ulaw_slice, encode_ulaw_slice, CwGenerator, DtmfDetector};

pub struct AudioDspEngine<G: RadioGpio> {
    pub radio_gpio: G,
    pub controller: RepeaterController,
    pub dtmf: DtmfDetector,
    pub cw: CwGenerator,
    pub last_cos: bool,
}

impl<G: RadioGpio> AudioDspEngine<G> {
    pub fn new(radio_gpio: G, config: RepeaterConfig) -> Self {
        let controller = RepeaterController::new(config);
        let dtmf = DtmfDetector::new(8000.0, 205);
        let cw = CwGenerator::new(8000.0, 800.0, 20);

        Self {
            radio_gpio,
            controller,
            dtmf,
            cw,
            last_cos: false,
        }
    }

    /// Process a single 20ms audio frame cycle (160 samples @ 8 kHz)
    pub fn process_cycle(
        &mut self,
        rx_pcm_in: &[i16; FRAME_SIZE],
        tx_pcm_out: &mut [i16; FRAME_SIZE],
        inbound_net_queue: &mut AudioRingBuffer,
        outbound_net_queue: &mut AudioRingBuffer,
    ) {
        // 1. Poll Radio Hardware COS
        let cos_current = self.radio_gpio.read_cos();
        if cos_current != self.last_cos {
            self.last_cos = cos_current;
            if let Some(action) = self.controller.on_cos_event(cos_current) {
                self.handle_controller_action(action);
            }
        }

        // 2. Advance Repeater Controller Timers by 20ms
        if let Some(action) = self.controller.tick(20) {
            self.handle_controller_action(action);
        }

        // 3. Process Inbound Radio Audio (RX)
        // Feed audio samples into DTMF decoder
        for &s in rx_pcm_in.iter() {
            if let Some(_key) = self.dtmf.process_sample(s) {
                // DTMF digit detected on RF
            }
        }

        // If local receiver has carrier, encode μ-law and push to network queue
        if cos_current {
            let mut frame = AudioFrame::default();
            frame.len = FRAME_SIZE;
            encode_ulaw_slice(rx_pcm_in, &mut frame.data);
            outbound_net_queue.push(frame);
        }

        // 4. Process Outbound Radio Audio (TX)
        // Check if there is incoming network audio from AllStarLink
        if let Some(net_frame) = inbound_net_queue.pop() {
            // Signal controller that network audio is active
            if let Some(action) = self.controller.on_network_rx_event(true) {
                self.handle_controller_action(action);
            }

            // Decode G.711 u-law from network to PCM output
            decode_ulaw_slice(&net_frame.data[..net_frame.len], tx_pcm_out);
        } else {
            // No network audio; clear TX audio buffer
            tx_pcm_out.fill(0);
        }

        // 5. Update Hardware PTT Output
        let ptt = self.controller.is_ptt_active();
        self.radio_gpio.set_ptt(ptt);
        self.radio_gpio.set_led(ptt);
    }

    fn handle_controller_action(&mut self, action: Action) {
        match action {
            Action::SetPtt(active) => {
                self.radio_gpio.set_ptt(active);
                self.radio_gpio.set_led(active);
            }
            Action::PlayCourtesyTone => {
                // Courtesy tone trigger
            }
            Action::PlayCwId(_call) => {
                // CW ID trigger
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::radio_driver::MockRadioDriver;

    #[test]
    fn test_audio_dsp_engine_cycle() {
        let gpio = MockRadioDriver::new();
        let config = RepeaterConfig::default();
        let mut engine = AudioDspEngine::new(gpio, config);

        let mut in_queue = AudioRingBuffer::new();
        let mut out_queue = AudioRingBuffer::new();

        let rx_in = [0i16; FRAME_SIZE];
        let mut tx_out = [0i16; FRAME_SIZE];

        // 1. Silent cycle in idle
        engine.process_cycle(&rx_in, &mut tx_out, &mut in_queue, &mut out_queue);
        assert_eq!(engine.radio_gpio.ptt_state, false);

        // 2. User keys up on RF
        engine.radio_gpio.cos_state = true;
        engine.process_cycle(&rx_in, &mut tx_out, &mut in_queue, &mut out_queue);
        assert_eq!(engine.radio_gpio.ptt_state, true);
        assert_eq!(out_queue.len(), 1); // Encoded u-law pushed to network queue
    }
}
