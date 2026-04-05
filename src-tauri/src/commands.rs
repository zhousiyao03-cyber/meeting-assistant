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
use crate::whisper::engine::SherpaEngine;
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

/// Simple character-level similarity (Jaccard on chars) for cross-channel dedup.
fn text_similarity(a: &str, b: &str) -> f64 {
    if a.is_empty() || b.is_empty() { return 0.0; }
    let chars_a: std::collections::HashSet<char> = a.chars().collect();
    let chars_b: std::collections::HashSet<char> = b.chars().collect();
    let intersection = chars_a.intersection(&chars_b).count();
    let union = chars_a.union(&chars_b).count();
    if union == 0 { return 0.0; }
    intersection as f64 / union as f64
}

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

    // Reset state from previous recording
    {
        let mut mic_buf = rec.mic_buffer.lock().unwrap();
        mic_buf.drain_all();
    }
    {
        let mut cap_buf = rec.capture_buffer.lock().unwrap();
        cap_buf.drain_all();
    }
    {
        let mut store = rec.transcript.lock().unwrap();
        store.clear();
    }
    rec.reference_docs.clear();

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
    let win_for_error = window.clone();
    std::thread::spawn(move || {
        let mic_stream = match capture::start_capture(&mic_device, mic_buf_for_thread) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[audio] Mic capture failed: {}", e);
                let _ = win_for_error.emit("backend-error", serde_json::json!({
                    "source": "audio",
                    "message": format!("麦克风启动失败: {}", e)
                }));
                return;
            }
        };
        let capture_stream = match capture::start_capture(&capture_device, cap_buf_for_thread) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[audio] System capture failed: {}", e);
                let _ = win_for_error.emit("backend-error", serde_json::json!({
                    "source": "audio",
                    "message": format!("系统音频捕获失败: {}", e)
                }));
                return;
            }
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

    // Spawn sherpa-onnx ASR processing (SenseVoice + Silero VAD)
    let start_time = rec.start_time.unwrap();
    let win = window.clone();
    let transcript_for_whisper = transcript.clone();
    tokio::spawn(async move {
        let model_dir = match crate::whisper::downloader::model_path() {
            Ok(Some(p)) => p,
            _ => {
                eprintln!("[sherpa] Model not found");
                let _ = win.emit("backend-error", serde_json::json!({
                    "source": "asr",
                    "message": "语音模型未下载，请先在设置中下载模型"
                }));
                return;
            }
        };
        let mic_engine = match SherpaEngine::new(&model_dir) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("[sherpa] Failed to load mic engine: {}", e);
                let _ = win.emit("backend-error", serde_json::json!({
                    "source": "asr",
                    "message": format!("语音模型加载失败: {}", e)
                }));
                return;
            }
        };
        let capture_engine = match SherpaEngine::new(&model_dir) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("[sherpa] Failed to load capture engine: {}", e);
                let _ = win.emit("backend-error", serde_json::json!({
                    "source": "asr",
                    "message": format!("语音模型加载失败(capture): {}", e)
                }));
                return;
            }
        };
        eprintln!("[sherpa] Dual engines loaded (mic + capture), starting transcription loop");

        // Track when each channel last had active audio (RMS above threshold)
        let mut capture_last_active: f64 = 0.0;
        let mut mic_last_active: f64 = 0.0;
        const ECHO_SUPPRESS_WINDOW: f64 = 1.5; // seconds
        const ACTIVE_RMS_THRESHOLD: f32 = 0.005;

        // Recent segments for cross-channel dedup
        let mut recent_segments: Vec<(String, f64, String)> = Vec::new();

        let emit_segment = |text: &str, offset: f64, speaker: &str,
                            recent: &mut Vec<(String, f64, String)>| {
            // Dedup: if the other channel produced a similar text within 3 seconds, skip
            let dominated = recent.iter().any(|(prev_text, prev_offset, prev_speaker)| {
                prev_speaker != speaker
                    && (offset - prev_offset).abs() < 3.0
                    && text_similarity(text, prev_text) > 0.5
            });
            if dominated {
                eprintln!("[sherpa] Dedup: skipping '{}' from {} (duplicate of other channel)", text, speaker);
                return;
            }
            recent.push((text.to_string(), offset, speaker.to_string()));
            if recent.len() > 20 { recent.drain(..recent.len() - 20); }

            {
                let mut store = transcript_for_whisper.lock().unwrap();
                store.add(text.to_string(), offset, speaker);
            }
            let segment = TranscriptSegment {
                timestamp: chrono::Utc::now(),
                text: text.to_string(),
                offset_secs: offset,
                speaker: speaker.to_string(),
            };
            let _ = win.emit("new-transcript", &segment);
        };

        /// Compute RMS energy of audio samples
        fn rms_energy(samples: &[f32]) -> f32 {
            if samples.is_empty() { return 0.0; }
            let sum: f32 = samples.iter().map(|s| s * s).sum();
            (sum / samples.len() as f32).sqrt()
        }

        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            let is_recording;
            let is_paused;
            {
                let rec = state_for_whisper.lock().await;
                is_recording = rec.is_recording;
                is_paused = rec.is_paused;
            }

            if !is_recording {
                let offset = start_time.elapsed().as_secs_f64();
                for text in mic_engine.flush() {
                    emit_segment(&text, offset, "me", &mut recent_segments);
                }
                for text in capture_engine.flush() {
                    emit_segment(&text, offset, "other", &mut recent_segments);
                }
                break;
            }

            if is_paused { continue; }

            let now = start_time.elapsed().as_secs_f64();

            // Drain mic audio
            let mic_data = {
                let mut buf = mic_buffer.lock().unwrap();
                if buf.len() > 0 { buf.drain_all() } else { vec![] }
            };

            // Drain capture audio
            let cap_data = {
                let mut buf = capture_buffer.lock().unwrap();
                if buf.len() > 0 { buf.drain_all() } else { vec![] }
            };

            // Track channel activity
            if !cap_data.is_empty() && rms_energy(&cap_data) > ACTIVE_RMS_THRESHOLD {
                capture_last_active = now;
            }
            if !mic_data.is_empty() && rms_energy(&mic_data) > ACTIVE_RMS_THRESHOLD {
                mic_last_active = now;
            }

            // Process capture audio ("other") — always process
            if !cap_data.is_empty() {
                for text in capture_engine.process_audio(&cap_data) {
                    emit_segment(&text, now, "other", &mut recent_segments);
                }
            }

            // Process mic audio ("me") — suppress if capture channel was recently active
            // (likely echo from speaker being picked up by mic)
            if !mic_data.is_empty() {
                let capture_was_active = (now - capture_last_active) < ECHO_SUPPRESS_WINDOW;
                let mic_is_active = (now - mic_last_active) < 0.5;

                // Feed audio to engine regardless (keeps VAD state consistent)
                let texts = mic_engine.process_audio(&mic_data);

                if capture_was_active && mic_is_active {
                    // Both active: likely echo. Only emit if mic energy is significantly
                    // higher than capture (user actually talking over the speaker)
                    for text in texts {
                        eprintln!("[sherpa] Echo suppress: dropping mic '{}' (capture was active)", text);
                    }
                } else {
                    for text in texts {
                        emit_segment(&text, now, "me", &mut recent_segments);
                    }
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

        // Cooldown: track last advice time and last trigger reason to avoid repetition
        let mut last_advice_time = std::time::Instant::now() - std::time::Duration::from_secs(60);
        let mut last_trigger_reason = String::new();
        let mut last_advice_transcript_len: usize = 0;
        const ADVICE_COOLDOWN_SECS: u64 = 30;
        const MIN_NEW_CHARS: usize = 50;

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
                    // Cooldown check: skip if too soon since last advice
                    if last_advice_time.elapsed().as_secs() < ADVICE_COOLDOWN_SECS {
                        continue;
                    }

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
                        if recent.is_empty() { continue; }

                        // Skip if not enough new content since last advice
                        if recent.len().saturating_sub(last_advice_transcript_len) < MIN_NEW_CHARS
                            && last_advice_transcript_len > 0 {
                            continue;
                        }

                        if let Some(trigger) = crate::advisor::rules::evaluate_triggers(
                            &recent, &tmpl.trigger_config, 10.0
                        ) {
                            // Skip if same trigger reason fired consecutively
                            if trigger.reason == last_trigger_reason
                                && last_advice_time.elapsed().as_secs() < ADVICE_COOLDOWN_SECS * 2 {
                                eprintln!("[advisor] Skipping duplicate trigger: {}", trigger.reason);
                                continue;
                            }

                            eprintln!("[advisor] Trigger fired: {}", trigger.reason);
                            let offset = start_time.elapsed().as_secs_f64();
                            match advisor.generate_advice(
                                tmpl, &recent, &trigger.reason, &ref_docs, offset
                            ).await {
                                Ok(advice) => {
                                    eprintln!("[advisor] Advice: {}", advice.suggestion);
                                    last_advice_time = std::time::Instant::now();
                                    last_trigger_reason = trigger.reason.clone();
                                    last_advice_transcript_len = recent.len();
                                    let _ = win_for_advisor.emit("speaking-advice", &advice);
                                }
                                Err(e) => eprintln!("[advisor] Advice error: {}", e),
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

// --- Meeting Minutes ---

#[command]
pub async fn generate_meeting_minutes(
    transcript: String,
    summary: String,
) -> Result<crate::advisor::engine::MeetingMinutes, String> {
    let config = config::load_config().map_err(|e| e.to_string())?;
    let advisor = crate::advisor::engine::AdvisorEngine::new(
        &config.llm.base_url,
        &config.llm.api_key,
        &config.llm.model,
    );
    advisor.generate_minutes(&transcript, &summary)
        .await
        .map_err(|e| e.to_string())
}

#[command]
pub fn delete_meeting(id: String) -> Result<(), String> {
    history::delete_meeting(&id).map_err(|e| e.to_string())
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
