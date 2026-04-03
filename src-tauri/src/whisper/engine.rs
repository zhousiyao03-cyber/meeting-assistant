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

    /// Transcribe a chunk of 16kHz mono f32 audio.
    /// Returns the recognized text.
    pub fn transcribe(&self, audio: &[f32]) -> Result<String> {
        let mut state = self.ctx.create_state()?;
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        // Auto-detect language (supports Chinese + English mix)
        params.set_language(None);
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

        Ok(text.trim().to_string())
    }
}
