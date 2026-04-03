use anyhow::Result;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MeetingRecord {
    pub id: String,
    pub template_name: String,
    pub started_at: String,
    pub duration_secs: u64,
    pub transcript: String,
    pub summary: String,
    pub advices_json: String,
}

pub fn db_path() -> PathBuf {
    let home = dirs::home_dir().expect("No home dir");
    let dir = home.join(".meeting-assistant");
    let _ = fs::create_dir_all(&dir);
    dir.join("history.db")
}

pub fn init_db() -> Result<Connection> {
    let path = db_path();
    let conn = Connection::open(&path)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS meetings (
            id TEXT PRIMARY KEY,
            template_name TEXT NOT NULL,
            started_at TEXT NOT NULL,
            duration_secs INTEGER NOT NULL,
            transcript TEXT NOT NULL,
            summary TEXT NOT NULL,
            advices_json TEXT NOT NULL
        );"
    )?;
    Ok(conn)
}

pub fn save_meeting(record: &MeetingRecord) -> Result<()> {
    let conn = init_db()?;
    conn.execute(
        "INSERT OR REPLACE INTO meetings (id, template_name, started_at, duration_secs, transcript, summary, advices_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![
            record.id,
            record.template_name,
            record.started_at,
            record.duration_secs,
            record.transcript,
            record.summary,
            record.advices_json,
        ],
    )?;
    Ok(())
}

pub fn list_meetings() -> Result<Vec<MeetingRecord>> {
    let conn = init_db()?;
    let mut stmt = conn.prepare(
        "SELECT id, template_name, started_at, duration_secs, transcript, summary, advices_json FROM meetings ORDER BY started_at DESC"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(MeetingRecord {
            id: row.get(0)?,
            template_name: row.get(1)?,
            started_at: row.get(2)?,
            duration_secs: row.get(3)?,
            transcript: row.get(4)?,
            summary: row.get(5)?,
            advices_json: row.get(6)?,
        })
    })?;
    let mut records = Vec::new();
    for row in rows {
        records.push(row?);
    }
    Ok(records)
}
