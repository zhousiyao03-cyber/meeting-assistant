use anyhow::{anyhow, Result};
use serde_json::json;

use super::{ChatOptions, LlmMessage, LlmProvider};

pub struct AnthropicProvider {
    client: reqwest::Client,
    api_key: String,
    model: String,
}

impl AnthropicProvider {
    pub fn new(api_key: &str, model: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: api_key.to_string(),
            model: model.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl LlmProvider for AnthropicProvider {
    async fn chat(&self, messages: &[LlmMessage], opts: &ChatOptions) -> Result<String> {
        let system_msg = messages
            .iter()
            .find(|m| m.role == "system")
            .map(|m| m.content.clone());
        let user_assistant: Vec<_> = messages
            .iter()
            .filter(|m| m.role != "system")
            .map(|m| json!({ "role": m.role, "content": m.content }))
            .collect();

        let system_field = match (system_msg.as_deref(), opts.enable_caching) {
            (Some(s), true) => json!([{
                "type": "text",
                "text": s,
                "cache_control": { "type": "ephemeral" }
            }]),
            (Some(s), false) => json!(s),
            (None, _) => json!(""),
        };

        let body = json!({
            "model": self.model,
            "max_tokens": opts.max_tokens,
            "temperature": opts.temperature,
            "system": system_field,
            "messages": user_assistant,
        });

        let resp = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Anthropic API error ({}): {}", status, body));
        }

        let json: serde_json::Value = resp.json().await?;
        let text = json["content"][0]["text"].as_str().unwrap_or("").to_string();
        Ok(text)
    }

    fn provider_name(&self) -> &'static str {
        "anthropic"
    }
}
