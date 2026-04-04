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
    // New configurable fields
    #[serde(default)]
    pub role_persona: String,       // e.g. "前端技术专家，目标成为小组长"
    #[serde(default)]
    pub mimic_style: String,        // e.g. "像张一鸣一样简洁直接" or custom prompt
    #[serde(default)]
    pub expertise_context: String,  // professional background/knowledge to inject
    #[serde(default)]
    pub trigger_config: TriggerConfig, // configurable trigger settings
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TriggerConfig {
    #[serde(default = "default_true")]
    pub on_ask_opinion: bool,       // trigger when someone asks group opinion
    #[serde(default = "default_true")]
    pub on_domain_topic: bool,      // trigger when discussion touches your domain
    #[serde(default = "default_true")]
    pub on_decision_point: bool,    // trigger on disagreement/decision needed
    #[serde(default = "default_true")]
    pub on_discussion_stuck: bool,  // trigger when discussion is stuck
    #[serde(default)]
    pub custom_keywords: Vec<String>, // additional trigger keywords
    #[serde(default)]
    pub domain_keywords: Vec<String>, // domain-specific keywords
}

fn default_true() -> bool { true }

impl Default for TriggerConfig {
    fn default() -> Self {
        Self {
            on_ask_opinion: true,
            on_domain_topic: true,
            on_decision_point: true,
            on_discussion_stuck: true,
            custom_keywords: vec![],
            domain_keywords: vec![
                "前端".into(), "页面".into(), "组件".into(), "CSS".into(),
                "渲染".into(), "性能优化".into(), "React".into(), "TypeScript".into(),
            ],
        }
    }
}

/// Returns ~/.meeting-assistant/templates/
fn templates_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("No home dir"))?;
    let dir = home.join(".meeting-assistant").join("templates");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Load all templates from the templates directory.
pub fn list_templates() -> Result<Vec<MeetingTemplate>> {
    let dir = templates_dir()?;
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

/// Save a template to disk.
pub fn save_template(template: &MeetingTemplate) -> Result<()> {
    let dir = templates_dir()?;
    let path = dir.join(format!("{}.json", template.id));
    let content = serde_json::to_string_pretty(template)?;
    fs::write(path, content)?;
    Ok(())
}

/// Delete a template by ID.
pub fn delete_template(id: &str) -> Result<()> {
    let dir = templates_dir()?;
    let path = dir.join(format!("{}.json", id));
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

/// Copy bundled default templates to user dir if none exist.
pub fn ensure_default_templates(bundled_dir: &std::path::Path) -> Result<()> {
    let user_dir = templates_dir()?;
    let existing: Vec<_> = fs::read_dir(&user_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "json").unwrap_or(false))
        .collect();

    if existing.is_empty() {
        // Copy bundled templates
        if bundled_dir.exists() {
            for entry in fs::read_dir(bundled_dir)? {
                let entry = entry?;
                let dest = user_dir.join(entry.file_name());
                fs::copy(entry.path(), dest)?;
            }
        }
    }
    Ok(())
}
