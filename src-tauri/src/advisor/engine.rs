use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::templates::MeetingTemplate;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LlmMessage {
    pub role: String,
    pub content: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct SpeakingAdvice {
    pub reason: String,
    pub suggestion: String,
    pub angle: String,
    pub timestamp: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct MeetingSummary {
    pub points: Vec<String>,
    pub current_topic: String,
}

pub struct AdvisorEngine {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
    model: String,
}

impl AdvisorEngine {
    pub fn new(base_url: &str, api_key: &str, model: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            model: model.to_string(),
        }
    }

    /// Call the LLM with the given messages. Returns the assistant's response text.
    async fn chat(&self, messages: &[LlmMessage]) -> Result<String> {
        let url = format!("{}/chat/completions", self.base_url);

        let body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "temperature": 0.7,
            "max_tokens": 500,
        });

        let mut req = self.client.post(&url).json(&body);
        if !self.api_key.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", self.api_key));
        }

        let resp = req.send().await?;
        let json: serde_json::Value = resp.json().await?;
        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();
        Ok(content)
    }

    /// Generate a meeting summary from recent transcript.
    pub async fn generate_summary(
        &self,
        transcript: &str,
        reference_docs: &str,
    ) -> Result<MeetingSummary> {
        let mut system = String::from(
            "你是一个会议记录助手。请根据以下会议转录内容，提取关键要点并总结当前正在讨论的话题。\n\
             输出格式：\n\
             要点：\n- 要点1\n- 要点2\n\n\
             当前讨论：一句话描述当前焦点话题"
        );

        if !reference_docs.is_empty() {
            system.push_str(&format!("\n\n参考文档：\n{}", reference_docs));
        }

        let messages = vec![
            LlmMessage { role: "system".into(), content: system },
            LlmMessage { role: "user".into(), content: format!("会议转录：\n{}", transcript) },
        ];

        let response = self.chat(&messages).await?;
        Ok(parse_summary(&response))
    }

    /// Generate speaking advice based on transcript, template, and trigger reason.
    pub async fn generate_advice(
        &self,
        template: &MeetingTemplate,
        transcript: &str,
        trigger_reason: &str,
        reference_docs: &str,
        offset_secs: f64,
    ) -> Result<SpeakingAdvice> {
        let mut system = template.system_prompt.clone();
        if !reference_docs.is_empty() {
            system.push_str(&format!("\n\n参考文档（用于提供背景上下文）：\n{}", reference_docs));
        }

        let user_msg = format!(
            "会议转录：\n{}\n\n触发原因：{}\n\n请给出发言建议。",
            transcript, trigger_reason
        );

        let messages = vec![
            LlmMessage { role: "system".into(), content: system },
            LlmMessage { role: "user".into(), content: user_msg },
        ];

        let response = self.chat(&messages).await?;
        Ok(parse_advice(&response, trigger_reason, offset_secs))
    }
}

fn parse_summary(text: &str) -> MeetingSummary {
    let mut points = Vec::new();
    let mut current_topic = String::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("- ") || trimmed.starts_with("• ") {
            points.push(trimmed.trim_start_matches("- ").trim_start_matches("• ").to_string());
        } else if trimmed.starts_with("当前讨论") || trimmed.starts_with("当前话题") {
            current_topic = trimmed
                .split_once(['：', ':'])
                .map(|(_, v)| v.trim().to_string())
                .unwrap_or_default();
        }
    }

    if points.is_empty() {
        points.push(text.trim().to_string());
    }

    MeetingSummary {
        points,
        current_topic,
    }
}

fn parse_advice(text: &str, trigger_reason: &str, offset_secs: f64) -> SpeakingAdvice {
    let lines: Vec<&str> = text.lines().collect();
    let suggestion = text.trim().to_string();
    let angle = lines
        .iter()
        .find(|l| l.contains("角度") || l.contains("视角"))
        .map(|l| {
            l.split_once(['：', ':'])
                .map(|(_, v)| v.trim().to_string())
                .unwrap_or_default()
        })
        .unwrap_or_default();

    SpeakingAdvice {
        reason: trigger_reason.to_string(),
        suggestion,
        angle,
        timestamp: offset_secs,
    }
}
