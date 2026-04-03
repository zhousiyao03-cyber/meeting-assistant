use anyhow::Result;
use std::path::Path;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub struct WhisperEngine {
    ctx: WhisperContext,
}

impl WhisperEngine {
    /// Load a Whisper model from the given path.
    pub fn new(model_path: &Path) -> Result<Self> {
        let params = WhisperContextParameters::default();
        let ctx = WhisperContext::new_with_params(
            model_path.to_str().unwrap_or_default(),
            params,
        )?;
        Ok(Self { ctx })
    }

    /// Check if audio chunk is mostly silence (RMS below threshold)
    pub fn is_silence(audio: &[f32]) -> bool {
        if audio.is_empty() {
            return true;
        }
        let rms = (audio.iter().map(|s| s * s).sum::<f32>() / audio.len() as f32).sqrt();
        eprintln!("[whisper] RMS: {:.6}", rms);
        rms < 0.02
    }

    /// Transcribe a chunk of 16kHz mono f32 audio.
    /// Returns the recognized text.
    pub fn transcribe(&self, audio: &[f32]) -> Result<String> {
        // Skip silence to avoid hallucinations
        if Self::is_silence(audio) {
            return Ok(String::new());
        }

        let mut state = self.ctx.create_state()?;
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        params.set_language(Some("zh"));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_suppress_blank(true);
        params.set_suppress_non_speech_tokens(true);

        state.full(params, audio)?;

        let num_segments = state.full_n_segments()?;
        let mut text = String::new();
        for i in 0..num_segments {
            if let Ok(segment) = state.full_get_segment_text(i) {
                text.push_str(&segment);
            }
        }

        let trimmed = text.trim().to_string();

        if Self::is_garbage(&trimmed) {
            return Ok(String::new());
        }

        Ok(trimmed)
    }

    fn is_garbage(text: &str) -> bool {
        if text.is_empty() {
            return true;
        }
        let char_count = text.chars().count();
        if char_count < 4 {
            return true;
        }
        // High ratio of non-CJK, non-ASCII printable chars = garbled
        let total = char_count as f32;
        let garbage_chars = text.chars().filter(|c| {
            !c.is_ascii_alphanumeric()
                && !c.is_ascii_punctuation()
                && !c.is_ascii_whitespace()
                && !('\u{4e00}'..='\u{9fff}').contains(c)
                && !('\u{3000}'..='\u{303f}').contains(c)
                && !('\u{ff00}'..='\u{ffef}').contains(c)
                && !('\u{3400}'..='\u{4dbf}').contains(c)
        }).count() as f32;
        if garbage_chars / total > 0.3 {
            return true;
        }
        // Whisper hallucination patterns on silence
        let lower = text.to_lowercase();
        let hallucinations = [
            "thank you", "thanks for watching", "subscribe",
            "please subscribe", "like and subscribe",
            "you", "the", "i'm", "okay", "bye",
            "字幕", "由 amara", "请不吝", "谢谢观看",
            "...", "♪", "music",
        ];
        for pat in &hallucinations {
            if lower.trim() == *pat || (lower.contains(pat) && char_count < 15) {
                return true;
            }
        }
        false
    }
}
