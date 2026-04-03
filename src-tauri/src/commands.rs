use serde::Serialize;
use tauri::command;
use tauri::Emitter;

use crate::audio::buffer::{create_shared_buffer, SharedBuffer};
use crate::audio::capture;
use crate::advisor::templates::{self, MeetingTemplate};
use crate::documents::loader::{self, LoadedDocument};
use crate::storage::config::{self, AppConfig};
use crate::storage::history::{self, MeetingRecord};
use crate::transcript::store::{create_shared_store, SharedTranscriptStore, TranscriptSegment};
use crate::whisper::downloader;
use crate::whisper::engine::WhisperEngine;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

// --- Audio ---

#[derive(Serialize)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
}

#[command]
pub fn list_audio_devices() -> Result<Vec<AudioDevice>, String> {
    capture::list_input_devices()
        .map(|devices| {
            devices
                .into_iter()
                .map(|(id, name)| AudioDevice { id, name })
                .collect()
        })
        .map_err(|e| e.to_string())
}

#[command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}! Meeting Assistant is running.", name)
}

// --- Whisper ---

#[derive(Serialize)]
pub struct ModelStatus {
    pub downloaded: bool,
    pub path: Option<String>,
}

#[command]
pub fn check_whisper_model() -> Result<ModelStatus, String> {
    let path = downloader::model_path().map_err(|e| e.to_string())?;
    Ok(ModelStatus {
        downloaded: path.is_some(),
        path: path.map(|p| p.to_string_lossy().to_string()),
    })
}

#[command]
pub async fn download_whisper_model(window: tauri::Window) -> Result<String, String> {
    let path = downloader::download_model(move |downloaded, total| {
        let _ = window.emit("model-download-progress", serde_json::json!({
            "downloaded": downloaded,
            "total": total,
        }));
    })
    .await
    .map_err(|e| e.to_string())?;

    Ok(path.to_string_lossy().to_string())
}

// --- Recording Pipeline ---

pub struct RecordingState {
    pub is_recording: bool,
    pub mic_buffer: SharedBuffer,
    pub capture_buffer: SharedBuffer,
    pub transcript: SharedTranscriptStore,
    pub start_time: Option<std::time::Instant>,
}

impl RecordingState {
    pub fn new() -> Self {
        Self {
            is_recording: false,
            mic_buffer: create_shared_buffer(3, 16000),
            capture_buffer: create_shared_buffer(3, 16000),
            transcript: create_shared_store(),
            start_time: None,
        }
    }
}

pub type SharedRecordingState = Arc<TokioMutex<RecordingState>>;

