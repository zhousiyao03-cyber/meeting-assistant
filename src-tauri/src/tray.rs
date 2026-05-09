//! Menu bar icon — only entry point to open the app since LSUIElement=true
//! removes the Dock icon.

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let new_interview = MenuItem::with_id(app, "new_interview", "New Interview Meeting", true, None::<&str>)?;
    let new_general = MenuItem::with_id(app, "new_general", "New General Meeting", true, None::<&str>)?;
    let separator1 = PredefinedMenuItem::separator(app)?;
    let toggle_stealth = MenuItem::with_id(app, "toggle_stealth", "Toggle Stealth", true, None::<&str>)?;
    let show_window = MenuItem::with_id(app, "show_window", "Show Window  ⌘⇧H", true, None::<&str>)?;
    let separator2 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit VoiceNote", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &new_interview,
            &new_general,
            &separator1,
            &toggle_stealth,
            &show_window,
            &separator2,
            &quit,
        ],
    )?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().cloned().unwrap())
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "new_interview" => {
                show_main_window(app);
                let _ = app.emit("menu-new-meeting", "interview");
            }
            "new_general" => {
                show_main_window(app);
                let _ = app.emit("menu-new-meeting", "general");
            }
            "toggle_stealth" => {
                let on = !crate::stealth::is_stealth_on();
                crate::stealth::set_stealth(on);
                if let Some(w) = app.get_webview_window("main") {
                    let _ = crate::stealth::apply_to_window(&w);
                }
                let _ = app.emit("stealth-changed", on);
            }
            "show_window" => show_main_window(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    Ok(())
}

fn show_main_window(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.set_focus();
    }
}
