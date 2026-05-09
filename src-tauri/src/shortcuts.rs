//! Global shortcuts for stealth-mode interactions.

use anyhow::Result;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

pub fn register(app: &AppHandle) -> Result<()> {
    let toggle = Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyH);
    let panic = Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyK);
    let opacity_up = Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::ArrowUp);
    let opacity_down = Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::ArrowDown);

    let app_for_toggle = app.clone();
    let app_for_panic = app.clone();
    let app_for_up = app.clone();
    let app_for_down = app.clone();

    let gs = app.global_shortcut();

    gs.on_shortcut(toggle, move |_app, _shortcut, event| {
        if event.state() == ShortcutState::Pressed {
            handle_toggle(&app_for_toggle);
        }
    })
    .map_err(|e| anyhow::anyhow!("register toggle shortcut: {:?}", e))?;

    gs.on_shortcut(panic, move |_app, _shortcut, event| {
        if event.state() == ShortcutState::Pressed {
            handle_panic(&app_for_panic);
        }
    })
    .map_err(|e| anyhow::anyhow!("register panic shortcut: {:?}", e))?;

    gs.on_shortcut(opacity_up, move |_app, _shortcut, event| {
        if event.state() == ShortcutState::Pressed {
            let _ = app_for_up.emit("opacity-step", 0.1f32);
        }
    })
    .map_err(|e| anyhow::anyhow!("register opacity_up: {:?}", e))?;

    gs.on_shortcut(opacity_down, move |_app, _shortcut, event| {
        if event.state() == ShortcutState::Pressed {
            let _ = app_for_down.emit("opacity-step", -0.1f32);
        }
    })
    .map_err(|e| anyhow::anyhow!("register opacity_down: {:?}", e))?;

    eprintln!("[shortcuts] Registered: ⌘⇧H toggle, ⌘⇧K panic, ⌘⇧↑/↓ opacity");
    Ok(())
}

fn handle_toggle(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

fn handle_panic(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    let _ = app.emit("panic-stop", ());
    eprintln!("[shortcuts] Panic key triggered: hidden + stop");
}
