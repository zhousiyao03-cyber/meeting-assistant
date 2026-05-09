#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use meeting_assistant::commands::{self, SharedRecordingState};
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

fn main() {
    env_logger::init();

    let _ = meeting_assistant::advisor::templates::ensure_default_templates(
        std::path::Path::new("../templates"),
    );

    let recording_state: SharedRecordingState =
        Arc::new(Mutex::new(commands::RecordingState::new()));

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(recording_state)
        .setup(|app| {
            // Stealth tray icon
            meeting_assistant::tray::setup(app.handle())?;

            // Global shortcuts (panic, toggle, opacity) — register but don't block setup on failure
            match meeting_assistant::shortcuts::register(app.handle()) {
                Ok(_) => {}
                Err(e) => eprintln!("[main] shortcut register failed: {}", e),
            }

            // Apply initial stealth state to main window + show
            if let Some(w) = app.get_webview_window("main") {
                let _ = meeting_assistant::stealth::apply_to_window(&w);
                let _ = w.show();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::list_audio_devices,
            commands::check_whisper_model,
            commands::download_whisper_model,
            commands::start_recording,
            commands::stop_recording,
            commands::get_transcript,
            commands::get_templates,
            commands::get_templates_for_locale,
            commands::save_template,
            commands::delete_template,
            commands::get_config,
            commands::save_app_config,
            commands::load_document,
            commands::load_reference_doc,
            commands::clear_reference_doc,
            commands::set_active_template,
            commands::set_meeting_context_note,
            commands::get_meeting_context_note,
            commands::set_active_locale,
            commands::get_recording_status,
            commands::pause_recording,
            commands::resume_recording,
            commands::save_meeting,
            commands::list_meetings,
            commands::generate_meeting_minutes,
            commands::delete_meeting,
            commands::check_screen_recording_permission,
            commands::open_screen_recording_settings,
            commands::set_stealth_mode,
            commands::is_stealth_on,
            commands::get_user_plan,
            commands::set_license_key,
            commands::clear_license_key,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
