use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const KEYRING_SERVICE: &str = "app.voicenote.confide";
const KEYRING_USER: &str = "license_key";

pub fn get_license_key() -> Result<Option<String>> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)?;
    match entry.get_password() {
        Ok(s) => Ok(Some(s)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

pub fn set_license_key(key: &str) -> Result<()> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)?;
    entry.set_password(key)?;
    Ok(())
}

pub fn clear_license_key() -> Result<()> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)?;
    let _ = entry.delete_credential();
    Ok(())
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct CachedPlan {
    pub plan: super::UserPlan,
    pub cached_at: i64,
    pub pending_usage: Vec<super::metering::UsageEvent>,
}

fn cache_path() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("No home dir"))?;
    let dir = home.join(".meeting-assistant");
    fs::create_dir_all(&dir)?;
    Ok(dir.join("license-cache.json"))
}

pub fn load_cached() -> Result<Option<CachedPlan>> {
    let path = cache_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let s = fs::read_to_string(&path)?;
    Ok(Some(serde_json::from_str(&s)?))
}

pub fn save_cached(c: &CachedPlan) -> Result<()> {
    let path = cache_path()?;
    fs::write(path, serde_json::to_string_pretty(c)?)?;
    Ok(())
}
