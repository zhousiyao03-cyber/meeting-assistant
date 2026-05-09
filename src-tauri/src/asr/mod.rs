use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub mod whisper_realtime;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AsrConfig {
    pub provider: String,        // "openai-realtime-whisper"
    pub openai_api_key: String,
    pub openai_model: String,    // e.g. "gpt-realtime-whisper"
    pub language_hint: String,   // "auto" | "zh" | "en"
}

impl Default for AsrConfig {
    fn default() -> Self {
        Self {
            provider: "openai-realtime-whisper".into(),
            openai_api_key: String::new(),
            openai_model: "gpt-realtime-whisper".into(),
            language_hint: "auto".into(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct TranscriptChunk {
    pub text: String,
    pub speaker: String,           // "me" | "other" | "mixed"
    pub offset_secs: f64,
    pub is_final: bool,
}

#[async_trait::async_trait]
pub trait AsrProvider: Send {
    async fn start(&mut self) -> Result<()>;
    async fn send_audio(&mut self, pcm_16k_mono: &[f32], speaker: &str) -> Result<()>;
    async fn stop(&mut self) -> Result<()>;
    fn provider_name(&self) -> &'static str;
}

pub type TranscriptCallback = Arc<dyn Fn(TranscriptChunk) + Send + Sync>;

pub fn create_asr_provider(
    config: &AsrConfig,
    on_transcript: TranscriptCallback,
) -> Result<Box<dyn AsrProvider>> {
    match config.provider.as_str() {
        "openai-realtime-whisper" => Ok(Box::new(
            whisper_realtime::OpenAiRealtimeWhisperProvider::new(config, on_transcript)?,
        )),
        other => Err(anyhow::anyhow!(
            "Unknown ASR provider: '{}'. MVP supports only 'openai-realtime-whisper'.",
            other
        )),
    }
}
