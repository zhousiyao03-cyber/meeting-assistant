use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeetingTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub system_prompt: String,
    pub trigger_hints: Vec<String>,
    pub advice_style: String,
    pub enabled: bool,
    #[serde(default)]
    pub language: String,
    #[serde(default)]
    pub role_persona: String,
    #[serde(default)]
    pub mimic_style: String,
    #[serde(default)]
    pub expertise_context: String,
    #[serde(default)]
    pub stealth_default: bool,
    #[serde(default = "default_cooldown")]
    pub advice_cooldown_seconds: u32,
    #[serde(default)]
    pub trigger_config: TriggerConfig,
}

fn default_cooldown() -> u32 {
    12
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TriggerConfig {
    #[serde(default = "default_true")]
    pub on_ask_opinion: bool,
    #[serde(default)]
    pub on_question_to_user: bool,
    #[serde(default = "default_true")]
    pub on_domain_topic: bool,
    #[serde(default = "default_true")]
    pub on_decision_point: bool,
    #[serde(default = "default_true")]
    pub on_discussion_stuck: bool,
    #[serde(default)]
    pub custom_keywords: Vec<String>,
    #[serde(default)]
    pub domain_keywords: Vec<String>,
}

fn default_true() -> bool {
    true
}

impl Default for TriggerConfig {
    fn default() -> Self {
        Self {
            on_ask_opinion: true,
            on_question_to_user: false,
            on_domain_topic: true,
            on_decision_point: true,
            on_discussion_stuck: true,
            custom_keywords: vec![],
            domain_keywords: vec![],
        }
    }
}

/// User templates dir, scoped by locale.
fn templates_dir(locale: &str) -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("No home dir"))?;
    let dir = home
        .join(".meeting-assistant")
        .join("templates")
        .join(locale);
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

pub fn list_templates_for_locale(locale: &str) -> Result<Vec<MeetingTemplate>> {
    let dir = templates_dir(locale)?;
    let mut templates = Vec::new();
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map(|e| e == "json").unwrap_or(false) {
            let content = fs::read_to_string(&path)?;
            let template: MeetingTemplate = serde_json::from_str(&content)?;
            templates.push(template);
        }
    }
    Ok(templates)
}

/// Backward-compat: defaults to en-US if no locale specified.
pub fn list_templates() -> Result<Vec<MeetingTemplate>> {
    let mut all = list_templates_for_locale("en-US").unwrap_or_default();
    let zh = list_templates_for_locale("zh-CN").unwrap_or_default();
    all.extend(zh);
    Ok(all)
}

pub fn save_template(template: &MeetingTemplate) -> Result<()> {
    let locale = if template.language.is_empty() {
        "en-US"
    } else {
        &template.language
    };
    let dir = templates_dir(locale)?;
    let path = dir.join(format!("{}.json", template.id));
    let content = serde_json::to_string_pretty(template)?;
    fs::write(path, content)?;
    Ok(())
}

pub fn delete_template(id: &str) -> Result<()> {
    for locale in &["zh-CN", "en-US"] {
        let dir = templates_dir(locale)?;
        let path = dir.join(format!("{}.json", id));
        if path.exists() {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

/// Copy bundled default templates to user dir if none exist (per locale).
pub fn ensure_default_templates(bundled_dir: &std::path::Path) -> Result<()> {
    for locale in &["zh-CN", "en-US"] {
        let user_dir = templates_dir(locale)?;
        let existing: Vec<_> = fs::read_dir(&user_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|x| x == "json").unwrap_or(false))
            .collect();

        if existing.is_empty() {
            let bundled_locale = bundled_dir.join(locale);
            if bundled_locale.exists() {
                for entry in fs::read_dir(&bundled_locale)? {
                    let entry = entry?;
                    let dest = user_dir.join(entry.file_name());
                    fs::copy(entry.path(), dest)?;
                }
            }
        }
    }
    Ok(())
}
