//! AllStarLink Conference Bridge & Multi-Node Audio Router
//!
//! Maintains a list of interconnected AllStarLink nodes, tracks link permissions
//! (Transceive vs Monitor-Only), and routes audio packets across all connected links.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use asl_dsp::{AudioMixer, AudioStreamSource, SAMPLES_PER_FRAME};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkMode {
    /// Full duplex transmit and receive
    Transceive,
    /// Receive only from remote node; do not transmit back
    Monitor,
}

#[derive(Debug, Clone)]
pub struct ConnectedPeer {
    pub call_id: u32,
    pub node_number: String,
    pub mode: LinkMode,
    pub last_rx_frame: Option<[i16; SAMPLES_PER_FRAME]>,
    pub rx_active: bool,
}

pub struct ConferenceBridge {
    local_node: String,
    peers: BTreeMap<u32, ConnectedPeer>,
    mixer: AudioMixer,
}

impl ConferenceBridge {
    pub fn new(local_node: impl Into<String>) -> Self {
        Self {
            local_node: local_node.into(),
            peers: BTreeMap::new(),
            mixer: AudioMixer::new(),
        }
    }

    pub fn local_node(&self) -> &str {
        &self.local_node
    }

    pub fn add_peer(&mut self, call_id: u32, node_number: impl Into<String>, mode: LinkMode) {
        self.peers.insert(
            call_id,
            ConnectedPeer {
                call_id,
                node_number: node_number.into(),
                mode,
                last_rx_frame: None,
                rx_active: false,
            },
        );
    }

    pub fn remove_peer(&mut self, call_id: u32) -> Option<ConnectedPeer> {
        self.peers.remove(&call_id)
    }

    pub fn peer_count(&self) -> usize {
        self.peers.len()
    }

    pub fn get_peer(&self, call_id: u32) -> Option<&ConnectedPeer> {
        self.peers.get(&call_id)
    }

    /// Feeds an inbound 20ms audio frame from a specific connected peer
    pub fn push_peer_audio(&mut self, call_id: u32, pcm_samples: [i16; SAMPLES_PER_FRAME]) {
        if let Some(peer) = self.peers.get_mut(&call_id) {
            peer.last_rx_frame = Some(pcm_samples);
            peer.rx_active = true;
        }
    }

    /// Marks a peer as no longer actively talking (silence)
    pub fn set_peer_inactive(&mut self, call_id: u32) {
        if let Some(peer) = self.peers.get_mut(&call_id) {
            peer.rx_active = false;
        }
    }

    /// Computes the outbound audio frames for every transceiving peer (N-1 mix),
    /// as well as the master mix for the local repeater transmitter.
    pub fn process_audio_cycle(
        &mut self,
        local_mic: Option<[i16; SAMPLES_PER_FRAME]>,
        master_out: &mut [i16; SAMPLES_PER_FRAME],
    ) -> Vec<(u32, [i16; SAMPLES_PER_FRAME])> {
        let mut sources = Vec::new();

        // Local microphone source (call_id 0)
        if let Some(mic) = local_mic {
            sources.push(AudioStreamSource {
                call_id: 0,
                pcm_samples: mic,
                active: true,
            });
        }

        // Add all active peer sources
        for peer in self.peers.values() {
            if peer.rx_active {
                if let Some(samples) = peer.last_rx_frame {
                    sources.push(AudioStreamSource {
                        call_id: peer.call_id,
                        pcm_samples: samples,
                        active: true,
                    });
                }
            }
        }

        // Master output for local repeater transmitter
        self.mixer.mix_master(&sources, master_out);

        // Compute individual outputs for peers that are in Transceive mode
        let mut peer_outputs: Vec<(u32, [i16; SAMPLES_PER_FRAME])> = self
            .peers
            .values()
            .filter(|p| p.mode == LinkMode::Transceive)
            .map(|p| (p.call_id, [0i16; SAMPLES_PER_FRAME]))
            .collect();

        self.mixer.mix_n_minus_one(&sources, &mut peer_outputs);

        peer_outputs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conference_bridge_linking_and_audio() {
        let mut bridge = ConferenceBridge::new("1999");
        bridge.add_peer(101, "50000", LinkMode::Transceive);
        bridge.add_peer(102, "50001", LinkMode::Transceive);
        bridge.add_peer(103, "50002", LinkMode::Monitor);

        assert_eq!(bridge.peer_count(), 3);

        // Peer 101 transmits audio frame of value 500
        bridge.push_peer_audio(101, [500i16; SAMPLES_PER_FRAME]);

        let mut master = [0i16; SAMPLES_PER_FRAME];
        let outbound = bridge.process_audio_cycle(None, &mut master);

        // Master should have audio from 101
        assert_eq!(master[0], 500);

        // Outbound should only be sent to Transceive peers (101 and 102)
        assert_eq!(outbound.len(), 2);

        // Node 101 hears N-1 (itself subtracted -> 0)
        let out_101 = outbound.iter().find(|(id, _)| *id == 101).unwrap();
        assert_eq!(out_101.1[0], 0);

        // Node 102 hears Node 101's audio (500)
        let out_102 = outbound.iter().find(|(id, _)| *id == 102).unwrap();
        assert_eq!(out_102.1[0], 500);

        // Monitor peer 103 should receive no outbound transmissions
        assert!(outbound.iter().all(|(id, _)| *id != 103));
    }
}
