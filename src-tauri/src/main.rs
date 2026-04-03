#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio;
mod advisor;
mod commands;
mod documents;
mod storage;
mod transcript;
mod whisper;

use commands::SharedRecordingState;
use std::sync::Arc;
use tokio::sync::Mutex;

fn main() {
    env_logger::init();

    let _ = crate::advisor::templates::ensure_default_templates(std::path::Path::new("../templates"));

    let recording_state: SharedRecordingState =
        Arc::new(Mutex::new(commands::RecordingState::new()));

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(recording_state)
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::list_audio_devices,
            commands::check_whisper_model,
            commands::download_whisper_model,
            commands::start_recording,
            commands::stop_recording,
            commands::get_transcript,
            commands::get_templates,
            commands::save_template,
            commands::delete_template,
            commands::get_config,
            commands::save_app_config,
            commands::load_document,
            commands::save_meeting,
            commands::list_meetings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
