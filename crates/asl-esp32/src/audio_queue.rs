//! Inter-Core Audio FIFO Queue
//! Transfers 20ms G.711 frames (160 bytes) between Core 0 (Network) and Core 1 (Audio DSP).

pub const FRAME_SIZE: usize = 160;
pub const QUEUE_CAPACITY: usize = 8; // ~160ms jitter buffer

#[derive(Clone, Copy)]
pub struct AudioFrame {
    pub data: [u8; FRAME_SIZE],
    pub len: usize,
}

impl Default for AudioFrame {
    fn default() -> Self {
        Self {
            data: [0u8; FRAME_SIZE],
            len: 0,
        }
    }
}

pub struct AudioRingBuffer {
    buffer: [AudioFrame; QUEUE_CAPACITY],
    head: usize,
    tail: usize,
    count: usize,
}

impl AudioRingBuffer {
    pub const fn new() -> Self {
        Self {
            buffer: [AudioFrame {
                data: [0u8; FRAME_SIZE],
                len: 0,
            }; QUEUE_CAPACITY],
            head: 0,
            tail: 0,
            count: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn is_full(&self) -> bool {
        self.count == QUEUE_CAPACITY
    }

    pub fn len(&self) -> usize {
        self.count
    }

    /// Push an audio frame into the buffer. Overwrites oldest frame if full.
    pub fn push(&mut self, frame: AudioFrame) {
        if self.is_full() {
            // Drop oldest
            self.tail = (self.tail + 1) % QUEUE_CAPACITY;
            self.count -= 1;
        }

        self.buffer[self.head] = frame;
        self.head = (self.head + 1) % QUEUE_CAPACITY;
        self.count += 1;
    }

    /// Pop an audio frame from the buffer
    pub fn pop(&mut self) -> Option<AudioFrame> {
        if self.is_empty() {
            None
        } else {
            let frame = self.buffer[self.tail];
            self.tail = (self.tail + 1) % QUEUE_CAPACITY;
            self.count -= 1;
            Some(frame)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_ring_buffer() {
        let mut q = AudioRingBuffer::new();
        assert!(q.is_empty());

        let mut frame = AudioFrame::default();
        frame.data[0] = 42;
        frame.len = 160;

        q.push(frame);
        assert_eq!(q.len(), 1);

        let popped = q.pop().unwrap();
        assert_eq!(popped.data[0], 42);
        assert!(q.is_empty());
    }
}
