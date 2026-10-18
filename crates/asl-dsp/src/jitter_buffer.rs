//! Adaptive Jitter Buffer for IAX2 VoIP Voice Packets
//!
//! Reorders out-of-order packets, absorbs network packet latency jitter,
//! and provides smooth 20ms audio frame chunks (160 samples @ 8 kHz) to the audio pipeline.

use alloc::collections::VecDeque;

/// A single audio packet stored in the jitter buffer
#[derive(Debug, Clone)]
pub struct JitterPacket {
    pub timestamp_ms: u32,
    pub payload: [u8; 160],
}

/// Adaptive Jitter Buffer configuration
#[derive(Debug, Clone)]
pub struct JitterBufferConfig {
    /// Target nominal buffer delay in milliseconds (e.g. 60ms = 3 frames)
    pub target_delay_ms: u32,
    /// Maximum allowable buffer capacity in milliseconds before dropping older packets
    pub max_capacity_ms: u32,
}

impl Default for JitterBufferConfig {
    fn default() -> Self {
        Self {
            target_delay_ms: 60,
            max_capacity_ms: 200,
        }
    }
}

pub struct JitterBuffer {
    config: JitterBufferConfig,
    queue: VecDeque<JitterPacket>,
    playout_initialized: bool,
    base_timestamp_ms: u32,
    packets_dropped: u32,
    packets_received: u32,
}

impl JitterBuffer {
    pub fn new(config: JitterBufferConfig) -> Self {
        Self {
            config,
            queue: VecDeque::with_capacity(16),
            playout_initialized: false,
            base_timestamp_ms: 0,
            packets_dropped: 0,
            packets_received: 0,
        }
    }

    /// Insert an incoming voice frame with its IAX2 timestamp
    pub fn put_frame(&mut self, timestamp_ms: u32, payload: &[u8]) {
        self.packets_received = self.packets_received.saturating_add(1);

        let mut packet_data = [0u8; 160];
        let len = payload.len().min(160);
        packet_data[..len].copy_from_slice(&payload[..len]);

        let packet = JitterPacket {
            timestamp_ms,
            payload: packet_data,
        };

        if self.queue.is_empty() {
            self.queue.push_back(packet);
            return;
        }

        // Drop if older than oldest entry when playout has started
        if self.playout_initialized && timestamp_ms < self.base_timestamp_ms {
            self.packets_dropped = self.packets_dropped.saturating_add(1);
            return;
        }

        // Ordered insertion based on timestamp
        let mut insert_idx = self.queue.len();
        for (i, p) in self.queue.iter().enumerate() {
            if timestamp_ms < p.timestamp_ms {
                insert_idx = i;
                break;
            } else if timestamp_ms == p.timestamp_ms {
                // Duplicate packet, ignore
                return;
            }
        }

        self.queue.insert(insert_idx, packet);

        // Cap buffer depth if network jitter burst exceeded max capacity
        let max_packets = (self.config.max_capacity_ms / 20) as usize;
        while self.queue.len() > max_packets {
            self.queue.pop_front();
            self.packets_dropped = self.packets_dropped.saturating_add(1);
        }
    }

    /// Retrieve the next 20ms voice frame for the DAC/speaker callback.
    /// Returns Some(payload) if frame is ready, or None for packet loss concealment (silence).
    pub fn get_frame(&mut self) -> Option<[u8; 160]> {
        if !self.playout_initialized {
            // Buffer initial delay to absorb startup jitter
            let min_initial_packets = (self.config.target_delay_ms / 20).max(1) as usize;
            if self.queue.len() < min_initial_packets {
                return None;
            }
            if let Some(front) = self.queue.front() {
                self.base_timestamp_ms = front.timestamp_ms;
                self.playout_initialized = true;
            }
        }

        if let Some(packet) = self.queue.pop_front() {
            self.base_timestamp_ms = packet.timestamp_ms.wrapping_add(20);
            Some(packet.payload)
        } else {
            // Buffer underrun
            self.base_timestamp_ms = self.base_timestamp_ms.wrapping_add(20);
            None
        }
    }

    /// Reset jitter buffer state (e.g. on new PTT talkspurt)
    pub fn reset(&mut self) {
        self.queue.clear();
        self.playout_initialized = false;
        self.base_timestamp_ms = 0;
    }

    pub fn depth(&self) -> usize {
        self.queue.len()
    }

    pub fn stats(&self) -> (u32, u32) {
        (self.packets_received, self.packets_dropped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jitter_buffer_ordering_and_playout() {
        let config = JitterBufferConfig {
            target_delay_ms: 40, // 2 frames
            max_capacity_ms: 100,
        };
        let mut jb = JitterBuffer::new(config);

        // Deliver packets out-of-order: timestamp 20 first, then 0, then 40
        let p20 = [20u8; 160];
        let p0 = [1u8; 160];
        let p40 = [40u8; 160];

        jb.put_frame(20, &p20);
        jb.put_frame(0, &p0);
        jb.put_frame(40, &p40);

        assert_eq!(jb.depth(), 3);

        // First playout: should pop timestamp 0
        let f1 = jb.get_frame().expect("expected frame 0");
        assert_eq!(f1[0], 1);

        // Second playout: timestamp 20
        let f2 = jb.get_frame().expect("expected frame 20");
        assert_eq!(f2[0], 20);

        // Third playout: timestamp 40
        let f3 = jb.get_frame().expect("expected frame 40");
        assert_eq!(f3[0], 40);

        // Queue empty -> packet loss / silence
        assert_eq!(jb.get_frame(), None);
    }
}
