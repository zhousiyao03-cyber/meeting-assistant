use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::templates::MeetingTemplate;
use crate::llm::{create_provider, ChatOptions, LlmMessage as LlmMsg, LlmMode, LlmProvider};

/// Backward-compat type alias used by existing call sites and tests.
pub type LlmMessage = LlmMsg;

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
    provider: Box<dyn LlmProvider>,
}

impl AdvisorEngine {
    /// Build provider from old AppConfig.llm — heuristic mapping for transitional period.
    /// Week 5 codepath replaces this with explicit LlmMode passed by caller.
    pub fn new(base_url: &str, api_key: &str, model: &str) -> Self {
        let mode = if base_url.contains("anthropic.com") {
            LlmMode::Anthropic {
                api_key: api_key.into(),
                model: model.into(),
            }
        } else if base_url.contains("llmgate") {
            LlmMode::ConfideProxy {
                base_url: base_url.into(),
                token: api_key.into(),
                model: model.into(),
            }
        } else {
            LlmMode::UserOpenAi {
                api_key: api_key.into(),
                model: model.into(),
                base_url: base_url.into(),
            }
        };
        Self {
            provider: create_provider(&mode),
        }
    }

    pub fn from_mode(mode: &LlmMode) -> Self {
        Self {
            provider: create_provider(mode),
        }
    }

    async fn chat(&self, messages: &[LlmMsg], max_tokens: u32) -> Result<String> {
        let opts = ChatOptions {
            max_tokens,
            temperature: 0.7,
            enable_caching: true,
        };
        self.provider.chat(messages, &opts).await
    }

    pub async fn generate_summary(
        &self,
        transcript: &str,
        reference_docs: &str,
    ) -> Result<MeetingSummary> {
        let mut system = String::from(
            "你是一个会议记录助手。请根据以下会议转录内容，提取关键要点并总结当前正在讨论的话题。\n\
             输出格式：\n\
             要点：\n- 要点1\n- 要点2\n\n\
             当前讨论：一句话描述当前焦点话题",
        );

        if !reference_docs.is_empty() {
            system.push_str(&format!("\n\n参考文档：\n{}", reference_docs));
        }

        let truncated = truncate_tail(transcript, 8000);

        let messages = vec![
            LlmMsg {
                role: "system".into(),
                content: system,
            },
            LlmMsg {
                role: "user".into(),
                content: format!("会议转录：\n{}", truncated),
            },
        ];

        let response = self.chat(&messages, 500).await?;
        Ok(parse_summary(&response))
    }

    pub async fn generate_minutes(
        &self,
        transcript: &str,
        summary: &str,
    ) -> Result<MeetingMinutes> {
        let system = "你是会议纪要专家。根据会议转录和实时摘要，生成结构化会议纪要。\n\n\
            严格按以下格式输出，每项一行：\n\
            标题：（10字以内的会议主题）\n\
            要点：\n- 要点1\n- 要点2\n\n\
            行动项：\n- [负责人] 具体任务\n\n\
            决策：\n- 决策1";

        let truncated = truncate_tail(transcript, 8000);
        let user_msg = format!(
            "会议转录：\n{}\n\n实时摘要：\n{}\n\n请生成会议纪要。",
            truncated, summary
        );

        let messages = vec![
            LlmMsg {
                role: "system".into(),
                content: system.into(),
            },
            LlmMsg {
                role: "user".into(),
                content: user_msg,
            },
        ];

        let response = self.chat(&messages, 800).await?;
        Ok(parse_minutes(&response))
    }

    /// Generate speaking advice. context_note is the per-meeting note from the user
    /// (≤500 chars), reference_docs is the loaded resume/agenda text.
    pub async fn generate_advice(
        &self,
        template: &MeetingTemplate,
        transcript: &str,
        trigger_reason: &str,
        reference_docs: &str,
        context_note: &str,
        offset_secs: f64,
    ) -> Result<SpeakingAdvice> {
        let mut system = String::new();

        if !template.role_persona.is_empty() {
            system.push_str(&format!(
                "用户角色：{}。\n\n",
                template.role_persona
            ));
        }
        if !template.mimic_style.is_empty() {
            system.push_str(&format!("发言风格：{}。\n\n", template.mimic_style));
        }
        if !template.expertise_context.is_empty() {
            system.push_str(&format!("专业背景：\n{}\n\n", template.expertise_context));
        }

        if system.is_empty() {
            system = template.system_prompt.clone();
        } else {
            system.push_str(&template.system_prompt);
        }

        if !context_note.is_empty() {
            system.push_str(&format!(
                "\n\n本场会议上下文（用户备注）：\n{}",
                context_note
            ));
        }
        if !reference_docs.is_empty() {
            system.push_str(&format!("\n\n参考文档：\n{}", reference_docs));
        }

        let user_msg = format!(
            "最近的对话内容：\n{}\n\n触发原因：{}\n\n请按格式输出建议和角度。",
            transcript, trigger_reason
        );

        let messages = vec![
            LlmMsg {
                role: "system".into(),
                content: system,
            },
            LlmMsg {
                role: "user".into(),
                content: user_msg,
            },
        ];

        let response = self.chat(&messages, 150).await?;
        Ok(parse_advice(&response, trigger_reason, offset_secs))
    }
}

fn truncate_tail(s: &str, max: usize) -> &str {
    if s.len() <= max {
        return s;
    }
    let start = s.len() - max;
    let break_at = s[start..]
        .find(|c: char| c.is_whitespace())
        .map(|i| start + i)
        .unwrap_or(start);
    &s[break_at..]
}

fn parse_summary(text: &str) -> MeetingSummary {
    let mut points = Vec::new();
    let mut current_topic = String::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("- ") || trimmed.starts_with("• ") {
            points.push(
                trimmed
                    .trim_start_matches("- ")
                    .trim_start_matches("• ")
                    .to_string(),
            );
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
        if let Some(val) = extract_field(trimmed, "建议").or_else(|| extract_field(trimmed, "Advice")) {
            suggestion = val;
        } else if let Some(val) = extract_field(trimmed, "角度").or_else(|| extract_field(trimmed, "Angle")) {
            angle = val;
        }
    }

    if suggestion.is_empty() {
        suggestion = text
            .lines()
            .map(|l| l.trim())
            .find(|l| !l.is_empty())
            .unwrap_or("")
            .to_string();
        if suggestion.chars().count() > 60 {
            suggestion = suggestion.chars().take(60).collect::<String>() + "...";
        }
    }

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
        if trimmed.is_empty() {
            continue;
        }

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
            let item = trimmed
                .trim_start_matches("- ")
                .trim_start_matches("• ")
                .to_string();
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

    MeetingMinutes {
        title,
        key_points,
        action_items,
        decisions,
    }
}

fn extract_field(line: &str, key: &str) -> Option<String> {
    if line.starts_with(key) {
        line.split_once(['：', ':'])
            .map(|(_, v)| v.trim().to_string())
            .filter(|v| !v.is_empty())
    } else {
        None
    }
}
