use serde::Serialize;
use tauri::command;
use tauri::Emitter;
use tauri::Manager;

use crate::advisor::templates::{self, MeetingTemplate};
use crate::asr::{create_asr_provider, AsrConfig, TranscriptChunk, TranscriptCallback};
use crate::audio::buffer::{create_shared_buffer, SharedBuffer};
use crate::audio::capture;
use crate::audio::permission;
use crate::audio::system_capture::{create_system_audio_capture, AppFilter};
use crate::documents::loader::{self, LoadedDocument};
use crate::license::{self, UserPlan};
use crate::storage::config::{self, AppConfig};
use crate::storage::history::{self, MeetingRecord};
use crate::transcript::store::{create_shared_store, SharedTranscriptStore, TranscriptSegment};
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
    format!("Hello, {}! VoiceNote is running.", name)
}

// --- Whisper (legacy local model — kept for v1.1 fallback) ---

#[derive(Serialize)]
pub struct ModelStatus {
    pub downloaded: bool,
    pub path: Option<String>,
}

#[command]
pub fn check_whisper_model() -> Result<ModelStatus, String> {
    let path = crate::whisper::downloader::model_path().map_err(|e| e.to_string())?;
    Ok(ModelStatus {
        downloaded: path.is_some(),
        path: path.map(|p| p.to_string_lossy().to_string()),
    })
}

#[command]
pub async fn download_whisper_model(window: tauri::Window) -> Result<String, String> {
    let path = crate::whisper::downloader::download_model(move |downloaded, total| {
        let _ = window.emit(
            "model-download-progress",
            serde_json::json!({ "downloaded": downloaded, "total": total }),
        );
    })
    .await
    .map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

// --- Permission ---

#[derive(Serialize)]
pub struct ScreenRecordingPermissionStatus {
    pub status: String,
    pub macos_version_ok: bool,
}

#[command]
pub fn check_screen_recording_permission() -> Result<ScreenRecordingPermissionStatus, String> {
    let status = match permission::check_screen_recording_permission() {
        permission::PermissionStatus::Granted => "granted",
        permission::PermissionStatus::Denied => "denied",
        permission::PermissionStatus::NotDetermined => "not-determined",
    };
    Ok(ScreenRecordingPermissionStatus {
        status: status.to_string(),
        macos_version_ok: permission::macos_version_at_least(13, 0),
    })
}

#[command]
pub fn open_screen_recording_settings() -> Result<(), String> {
    permission::open_settings_screen_recording().map_err(|e| e.to_string())
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
    pub context_note: String,
    pub active_locale: String,
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
            context_note: String::new(),
            active_locale: "en-US".into(),
        }
    }
}

pub type SharedRecordingState = Arc<TokioMutex<RecordingState>>;

