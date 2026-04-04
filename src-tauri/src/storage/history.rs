use anyhow::Result;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeetingRecord {
    pub id: String,
    pub title: String,
    pub template_name: String,
    pub started_at: String,
    pub duration_secs: u64,
    pub transcript: String,
    pub summary: String,
    pub action_items: String,
    pub advices_json: String,
}

pub fn db_path() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("No home dir"))?;
    let dir = home.join(".meeting-assistant");
    fs::create_dir_all(&dir)?;
    Ok(dir.join("history.db"))
}

pub fn init_db() -> Result<Connection> {
    let path = db_path()?;
    let conn = Connection::open(&path)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS meetings (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL DEFAULT '',
            template_name TEXT NOT NULL,
            started_at TEXT NOT NULL,
            duration_secs INTEGER NOT NULL,
            transcript TEXT NOT NULL,
            summary TEXT NOT NULL,
            action_items TEXT NOT NULL DEFAULT '',
            advices_json TEXT NOT NULL
        );"
    )?;
    Ok(conn)
}

pub fn save_meeting(record: &MeetingRecord) -> Result<()> {
    let conn = init_db()?;
    conn.execute(
        "INSERT OR REPLACE INTO meetings (id, title, template_name, started_at, duration_secs, transcript, summary, action_items, advices_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            record.id,
            record.title,
            record.template_name,
            record.started_at,
            record.duration_secs,
            record.transcript,
            record.summary,
            record.action_items,
            record.advices_json,
        ],
    )?;
    Ok(())
}

pub fn list_meetings() -> Result<Vec<MeetingRecord>> {
    let conn = init_db()?;
    let mut stmt = conn.prepare(
        "SELECT id, title, template_name, started_at, duration_secs, transcript, summary, action_items, advices_json FROM meetings ORDER BY started_at DESC"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(MeetingRecord {
            id: row.get(0)?,
            title: row.get(1)?,
            template_name: row.get(2)?,
            started_at: row.get(3)?,
            duration_secs: row.get(4)?,
            transcript: row.get(5)?,
            summary: row.get(6)?,
            action_items: row.get(7)?,
            advices_json: row.get(8)?,
        })
    })?;
    let mut records = Vec::new();
    for row in rows {
        records.push(row?);
    }
    Ok(records)
}

pub fn delete_meeting(id: &str) -> Result<()> {
    let conn = init_db()?;
    conn.execute("DELETE FROM meetings WHERE id = ?1", rusqlite::params![id])?;
    Ok(())
}
