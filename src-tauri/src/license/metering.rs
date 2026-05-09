use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

const API_BASE: &str = "https://api.confide.knosi.xyz";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UsageEvent {
    pub event_id: String,
    pub meeting_id: String,
    pub provider: String,
    pub seconds_used: f64,
    pub started_at: i64,
    pub ended_at: i64,
}

pub struct Meter {
    pub meeting_id: String,
    pub provider: String,
    pub started_at: SystemTime,
    pub last_sync_at: SystemTime,
    pub accumulated_seconds: f64,
}

impl Meter {
    pub fn new(meeting_id: String, provider: String) -> Self {
        let now = SystemTime::now();
        Self {
            meeting_id,
            provider,
            started_at: now,
            last_sync_at: now,
            accumulated_seconds: 0.0,
        }
    }

    pub fn maybe_create_event(&mut self) -> Option<UsageEvent> {
        let total_elapsed = self.started_at.elapsed().ok()?.as_secs_f64();
        let unsynced = total_elapsed - self.accumulated_seconds;
        if unsynced < 300.0 {
            return None;
        }
        self.create_event_now(unsynced)
    }

    pub fn create_final_event(&mut self) -> Option<UsageEvent> {
        let total_elapsed = self.started_at.elapsed().ok()?.as_secs_f64();
        let unsynced = total_elapsed - self.accumulated_seconds;
        if unsynced < 1.0 {
            return None;
        }
        self.create_event_now(unsynced)
    }

    fn create_event_now(&mut self, unsynced: f64) -> Option<UsageEvent> {
        let now = SystemTime::now();
        let evt = UsageEvent {
            event_id: format!("{}-{}", self.meeting_id, uuid::Uuid::new_v4()),
            meeting_id: self.meeting_id.clone(),
            provider: self.provider.clone(),
            seconds_used: unsynced,
            started_at: self
                .last_sync_at
                .duration_since(SystemTime::UNIX_EPOCH)
                .ok()?
                .as_secs() as i64,
            ended_at: now
                .duration_since(SystemTime::UNIX_EPOCH)
                .ok()?
                .as_secs() as i64,
        };
        self.accumulated_seconds += unsynced;
        self.last_sync_at = now;
        Some(evt)
    }
}

pub async fn sync_usage(key: &str, events: Vec<UsageEvent>) -> Result<()> {
    if events.is_empty() {
        return Ok(());
    }
    let url = format!("{}/usage", API_BASE);
    let body = serde_json::json!({ "key": key, "events": events });
    let client = reqwest::Client::new();
    let resp = client.post(&url).json(&body).send().await?;
    if !resp.status().is_success() {
        return Err(anyhow::anyhow!("Usage sync failed: {}", resp.status()));
    }
    Ok(())
}