#[command]
pub async fn start_recording(
    mic_device: String,
    state: tauri::State<'_, SharedRecordingState>,
    window: tauri::Window,
) -> Result<(), String> {
    if !permission::macos_version_at_least(13, 0) {
        return Err("Confide requires macOS 13.0 or later.".into());
    }
    if matches!(
        permission::check_screen_recording_permission(),
        permission::PermissionStatus::Denied | permission::PermissionStatus::NotDetermined
    ) {
        return Err(
            "Screen Recording permission required. Open System Settings → Privacy & Security → Screen Recording, enable VoiceNote, and restart the app.".into(),
        );
    }

    let mut rec = state.lock().await;
    if rec.is_recording {
        return Err("Already recording".into());
    }

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
    let start_time = rec.start_time.unwrap();
    let active_locale = rec.active_locale.clone();

    drop(rec);

    // === 1. Mic capture via cpal ===
    let state_for_mic_thread = Arc::clone(&state);
    let mic_buf_for_thread = mic_buffer.clone();
    let win_for_mic_err = window.clone();
    std::thread::spawn(move || {
        let mic_stream = match capture::start_capture(&mic_device, mic_buf_for_thread) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[audio] Mic capture failed: {}", e);
                let _ = win_for_mic_err.emit(
                    "backend-error",
                    serde_json::json!({
                        "source": "audio",
                        "message": format!("Microphone start failed: {}", e)
                    }),
                );
                return;
            }
        };
        eprintln!("[audio] Mic stream started, holding alive...");
        loop {
            std::thread::sleep(std::time::Duration::from_millis(200));
            if let Ok(rec) = state_for_mic_thread.try_lock() {
                if !rec.is_recording {
                    break;
                }
            }
        }
        drop(mic_stream);
        eprintln!("[audio] Mic stream dropped");
    });

    // === 2. System audio capture via ScreenCaptureKit ===
    let cap_buf_for_sckit = capture_buffer.clone();
    let win_for_sckit = window.clone();
    let state_for_sckit: SharedRecordingState = Arc::clone(&state);
    tokio::spawn(async move {
        let mut sckit = match create_system_audio_capture(cap_buf_for_sckit, AppFilter::default())
        {
            Ok(b) => b,
            Err(e) => {
                let _ = win_for_sckit.emit(
                    "backend-error",
                    serde_json::json!({
                        "source": "audio",
                        "message": format!("System audio init failed: {}", e)
                    }),
                );
                return;
            }
        };
        if let Err(e) = sckit.start() {
            let _ = win_for_sckit.emit(
                "backend-error",
                serde_json::json!({
                    "source": "audio",
                    "message": format!("System audio start failed: {}", e)
                }),
            );
            return;
        }
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            let rec = state_for_sckit.lock().await;
            if !rec.is_recording {
                break;
            }
        }
        let _ = sckit.stop();
        eprintln!("[audio] System audio capture stopped");
    });

    // === 3. ASR loop: GPT-Realtime-Whisper ===
    let asr_config = {
        let cfg = config::load_config().unwrap_or_default();
        AsrConfig {
            provider: "openai-realtime-whisper".into(),
            openai_api_key: if cfg.openai_asr_api_key.is_empty() {
                std::env::var("OPENAI_API_KEY").unwrap_or_default()
            } else {
                cfg.openai_asr_api_key
            },
            openai_model: cfg.openai_asr_model,
            language_hint: cfg.language_preference,
        }
    };
    let win_for_asr_outer = window.clone();
    let win_for_asr_cb = window.clone();
    let transcript_for_asr = transcript.clone();
    let state_for_asr_loop: SharedRecordingState = Arc::clone(&state);
    let mic_buf_for_asr = mic_buffer.clone();
    let cap_buf_for_asr = capture_buffer.clone();
    tokio::spawn(async move {
        let on_transcript: TranscriptCallback = std::sync::Arc::new(move |chunk: TranscriptChunk| {
            if chunk.is_final {
                {
                    let mut store = transcript_for_asr.lock().unwrap();
                    store.add(chunk.text.clone(), chunk.offset_secs, &chunk.speaker);
                }
                let segment = TranscriptSegment {
                    timestamp: chrono::Utc::now(),
                    text: chunk.text.clone(),
                    offset_secs: chunk.offset_secs,
                    speaker: chunk.speaker.clone(),
                };
                let _ = win_for_asr_cb.emit("new-transcript", &segment);
            } else {
                let _ = win_for_asr_cb.emit("transcript-delta", &chunk);
            }
        });

        let mut asr = match create_asr_provider(&asr_config, on_transcript) {
            Ok(p) => p,
            Err(e) => {
                let _ = win_for_asr_outer.emit(
                    "backend-error",
                    serde_json::json!({
                        "source": "asr",
                        "message": format!("ASR init failed: {}", e)
                    }),
                );
                return;
            }
        };
        if let Err(e) = asr.start().await {
            let _ = win_for_asr_outer.emit(
                "backend-error",
                serde_json::json!({
                    "source": "asr",
                    "message": format!("ASR session start failed: {}", e)
                }),
            );
            return;
        }

        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            let (is_recording, is_paused) = {
                let rec = state_for_asr_loop.lock().await;
                (rec.is_recording, rec.is_paused)
            };
            if !is_recording {
                break;
            }
            if is_paused {
                continue;
            }

            let mic_data = {
                let mut buf = mic_buf_for_asr.lock().unwrap();
                if buf.len() > 0 { buf.drain_all() } else { vec![] }
            };
            let cap_data = {
                let mut buf = cap_buf_for_asr.lock().unwrap();
                if buf.len() > 0 { buf.drain_all() } else { vec![] }
            };

            if !mic_data.is_empty() || !cap_data.is_empty() {
                let mixed = mix_audio(&mic_data, &cap_data);
                if let Err(e) = asr.send_audio(&mixed, "mixed").await {
                    eprintln!("[asr] send_audio error: {}", e);
                }
            }
        }
        let _ = asr.stop().await;
    });

    // === 4. Meter loop: 5-minute sync ===
    let state_for_meter: SharedRecordingState = Arc::clone(&state);
    let win_for_meter = window.clone();
    tokio::spawn(async move {
        let key = match license::storage::get_license_key() {
            Ok(Some(k)) => k,
            _ => {
                eprintln!("[meter] No license key (free trial mode)");
                return;
            }
        };
        let meeting_id = uuid::Uuid::new_v4().to_string();
        let provider = "confide".to_string();
        let mut meter = license::metering::Meter::new(meeting_id, provider);

        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;

            let recording = {
                let rec = state_for_meter.lock().await;
                rec.is_recording
            };
            if !recording {
                if let Some(evt) = meter.create_final_event() {
                    let _ = license::metering::sync_usage(&key, vec![evt]).await;
                }
                break;
            }

            if let Some(evt) = meter.maybe_create_event() {
                let secs_used = evt.seconds_used;
                match license::metering::sync_usage(&key, vec![evt]).await {
                    Ok(()) => eprintln!("[meter] synced {} sec", secs_used),
                    Err(e) => eprintln!("[meter] sync failed (will retry): {}", e),
                }
                match license::verify::fetch_plan(&key).await {
                    Ok(plan) => {
                        let remaining = plan.quota_remaining_seconds();
                        let _ = win_for_meter.emit("plan-updated", &plan);
                        if remaining < 60 && remaining > 0 {
                            let _ = win_for_meter.emit("quota-low", remaining);
                        }
                        if remaining <= 0 && !plan.auto_topup_enabled {
                            let _ = win_for_meter.emit("quota-exhausted", ());
                            let mut rec = state_for_meter.lock().await;
                            rec.is_recording = false;
                            break;
                        }
                    }
                    Err(e) => eprintln!("[meter] plan refresh failed: {}", e),
                }
            }
        }
    });

    // === 5. Advisor loop ===
    let state_for_advisor: SharedRecordingState = Arc::clone(&state);
    let transcript_for_advisor = transcript.clone();
    let win_for_advisor = window.clone();
    spawn_advisor_loop(
        state_for_advisor,
        transcript_for_advisor,
        win_for_advisor,
        start_time,
        active_locale,
    );

    Ok(())
}

