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
