use anyhow::Result;
use serde::{Deserialize, Serialize};

pub mod anthropic;
pub mod openai;
pub mod confide_proxy;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LlmMessage {
    pub role: String,
    pub content: String,
}

#[derive(Clone, Debug)]
pub struct ChatOptions {
    pub max_tokens: u32,
    pub temperature: f32,
    pub enable_caching: bool,
}

impl Default for ChatOptions {
    fn default() -> Self {
        Self {
            max_tokens: 500,
            temperature: 0.7,
            enable_caching: true,
        }
    }
}

#[async_trait::async_trait]
pub trait LlmProvider: Send + Sync {
    async fn chat(&self, messages: &[LlmMessage], opts: &ChatOptions) -> Result<String>;
    fn provider_name(&self) -> &'static str;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LlmMode {
    /// Alpha period: route through llmgate (Bytedance internal gateway)
    ConfideProxy {
        base_url: String,
        token: String,
        model: String,
    },
    /// Production: direct Anthropic
    Anthropic { api_key: String, model: String },
    /// BYO: user's OpenAI key
    UserOpenAi {
        api_key: String,
        model: String,
        base_url: String,
    },
    /// BYO: user's Anthropic key
    UserAnthropic { api_key: String, model: String },
}

pub fn create_provider(mode: &LlmMode) -> Box<dyn LlmProvider> {
    match mode {
        LlmMode::ConfideProxy { base_url, token, model } => {
            Box::new(confide_proxy::ConfideProxyProvider::new(base_url, token, model))
        }
        LlmMode::Anthropic { api_key, model }
        | LlmMode::UserAnthropic { api_key, model } => {
            Box::new(anthropic::AnthropicProvider::new(api_key, model))
        }
        LlmMode::UserOpenAi {
            api_key,
            model,
            base_url,
        } => Box::new(openai::OpenAiProvider::new(api_key, model, base_url)),
    }
}