fn mix_audio(a: &[f32], b: &[f32]) -> Vec<f32> {
    let n = a.len().max(b.len());
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let av = a.get(i).copied().unwrap_or(0.0);
        let bv = b.get(i).copied().unwrap_or(0.0);
        if a.len() > i && b.len() > i {
            out.push((av + bv) * 0.5);
        } else {
            out.push(av + bv);
        }
    }
    out
}

fn spawn_advisor_loop(
    state: SharedRecordingState,
    transcript: SharedTranscriptStore,
    window: tauri::Window,
    start_time: std::time::Instant,
    active_locale: String,
) {
    tokio::spawn(async move {
        let config = config::load_config().unwrap_or_default();
        eprintln!(
            "[advisor] LLM config: base_url={}, model={}, byo={}",
            config.llm.base_url, config.llm.model, config.byo.active
        );

        // Choose LLM provider based on byo mode
        let advisor = if config.byo.active && !config.byo.anthropic_api_key.is_empty() {
            crate::advisor::engine::AdvisorEngine::from_mode(
                &crate::llm::LlmMode::UserAnthropic {
                    api_key: config.byo.anthropic_api_key.clone(),
                    model: config.byo.anthropic_model.clone(),
                },
            )
        } else if config.byo.active && !config.byo.openai_api_key.is_empty() {
            crate::advisor::engine::AdvisorEngine::from_mode(
                &crate::llm::LlmMode::UserOpenAi {
                    api_key: config.byo.openai_api_key.clone(),
                    model: "gpt-4o".into(),
                    base_url: "https://api.openai.com/v1".into(),
                },
            )
        } else {
            crate::advisor::engine::AdvisorEngine::new(
                &config.llm.base_url,
                &config.llm.api_key,
                &config.llm.model,
            )
        };

        let templates_list =
            templates::list_templates_for_locale(&active_locale).unwrap_or_default();
        eprintln!("[advisor] Loaded {} templates for locale {}", templates_list.len(), active_locale);

        let mut summary_interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        let mut advice_interval = tokio::time::interval(tokio::time::Duration::from_secs(10));

        let mut last_advice_time =
            std::time::Instant::now() - std::time::Duration::from_secs(60);
        let mut last_trigger_reason = String::new();
        let mut last_advice_transcript_len: usize = 0;
        const ADVICE_COOLDOWN_SECS: u64 = 30;
        const MIN_NEW_CHARS: usize = 50;

        loop {
            {
                let rec = state.lock().await;
                if !rec.is_recording {
                    break;
                }
                if rec.is_paused {
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    continue;
                }
            }

            let (ref_docs, ctx_note) = {
                let rec = state.lock().await;
                (rec.reference_docs.clone(), rec.context_note.clone())
            };

            tokio::select! {
                _ = summary_interval.tick() => {
                    let text = {
                        let store = transcript.lock().unwrap();
                        store.full_text()
                    };
                    if !text.is_empty() {
                        match advisor.generate_summary(&text, &ref_docs).await {
                            Ok(summary) => {
                                let _ = window.emit("meeting-summary", &summary);
                            }
                            Err(e) => eprintln!("[advisor] Summary error: {}", e),
                        }
                    }
                }
                _ = advice_interval.tick() => {
                    if last_advice_time.elapsed().as_secs() < ADVICE_COOLDOWN_SECS {
                        continue;
                    }
                    let tmpl = {
                        let rec = state.lock().await;
                        match &rec.active_template_id {
                            Some(id) => templates_list.iter().find(|t| t.id == *id).cloned(),
                            None => templates_list.first().cloned(),
                        }
                    };
                    if let Some(ref tmpl) = tmpl {
                        let recent = {
                            let store = transcript.lock().unwrap();
                            store.recent_text(30.0)
                        };
                        if recent.is_empty() {
                            continue;
                        }
                        if recent.len().saturating_sub(last_advice_transcript_len) < MIN_NEW_CHARS
                            && last_advice_transcript_len > 0
                        {
                            continue;
                        }

                        if let Some(trigger) = crate::advisor::rules::evaluate_triggers(
                            &recent, &tmpl.trigger_config, 10.0,
                        ) {
                            if trigger.reason == last_trigger_reason
                                && last_advice_time.elapsed().as_secs()
                                    < ADVICE_COOLDOWN_SECS * 2
                            {
                                continue;
                            }
                            let offset = start_time.elapsed().as_secs_f64();
                            match advisor
                                .generate_advice(
                                    tmpl,
                                    &recent,
                                    &trigger.reason,
                                    &ref_docs,
                                    &ctx_note,
                                    offset,
                                )
                                .await
                            {
                                Ok(advice) => {
                                    last_advice_time = std::time::Instant::now();
                                    last_trigger_reason = trigger.reason.clone();
                                    last_advice_transcript_len = recent.len();
                                    let _ = window.emit("speaking-advice", &advice);
                                }
                                Err(e) => eprintln!("[advisor] Advice error: {}", e),
                            }
                        }
                    }
                }
            }
        }
    });
}

