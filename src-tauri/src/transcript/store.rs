use chrono::{DateTime, Utc};
use serde::Serialize;
use std::sync::{Arc, Mutex};

#[derive(Clone, Serialize, Debug)]
pub struct TranscriptSegment {
    pub timestamp: DateTime<Utc>,
    pub text: String,
    /// Seconds since recording started
    pub offset_secs: f64,
}

pub struct TranscriptStore {
    segments: Vec<TranscriptSegment>,
}

impl TranscriptStore {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    pub fn add(&mut self, text: String, offset_secs: f64) {
        if text.is_empty() {
            return;
        }
        self.segments.push(TranscriptSegment {
            timestamp: Utc::now(),
            text,
            offset_secs,
        });
    }

    /// Get all segments.
    pub fn all(&self) -> &[TranscriptSegment] {
        &self.segments
    }

    /// Get text from the last N seconds.
    pub fn recent_text(&self, last_n_seconds: f64) -> String {
        if self.segments.is_empty() {
            return String::new();
        }
        let latest_offset = self.segments.last().unwrap().offset_secs;
        let cutoff = latest_offset - last_n_seconds;
        self.segments
            .iter()
            .filter(|s| s.offset_secs >= cutoff)
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Get full transcript as one string.
    pub fn full_text(&self) -> String {
        self.segments
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn clear(&mut self) {
        self.segments.clear();
    }
}

pub type SharedTranscriptStore = Arc<Mutex<TranscriptStore>>;

pub fn create_shared_store() -> SharedTranscriptStore {
    Arc::new(Mutex::new(TranscriptStore::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recent_text() {
        let mut store = TranscriptStore::new();
        store.add("hello".into(), 0.0);
        store.add("world".into(), 5.0);
        store.add("foo".into(), 10.0);
        store.add("bar".into(), 35.0);

        let recent = store.recent_text(30.0);
        assert!(recent.contains("world"));
        assert!(recent.contains("foo"));
        assert!(recent.contains("bar"));
        assert!(!recent.contains("hello"));
    }
}
