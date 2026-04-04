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
    pub is_paused: bool,
    pub mic_buffer: SharedBuffer,
    pub capture_buffer: SharedBuffer,
    pub transcript: SharedTranscriptStore,
    pub start_time: Option<std::time::Instant>,
    pub reference_docs: String,
    pub active_template_id: Option<String>,
}

impl RecordingState {
    pub fn new() -> Self {
        Self {
            is_recording: false,
            is_paused: false,
            mic_buffer: create_shared_buffer(2, 16000),
            capture_buffer: create_shared_buffer(2, 16000),
            transcript: create_shared_store(),
            start_time: None,
            reference_docs: String::new(),
            active_template_id: None,
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
    rec.is_paused = false;
    rec.start_time = Some(std::time::Instant::now());
    let mic_buffer = rec.mic_buffer.clone();
    let capture_buffer = rec.capture_buffer.clone();
    let transcript = rec.transcript.clone();

    // Clone the Arc for spawned tasks
    let state_for_whisper: SharedRecordingState = Arc::clone(&state);
    let state_for_advisor: SharedRecordingState = Arc::clone(&state);

    // Start audio capture in a dedicated thread (cpal::Stream is !Send)
    let mic_buf_for_thread = mic_buffer.clone();
    let cap_buf_for_thread = capture_buffer.clone();
    let state_for_streams: SharedRecordingState = Arc::clone(&state);
    std::thread::spawn(move || {
        let mic_stream = match capture::start_capture(&mic_device, mic_buf_for_thread) {
            Ok(s) => s,
            Err(e) => { eprintln!("[audio] Mic capture failed: {}", e); return; }
        };
        let capture_stream = match capture::start_capture(&capture_device, cap_buf_for_thread) {
            Ok(s) => s,
            Err(e) => { eprintln!("[audio] System capture failed: {}", e); return; }
        };
        eprintln!("[audio] Streams started, holding alive...");
        loop {
            std::thread::sleep(std::time::Duration::from_millis(200));
            if let Ok(rec) = state_for_streams.try_lock() {
                if !rec.is_recording { break; }
            }
        }
        drop(mic_stream);
        drop(capture_stream);
        eprintln!("[audio] Streams dropped, capture stopped");
    });

    // Spawn Whisper processing
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

            // Check if paused or stopped
            {
                let rec = state_for_whisper.lock().await;
                if !rec.is_recording { break; }
                if rec.is_paused { continue; }
            }

            // Drain BOTH buffers every tick
            let mic_chunk = {
                let mut buf = mic_buffer.lock().unwrap();
                buf.drain_chunk()
            };
            let capture_chunk = {
                let mut buf = capture_buffer.lock().unwrap();
                buf.drain_chunk()
            };

            // Mix mic + capture into a single stream
            let audio_data = match (mic_chunk, capture_chunk) {
                (Some(mic), Some(cap)) => {
                    let len = mic.len().min(cap.len());
                    Some(mic[..len].iter().zip(&cap[..len])
                        .map(|(m, c)| (m + c) * 0.5)
                        .collect::<Vec<f32>>())
                }
                (Some(mic), None) => Some(mic),
                (None, Some(cap)) => Some(cap),
                (None, None) => None,
            };

            if let Some(audio_data) = audio_data {
                if WhisperEngine::is_silence(&audio_data) { continue; }

                let offset = start_time.elapsed().as_secs_f64();
                match engine.transcribe(&audio_data, &last_text) {
                    Ok(text) if !text.is_empty() => {
                        // Deduplicate
                        if text == last_text || last_text.contains(&text) || text.contains(&last_text) {
                            if text.len() <= last_text.len() { continue; }
                        }
                        last_text = text.clone();
                        eprintln!("[whisper] {}", &text);

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
        eprintln!("[advisor] LLM config: base_url={}, model={}", config.llm.base_url, config.llm.model);
        let advisor = crate::advisor::engine::AdvisorEngine::new(
            &config.llm.base_url,
            &config.llm.api_key,
            &config.llm.model,
        );

        let templates_list = templates::list_templates().unwrap_or_default();
        eprintln!("[advisor] Loaded {} templates", templates_list.len());

        let mut summary_interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        let mut advice_interval = tokio::time::interval(tokio::time::Duration::from_secs(10));

        loop {
            // Check if stopped or paused
            {
                let rec = state_for_advisor.lock().await;
                if !rec.is_recording {
                    break;
                }
                if rec.is_paused {
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    continue;
                }
            }

            // Read reference docs for context
            let ref_docs = {
                let rec = state_for_advisor.lock().await;
                rec.reference_docs.clone()
            };

            tokio::select! {
                _ = summary_interval.tick() => {
                    let text = {
                        let store = transcript_for_advisor.lock().unwrap();
                        store.full_text()
                    };
                    if !text.is_empty() {
                        eprintln!("[advisor] Generating summary ({} chars)...", text.len());
                        match advisor.generate_summary(&text, &ref_docs).await {
                            Ok(summary) => {
                                eprintln!("[advisor] Summary generated: {} points", summary.points.len());
                                let _ = win_for_advisor.emit("meeting-summary", &summary);
                            }
                            Err(e) => eprintln!("[advisor] Summary error: {}", e),
                        }
                    }
                }
                _ = advice_interval.tick() => {
                    // Pick template: use active_template_id if set, otherwise first
                    let tmpl = {
                        let rec = state_for_advisor.lock().await;
                        match &rec.active_template_id {
                            Some(id) => templates_list.iter().find(|t| t.id == *id).cloned(),
                            None => templates_list.first().cloned(),
                        }
                    };
                    if let Some(ref tmpl) = tmpl {
                        let recent = {
                            let store = transcript_for_advisor.lock().unwrap();
                            store.recent_text(30.0)
                        };
                        if !recent.is_empty() {
                            if let Some(trigger) = crate::advisor::rules::evaluate_triggers(
                                &recent, &tmpl.trigger_hints, 10.0
                            ) {
                                eprintln!("[advisor] Trigger fired: {}", trigger.reason);
                                let offset = start_time.elapsed().as_secs_f64();
                                match advisor.generate_advice(
                                    tmpl, &recent, &trigger.reason, &ref_docs, offset
                                ).await {
                                    Ok(advice) => {
                                        eprintln!("[advisor] Advice generated");
                                        let _ = win_for_advisor.emit("speaking-advice", &advice);
                                    }
                                    Err(e) => eprintln!("[advisor] Advice error: {}", e),
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
    rec.is_paused = false;
    rec.start_time = None;
    // The stream-holding thread will detect is_recording=false and exit,
    // dropping the cpal::Stream objects and stopping capture
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

#[command]
pub async fn load_reference_doc(
    path: String,
    state: tauri::State<'_, SharedRecordingState>,
) -> Result<String, String> {
    let doc = loader::load_document(std::path::Path::new(&path)).map_err(|e| e.to_string())?;
    let mut rec = state.lock().await;
    rec.reference_docs = doc.content.clone();
    eprintln!("[docs] Loaded reference doc: {} ({} chars)", doc.filename, doc.content.len());
    Ok(doc.filename)
}

#[command]
pub async fn clear_reference_doc(
    state: tauri::State<'_, SharedRecordingState>,
) -> Result<(), String> {
    let mut rec = state.lock().await;
    rec.reference_docs.clear();
    Ok(())
}

// --- Active Template ---

#[command]
pub async fn set_active_template(
    id: String,
    state: tauri::State<'_, SharedRecordingState>,
) -> Result<(), String> {
    let mut rec = state.lock().await;
    rec.active_template_id = Some(id);
    Ok(())
}

// --- Recording Status ---

#[derive(Serialize)]
pub struct RecordingStatusInfo {
    pub is_recording: bool,
    pub is_paused: bool,
    pub elapsed_secs: u64,
}

#[command]
pub async fn get_recording_status(
    state: tauri::State<'_, SharedRecordingState>,
) -> Result<RecordingStatusInfo, String> {
    let rec = state.lock().await;
    let elapsed = rec.start_time.map(|t| t.elapsed().as_secs()).unwrap_or(0);
    Ok(RecordingStatusInfo {
        is_recording: rec.is_recording,
        is_paused: rec.is_paused,
        elapsed_secs: elapsed,
    })
}

// --- Pause/Resume ---

#[command]
pub async fn pause_recording(
    state: tauri::State<'_, SharedRecordingState>,
) -> Result<(), String> {
    let mut rec = state.lock().await;
    if !rec.is_recording {
        return Err("Not recording".into());
    }
    rec.is_paused = true;
    eprintln!("[recording] Paused");
    Ok(())
}

#[command]
pub async fn resume_recording(
    state: tauri::State<'_, SharedRecordingState>,
) -> Result<(), String> {
    let mut rec = state.lock().await;
    if !rec.is_recording {
        return Err("Not recording".into());
    }
    rec.is_paused = false;
    eprintln!("[recording] Resumed");
    Ok(())
}
