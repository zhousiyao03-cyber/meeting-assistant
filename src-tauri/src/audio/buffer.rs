use std::sync::{Arc, Mutex};

/// Ring buffer that accumulates PCM f32 samples at 16kHz mono.
/// Consumers drain chunks of `chunk_size` samples (e.g., 5s = 80000 samples).
pub struct AudioBuffer {
    data: Vec<f32>,
    chunk_size: usize,
}

impl AudioBuffer {
    /// Create a new buffer. `chunk_seconds` defines how many seconds per chunk.
    pub fn new(chunk_seconds: usize, sample_rate: usize) -> Self {
        Self {
            data: Vec::with_capacity(sample_rate * chunk_seconds * 2),
            chunk_size: sample_rate * chunk_seconds,
        }
    }

    /// Push samples into the buffer.
    pub fn push(&mut self, samples: &[f32]) {
        self.data.extend_from_slice(samples);
    }

    /// Drain a full chunk if available. Returns None if not enough data yet.
    pub fn drain_chunk(&mut self) -> Option<Vec<f32>> {
        if self.data.len() >= self.chunk_size {
            let chunk: Vec<f32> = self.data.drain(..self.chunk_size).collect();
            Some(chunk)
        } else {
            None
        }
    }

    /// How many samples are buffered.
    pub fn len(&self) -> usize {
        self.data.len()
    }
}

pub type SharedBuffer = Arc<Mutex<AudioBuffer>>;

pub fn create_shared_buffer(chunk_seconds: usize, sample_rate: usize) -> SharedBuffer {
    Arc::new(Mutex::new(AudioBuffer::new(chunk_seconds, sample_rate)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_drain() {
        let mut buf = AudioBuffer::new(1, 16000); // 1 second chunks at 16kHz
        let samples = vec![0.5f32; 8000]; // 0.5 seconds
        buf.push(&samples);
        assert_eq!(buf.drain_chunk(), None); // not enough

        buf.push(&samples); // now 1 second
        let chunk = buf.drain_chunk().unwrap();
        assert_eq!(chunk.len(), 16000);
        assert_eq!(buf.len(), 0);
    }
}
