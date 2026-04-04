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

#[derive(Clone, Debug, Serialize)]
pub struct MeetingMinutes {
    pub title: String,
    pub key_points: Vec<String>,
    pub action_items: Vec<String>,
    pub decisions: Vec<String>,
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
    async fn chat(&self, messages: &[LlmMessage], max_tokens: u32) -> Result<String> {
        let url = format!("{}/chat/completions", self.base_url);

        let body = serde_json::json!({
            "model": self.model,
            "messages": messages,
            "temperature": 0.7,
            "max_tokens": max_tokens,
        });

        let mut req = self.client.post(&url).json(&body);
        if !self.api_key.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", self.api_key));
        }

        let resp = req.send().await?;
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "LLM API error ({}): {}",
                status,
                if body.len() > 200 { &body[..200] } else { &body }
            ));
        }
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

        // Truncate to last ~8000 chars to stay within LLM token limits
        let truncated = if transcript.len() > 8000 {
            let start = transcript.len() - 8000;
            let break_at = transcript[start..].find(|c: char| c.is_whitespace())
                .map(|i| start + i)
                .unwrap_or(start);
            &transcript[break_at..]
        } else {
            transcript
        };

        let messages = vec![
            LlmMessage { role: "system".into(), content: system },
            LlmMessage { role: "user".into(), content: format!("会议转录：\n{}", truncated) },
        ];

        let response = self.chat(&messages, 500).await?;
        Ok(parse_summary(&response))
    }

    /// Generate structured meeting minutes from full transcript and summary.
    pub async fn generate_minutes(&self, transcript: &str, summary: &str) -> Result<MeetingMinutes> {
        let system = "你是会议纪要专家。根据会议转录和实时摘要，生成结构化会议纪要。\n\n\
            严格按以下格式输出，每项一行：\n\
            标题：（10字以内的会议主题）\n\
            要点：\n- 要点1\n- 要点2\n\n\
            行动项：\n- [负责人] 具体任务\n\n\
            决策：\n- 决策1";

        // Truncate transcript to last 8000 chars
        let truncated = if transcript.len() > 8000 {
            let start = transcript.len() - 8000;
            let break_at = transcript[start..].find(|c: char| c.is_whitespace())
                .map(|i| start + i).unwrap_or(start);
            &transcript[break_at..]
        } else {
            transcript
        };

        let user_msg = format!(
            "会议转录：\n{}\n\n实时摘要：\n{}\n\n请生成会议纪要。",
            truncated, summary
        );

        let messages = vec![
            LlmMessage { role: "system".into(), content: system.into() },
            LlmMessage { role: "user".into(), content: user_msg },
        ];

        let response = self.chat(&messages, 800).await?;
        Ok(parse_minutes(&response))
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
            system.push_str(&format!("\n\n参考文档：\n{}", reference_docs));
        }

        let user_msg = format!(
            "最近的对话内容：\n{}\n\n\
             触发原因：{}\n\n\
             请严格按以下格式输出，每项一行，不要多余文字：\n\
             建议：（一句你可以直接说出口的话，不超过30字，必须引用对话中的具体内容）\n\
             角度：（2-4个字的发言角度标签）",
            transcript, trigger_reason
        );

        let messages = vec![
            LlmMessage { role: "system".into(), content: system },
            LlmMessage { role: "user".into(), content: user_msg },
        ];

        let response = self.chat(&messages, 150).await?;
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
    let mut suggestion = String::new();
    let mut angle = String::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(val) = extract_field(trimmed, "建议") {
            suggestion = val;
        } else if let Some(val) = extract_field(trimmed, "角度") {
            angle = val;
        }
    }

    // Fallback: if structured parsing failed, use first non-empty line as suggestion
    if suggestion.is_empty() {
        suggestion = text
            .lines()
            .map(|l| l.trim())
            .find(|l| !l.is_empty())
            .unwrap_or("")
            .to_string();
        // Truncate overly long fallback
        if suggestion.chars().count() > 60 {
            suggestion = suggestion.chars().take(60).collect::<String>() + "...";
        }
    }

    // Strip surrounding quotes from suggestion
    suggestion = suggestion
        .trim_start_matches(['\"', '"', '「'])
        .trim_end_matches(['\"', '"', '」'])
        .to_string();

    SpeakingAdvice {
        reason: trigger_reason.to_string(),
        suggestion,
        angle,
        timestamp: offset_secs,
    }
}

fn parse_minutes(text: &str) -> MeetingMinutes {
    let mut title = String::new();
    let mut key_points = Vec::new();
    let mut action_items = Vec::new();
    let mut decisions = Vec::new();
    let mut current_section = "";

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() { continue; }

        if let Some(val) = extract_field(trimmed, "标题") {
            title = val;
            current_section = "";
        } else if trimmed.starts_with("要点") {
            current_section = "points";
        } else if trimmed.starts_with("行动项") || trimmed.starts_with("待办") {
            current_section = "actions";
        } else if trimmed.starts_with("决策") {
            current_section = "decisions";
        } else if trimmed.starts_with("- ") || trimmed.starts_with("• ") {
            let item = trimmed.trim_start_matches("- ").trim_start_matches("• ").to_string();
            match current_section {
                "points" => key_points.push(item),
                "actions" => action_items.push(item),
                "decisions" => decisions.push(item),
                _ => key_points.push(item),
            }
        }
    }

    if title.is_empty() {
        title = "会议纪要".into();
    }

    MeetingMinutes { title, key_points, action_items, decisions }
}

/// Extract the value after a "key：value" or "key: value" pattern.
fn extract_field(line: &str, key: &str) -> Option<String> {
    if line.starts_with(key) {
        line.split_once(['：', ':'])
            .map(|(_, v)| v.trim().to_string())
            .filter(|v| !v.is_empty())
    } else {
        None
    }
}
