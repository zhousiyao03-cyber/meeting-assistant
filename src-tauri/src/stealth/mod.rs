pub mod window;

use std::sync::atomic::{AtomicBool, Ordering};

static STEALTH_ON: AtomicBool = AtomicBool::new(false);

pub fn is_stealth_on() -> bool {
    STEALTH_ON.load(Ordering::Acquire)
}

pub fn set_stealth(on: bool) {
    STEALTH_ON.store(on, Ordering::Release);
}

/// Apply current stealth state to a Tauri window.
#[cfg(target_os = "macos")]
pub fn apply_to_window(app_window: &tauri::WebviewWindow) -> tauri::Result<()> {
    let ns_window = app_window.ns_window()?;
    let on = is_stealth_on();
    unsafe {
        window::set_window_sharing_type(ns_window, on);
    }
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn apply_to_window(_app_window: &tauri::WebviewWindow) -> tauri::Result<()> {
    Ok(())
}