#[command]
pub async fn stop_recording(state: tauri::State<'_, SharedRecordingState>) -> Result<(), String> {
    let mut rec = state.lock().await;
    rec.is_recording = false;
    rec.is_paused = false;
    rec.start_time = None;
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
pub fn get_templates_for_locale(locale: String) -> Result<Vec<MeetingTemplate>, String> {
    templates::list_templates_for_locale(&locale).map_err(|e| e.to_string())
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

#[command]
pub fn delete_meeting(id: String) -> Result<(), String> {
    history::delete_meeting(&id).map_err(|e| e.to_string())
}

// --- Meeting Minutes ---

#[command]
pub async fn generate_meeting_minutes(
    transcript: String,
    summary: String,
) -> Result<crate::advisor::engine::MeetingMinutes, String> {
    let cfg = config::load_config().map_err(|e| e.to_string())?;
    let advisor = crate::advisor::engine::AdvisorEngine::new(
        &cfg.llm.base_url,
        &cfg.llm.api_key,
        &cfg.llm.model,
    );
    advisor
        .generate_minutes(&transcript, &summary)
        .await
        .map_err(|e| e.to_string())
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
    eprintln!(
        "[docs] Loaded reference doc: {} ({} chars)",
        doc.filename,
        doc.content.len()
    );
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

// --- Active Template + Context Note + Locale ---

#[command]
pub async fn set_active_template(
    id: String,
    state: tauri::State<'_, SharedRecordingState>,
) -> Result<(), String> {
    let mut rec = state.lock().await;
    rec.active_template_id = Some(id);
    Ok(())
}

#[command]
pub async fn set_meeting_context_note(
    note: String,
    state: tauri::State<'_, SharedRecordingState>,
) -> Result<(), String> {
    if note.chars().count() > 500 {
        return Err("Context note must be ≤500 characters".into());
    }
    let mut rec = state.lock().await;
    rec.context_note = note;
    Ok(())
}

#[command]
pub async fn get_meeting_context_note(
    state: tauri::State<'_, SharedRecordingState>,
) -> Result<String, String> {
    let rec = state.lock().await;
    Ok(rec.context_note.clone())
}

#[command]
pub async fn set_active_locale(
    locale: String,
    state: tauri::State<'_, SharedRecordingState>,
) -> Result<(), String> {
    let mut rec = state.lock().await;
    rec.active_locale = locale;
    Ok(())
}

// --- Recording Status / Pause / Resume ---

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

#[command]
pub async fn pause_recording(
    state: tauri::State<'_, SharedRecordingState>,
) -> Result<(), String> {
    let mut rec = state.lock().await;
    if !rec.is_recording {
        return Err("Not recording".into());
    }
    rec.is_paused = true;
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
    Ok(())
}

// --- Stealth ---

#[command]
pub fn set_stealth_mode(on: bool, app: tauri::AppHandle) -> Result<(), String> {
    crate::stealth::set_stealth(on);
    if let Some(w) = app.get_webview_window("main") {
        crate::stealth::apply_to_window(&w).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[command]
pub fn is_stealth_on() -> Result<bool, String> {
    Ok(crate::stealth::is_stealth_on())
}

// --- License ---

#[command]
pub async fn get_user_plan() -> Result<UserPlan, String> {
    let key = license::storage::get_license_key().map_err(|e| e.to_string())?;
    if let Some(k) = key {
        match license::verify::fetch_plan(&k).await {
            Ok(p) => {
                let cached = license::storage::CachedPlan {
                    plan: p.clone(),
                    cached_at: chrono::Utc::now().timestamp(),
                    pending_usage: vec![],
                };
                let _ = license::storage::save_cached(&cached);
                Ok(p)
            }
            Err(e) => {
                if let Ok(Some(c)) = license::storage::load_cached() {
                    let age_days = (chrono::Utc::now().timestamp() - c.cached_at) / 86400;
                    if age_days <= 7 {
                        return Ok(c.plan);
                    }
                }
                Err(format!("Cannot verify license: {}", e))
            }
        }
    } else {
        Ok(UserPlan::free_default())
    }
}

#[command]
pub async fn set_license_key(key: String) -> Result<UserPlan, String> {
    license::storage::set_license_key(&key).map_err(|e| e.to_string())?;
    license::verify::fetch_plan(&key).await.map_err(|e| e.to_string())
}

#[command]
pub async fn clear_license_key() -> Result<(), String> {
    license::storage::clear_license_key().map_err(|e| e.to_string())
}
