use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LlmConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AudioConfig {
    pub mic_device: String,
    pub capture_device: String,
    pub noise_reduction: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ByoConfig {
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub openai_api_key: String,
    #[serde(default)]
    pub anthropic_api_key: String,
    #[serde(default = "default_anthropic_model")]
    pub anthropic_model: String,
}

fn default_anthropic_model() -> String {
    "claude-sonnet-4-6".into()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub llm: LlmConfig,
    pub audio: AudioConfig,
    pub language_preference: String,
    pub analysis_mode: String,
    #[serde(default)]
    pub byo: ByoConfig,
    #[serde(default = "default_openai_asr_key")]
    pub openai_asr_api_key: String,
    #[serde(default = "default_openai_asr_model")]
    pub openai_asr_model: String,
}

fn default_openai_asr_key() -> String {
    String::new()
}
fn default_openai_asr_model() -> String {
    "gpt-realtime-whisper".into()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            llm: LlmConfig {
                base_url: "https://llmgate.io/v1".into(),
                api_key: String::new(),
                model: "claude-sonnet-4-6".into(),
            },
            audio: AudioConfig {
                mic_device: String::new(),
                capture_device: String::new(),
                noise_reduction: true,
            },
            language_preference: "auto".into(),
            analysis_mode: "balanced".into(),
            byo: ByoConfig {
                active: false,
                openai_api_key: String::new(),
                anthropic_api_key: String::new(),
                anthropic_model: "claude-sonnet-4-6".into(),
            },
            openai_asr_api_key: String::new(),
            openai_asr_model: "gpt-realtime-whisper".into(),
        }
    }
}

fn config_path() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("No home dir"))?;
    let dir = home.join(".meeting-assistant");
    fs::create_dir_all(&dir)?;
    Ok(dir.join("config.json"))
}

pub fn load_config() -> Result<AppConfig> {
    let path = config_path()?;
    if path.exists() {
        let content = fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&content)?)
    } else {
        let config = AppConfig::default();
        save_config(&config)?;
        Ok(config)
    }
}

pub fn save_config(config: &AppConfig) -> Result<()> {
    let path = config_path()?;
    let content = serde_json::to_string_pretty(config)?;
    fs::write(path, content)?;
    Ok(())
}