#[command]
pub async fn start_recording(
    mic_device: String,
    capture_device: String,
    state: tauri::State<'_, SharedRecordingState>,
    window: tauri::Window,
) -> Result<(), String> {
    let mut rec = state.lock().await;
    if rec.is_recording {
        return Err("Already recording".into());
    }

    rec.is_recording = true;
    rec.start_time = Some(std::time::Instant::now());
    let mic_buffer = rec.mic_buffer.clone();
    let capture_buffer = rec.capture_buffer.clone();
    let transcript = rec.transcript.clone();

    // Separate streams into separate buffers
    let _mic_stream = capture::start_capture(&mic_device, mic_buffer.clone())
        .map_err(|e| e.to_string())?;
    let _capture_stream = capture::start_capture(&capture_device, capture_buffer.clone())
        .map_err(|e| e.to_string())?;

    // Spawn Whisper processing — process mic and capture separately, deduplicate
    let start_time = rec.start_time.unwrap();
    let win = window.clone();
    let transcript_for_whisper = transcript.clone();
    tokio::spawn(async move {
        let model_path = match crate::whisper::downloader::model_path() {
            Ok(Some(p)) => p,
            _ => {
                eprintln!("[whisper] Model not found");
                return;
            }
        };
        let engine = match WhisperEngine::new(&model_path) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("[whisper] Failed to load: {}", e);
                return;
            }
        };
        eprintln!("[whisper] Model loaded, starting transcription loop");

        let mut last_text = String::new();

        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            // Always drain BOTH buffers to prevent stale data buildup
            let mic_chunk = {
                let mut buf = mic_buffer.lock().unwrap();
                buf.drain_chunk()
            };
            let capture_chunk = {
                let mut buf = capture_buffer.lock().unwrap();
                buf.drain_chunk()
            };

            // Only transcribe mic — capture is drained but discarded
            // In a real meeting, you'd merge or alternate, but for now
            // mic is the primary source
            let chunk = mic_chunk.or(capture_chunk);

            if let Some(audio_data) = chunk {
                // Skip silence — this prevents hallucinations
                if WhisperEngine::is_silence(&audio_data) {
                    continue;
                }

                let offset = start_time.elapsed().as_secs_f64();
                match engine.transcribe(&audio_data) {
                    Ok(text) if !text.is_empty() => {
                        // Deduplicate
                        if text == last_text {
                            continue;
                        }
                        last_text = text.clone();

                        {
                            let mut store = transcript_for_whisper.lock().unwrap();
                            store.add(text.clone(), offset);
                        }
                        let segment = TranscriptSegment {
                            timestamp: chrono::Utc::now(),
                            text,
                            offset_secs: offset,
                        };
                        let _ = win.emit("new-transcript", &segment);
                    }
                    Err(e) => eprintln!("[whisper] Error: {}", e),
                    _ => {}
                }
            }
        }
    });

    // Spawn advisor loop
    let transcript_for_advisor = transcript.clone();
    let win_for_advisor = window.clone();
    tokio::spawn(async move {
        let config = config::load_config().unwrap_or_default();
        let advisor = crate::advisor::engine::AdvisorEngine::new(
            &config.llm.base_url,
            &config.llm.api_key,
            &config.llm.model,
        );

        let templates_list = templates::list_templates().unwrap_or_default();
        let template = templates_list.first().cloned();

        let mut summary_interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        let mut advice_interval = tokio::time::interval(tokio::time::Duration::from_secs(10));

        loop {
            tokio::select! {
                _ = summary_interval.tick() => {
                    let text = {
                        let store = transcript_for_advisor.lock().unwrap();
                        store.full_text()
                    };
                    if !text.is_empty() {
                        if let Ok(summary) = advisor.generate_summary(&text, "").await {
                            let _ = win_for_advisor.emit("meeting-summary", &summary);
                        }
                    }
                }
                _ = advice_interval.tick() => {
                    if let Some(ref tmpl) = template {
                        let recent = {
                            let store = transcript_for_advisor.lock().unwrap();
                            store.recent_text(30.0)
                        };
                        if !recent.is_empty() {
                            if let Some(trigger) = crate::advisor::rules::evaluate_triggers(
                                &recent, &tmpl.trigger_hints, 10.0
                            ) {
                                let offset = start_time.elapsed().as_secs_f64();
                                if let Ok(advice) = advisor.generate_advice(
                                    tmpl, &recent, &trigger.reason, "", offset
                                ).await {
                                    let _ = win_for_advisor.emit("speaking-advice", &advice);
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    Ok(())
}

#[command]
pub async fn stop_recording(
    state: tauri::State<'_, SharedRecordingState>,
) -> Result<(), String> {
    let mut rec = state.lock().await;
    rec.is_recording = false;
    rec.start_time = None;
    // Streams will be dropped when this function's scope ends
    // In production, store streams in RecordingState and drop them here
    Ok(())
}

#[command]
pub async fn get_transcript(
    state: tauri::State<'_, SharedRecordingState>,
) -> Result<Vec<TranscriptSegment>, String> {
    let rec = state.lock().await;
    let store = rec.transcript.lock().unwrap();
    Ok(store.all().to_vec())
}

// --- Templates ---

#[command]
pub fn get_templates() -> Result<Vec<MeetingTemplate>, String> {
    templates::list_templates().map_err(|e| e.to_string())
}

#[command]
pub fn save_template(template: MeetingTemplate) -> Result<(), String> {
    templates::save_template(&template).map_err(|e| e.to_string())
}

#[command]
pub fn delete_template(id: String) -> Result<(), String> {
    templates::delete_template(&id).map_err(|e| e.to_string())
}

// --- Config ---

#[command]
pub fn get_config() -> Result<AppConfig, String> {
    config::load_config().map_err(|e| e.to_string())
}

#[command]
pub fn save_app_config(config: AppConfig) -> Result<(), String> {
    config::save_config(&config).map_err(|e| e.to_string())
}

// --- History ---

#[command]
pub fn save_meeting(record: MeetingRecord) -> Result<(), String> {
    history::save_meeting(&record).map_err(|e| e.to_string())
}

#[command]
pub fn list_meetings() -> Result<Vec<MeetingRecord>, String> {
    history::list_meetings().map_err(|e| e.to_string())
}

// --- Documents ---

#[command]
pub fn load_document(path: String) -> Result<LoadedDocument, String> {
    loader::load_document(std::path::Path::new(&path)).map_err(|e| e.to_string())
}
