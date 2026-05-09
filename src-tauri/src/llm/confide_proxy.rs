//! Alpha-period adapter: routes to llmgate.io with the bundled token.
//! llmgate is OpenAI-protocol-compatible, so this delegates to OpenAiProvider
//! pointed at the internal gateway.

use anyhow::Result;

use super::openai::OpenAiProvider;
use super::{ChatOptions, LlmMessage, LlmProvider};

pub struct ConfideProxyProvider {
    inner: OpenAiProvider,
}

impl ConfideProxyProvider {
    pub fn new(base_url: &str, token: &str, model: &str) -> Self {
        Self {
            inner: OpenAiProvider::new(token, model, base_url),
        }
    }
}

#[async_trait::async_trait]
impl LlmProvider for ConfideProxyProvider {
    async fn chat(&self, messages: &[LlmMessage], opts: &ChatOptions) -> Result<String> {
        self.inner.chat(messages, opts).await
    }
    fn provider_name(&self) -> &'static str {
        "confide-proxy-llmgate"
    }
}
