# Confide Week 2 — Stealth Mode + Window Chrome

> **For agentic workers:** REQUIRED SUB-SKILL: Use gecc-dev:subagent-driven-development (recommended) or gecc-dev:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 录音中切到 Stealth → Zoom 共享屏幕给同事 → 同事看不到 Confide 浮层；进程名显示 "VoiceNote"；不出现在 Dock / Cmd+Tab。

**Domain:** general

**Architecture:**
- Tauri 窗口配置改成 frameless + always-on-top + 320x500
- 加 NSWindowSharingType.None（核心 stealth）
- LSUIElement = true（Dock 隐藏）
- CFBundleName = "VoiceNote"（进程伪装）
- 全局快捷键 4 组（toggle / panic / 透明度）
- 菜单栏图标作为唯一打开入口

**Tech Stack:** Tauri 2 plugin-global-shortcut、objc2-app-kit、tauri 内置 menubar 支持

**Spec reference:** `docs/specs/2026-05-09-overseas-meeting-copilot-design.md` Section 5

**Prerequisite:** Week 1 完成；ScreenCaptureKit 录音 + ASR 链路工作

---

## File Structure

```
src-tauri/
├── Cargo.toml                              [Modify] 加 tauri-plugin-global-shortcut + objc2-app-kit
├── tauri.conf.json                         [Modify] 窗口尺寸、frameless、productName
├── Info.plist                              [Modify] LSUIElement、CFBundleName=VoiceNote
├── capabilities/default.json               [Modify] 加 globalShortcut 权限
└── src/
    ├── stealth/                            [Create dir]
    │   ├── mod.rs                          [Create] StealthState 全局开关
    │   └── window.rs                       [Create] NSWindowSharingType 调用
    ├── shortcuts.rs                        [Create] 全局快捷键注册
    ├── tray.rs                             [Create] 菜单栏图标 + 弹出菜单
    ├── commands.rs                         [Modify] 加 stealth 相关 command
    ├── main.rs                             [Modify] 注册 plugin / shortcuts / tray
    └── lib.rs                              [Modify] 加 pub mod stealth + shortcuts + tray

src/
├── components/
│   └── stealth/
│       └── StealthIndicator.tsx            [Create] 录音中显示 stealth 状态徽章
├── lib/tauri.ts                            [Modify] 加 stealth wrapper
└── App.tsx                                 [Modify] 监听 stealth toggle event
```

---

### Task 1: 加依赖到 Cargo.toml

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: 加 tauri 插件 + objc2 binding**

在 `[dependencies]` 段加：

```toml
# Week 2: Stealth mode
tauri-plugin-global-shortcut = "2"
objc2 = "0.5"
objc2-app-kit = { version = "0.2", features = ["NSWindow"] }
```

- [ ] **Step 2: 验证编译**

Run:
```bash
cd /Users/bytedance/meeting-assistant
cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | tail -10
```

Expected: 通过。

---

### Task 2: 修改 tauri.conf.json — frameless + 尺寸 + productName

**Files:**
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: 替换文件内容**

```json
{
  "productName": "VoiceNote",
  "version": "0.1.0",
  "identifier": "app.voicenote.confide",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:1420",
    "beforeDevCommand": "pnpm run dev",
    "beforeBuildCommand": "pnpm run build"
  },
  "bundle": {
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/256x256.png",
      "icons/icon.icns",
      "icons/icon.png"
    ],
    "macOS": {
      "infoPlist": "Info.plist",
      "minimumSystemVersion": "13.0"
    }
  },
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "VoiceNote",
        "width": 320,
        "height": 500,
        "resizable": true,
        "alwaysOnTop": true,
        "decorations": false,
        "transparent": true,
        "shadow": true,
        "visible": false
      }
    ],
    "security": {
      "csp": null
    }
  },
  "plugins": {
    "globalShortcut": {}
  }
}
```

注意改动：
- `productName: "VoiceNote"`（进程伪装）
- `identifier: "app.voicenote.confide"`（包标识符避免冲突）
- 窗口尺寸 320x500（Section 5.3）
- `decorations: false`（frameless）
- `transparent: true`（半透明背景）
- `visible: false`（启动时隐藏，菜单栏点击才显示）
- `minimumSystemVersion: "13.0"`（macOS 13+ 限定）

- [ ] **Step 2: 验证 tauri build 配置 OK**

Run:
```bash
cd /Users/bytedance/meeting-assistant
cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | tail -5
```

Expected: 通过。

---

### Task 3: 修改 Info.plist — LSUIElement + CFBundleName

**Files:**
- Modify: `src-tauri/Info.plist`

- [ ] **Step 1: 替换为完整 plist**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleName</key>
	<string>VoiceNote</string>
	<key>CFBundleDisplayName</key>
	<string>VoiceNote</string>
	<key>LSUIElement</key>
	<true/>
	<key>NSMicrophoneUsageDescription</key>
	<string>VoiceNote needs microphone access to capture your voice during meetings for transcription.</string>
	<key>NSScreenCaptureDescription</key>
	<string>VoiceNote needs Screen Recording permission to capture meeting audio (Zoom, Meet, Teams). We never see your screen — only system audio. macOS does not have a separate audio-only permission.</string>
</dict>
</plist>
```

注：`LSUIElement = true` 让 app **从 Dock 消失 + 不出现在 Cmd+Tab**。窗口仍然能显示，但用户必须从菜单栏图标打开。

- [ ] **Step 2: 验证 macOS 解析无误**

Run:
```bash
plutil -lint src-tauri/Info.plist
```

Expected: `OK`

---

### Task 4: 修改 capabilities — 加 globalShortcut 权限

**Files:**
- Modify: `src-tauri/capabilities/default.json`

- [ ] **Step 1: 加 permission**

```json
{
  "identifier": "default",
  "description": "Default permissions for VoiceNote",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:window:allow-set-always-on-top",
    "core:window:allow-set-size",
    "core:window:allow-set-title",
    "core:window:allow-show",
    "core:window:allow-hide",
    "core:window:allow-set-focus",
    "dialog:allow-open",
    "global-shortcut:allow-register",
    "global-shortcut:allow-unregister",
    "global-shortcut:allow-is-registered"
  ]
}
```

- [ ] **Step 2: 验证 schema**

Run:
```bash
cat src-tauri/capabilities/default.json | python3 -m json.tool
```

Expected: 输出格式化 JSON 无错误。

---

### Task 5: 创建 stealth/window.rs — NSWindowSharingType 调用

**Files:**
- Create: `src-tauri/src/stealth/window.rs`

- [ ] **Step 1: 写 Objective-C runtime 调用**

```rust
//! NSWindow.sharingType controls whether the window is captured by
//! ScreenCaptureKit / NSScreenshot / Zoom screen sharing.
//!
//! Setting to NSWindowSharingNone (= 0) makes the window invisible to all
//! capture APIs while remaining visible to the user.

use objc2::msg_send;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_app_kit::{NSWindow, NSWindowSharingType};

/// Set the given NSWindow's sharing type.
///
/// # Safety
/// Caller must ensure `ns_window_ptr` points to a valid NSWindow object
/// (typically obtained from Tauri via window.ns_window()).
pub unsafe fn set_window_sharing_type(ns_window_ptr: *mut std::ffi::c_void, hidden: bool) {
    let ns_window: &NSWindow = unsafe { &*(ns_window_ptr as *const NSWindow) };
    let sharing_type = if hidden {
        NSWindowSharingType::None
    } else {
        NSWindowSharingType::ReadOnly
    };
    let _: () = unsafe { msg_send![ns_window, setSharingType: sharing_type] };
}
```

- [ ] **Step 2: 写 stealth/mod.rs**

```rust
pub mod window;

use std::sync::atomic::{AtomicBool, Ordering};

static STEALTH_ON: AtomicBool = AtomicBool::new(false);

pub fn is_stealth_on() -> bool {
    STEALTH_ON.load(Ordering::Acquire)
}

pub fn set_stealth(on: bool) {
    STEALTH_ON.store(on, Ordering::Release);
}

/// Apply current stealth state to the given Tauri window.
pub fn apply_to_window(app_window: &tauri::WebviewWindow) -> tauri::Result<()> {
    let ns_window = app_window.ns_window()?;
    let on = is_stealth_on();
    unsafe {
        window::set_window_sharing_type(ns_window, on);
    }
    Ok(())
}
```

- [ ] **Step 3: 在 lib.rs / main.rs 暴露**

`src-tauri/src/lib.rs` 加：
```rust
pub mod stealth;
```

`src-tauri/src/main.rs` 加：
```rust
mod stealth;
```

- [ ] **Step 4: 编译验证**

Run:
```bash
cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | tail -10
```

Expected: 通过。如果错 `ns_window()` 返回类型不匹配，参考 tauri 2.x docs 的 platform-specific window API：
```rust
#[cfg(target_os = "macos")]
let ns_window = app_window.ns_window()?;
```
返回 `*mut std::ffi::c_void`。

---

### Task 6: 创建 shortcuts.rs — 全局快捷键

**Files:**
- Create: `src-tauri/src/shortcuts.rs`

- [ ] **Step 1: 实现 4 组快捷键**

```rust
//! Global shortcuts for stealth-mode interactions.
//! Registered at app startup; cleaned up automatically on quit.

use tauri::{AppHandle, Manager, Emitter};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState};

/// Register all global shortcuts. Called from main.rs after Tauri builder setup.
pub fn register(app: &AppHandle) -> tauri::Result<()> {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;

    let toggle = Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyH);
    let panic = Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyK);
    let opacity_up = Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::ArrowUp);
    let opacity_down = Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::ArrowDown);

    let app_handle = app.clone();
    app.global_shortcut().on_shortcut(toggle, move |_app, _shortcut, event| {
        if event.state() == ShortcutState::Pressed {
            handle_toggle(&app_handle);
        }
    })?;

    let app_handle = app.clone();
    app.global_shortcut().on_shortcut(panic, move |_app, _shortcut, event| {
        if event.state() == ShortcutState::Pressed {
            handle_panic(&app_handle);
        }
    })?;

    let app_handle = app.clone();
    app.global_shortcut().on_shortcut(opacity_up, move |_app, _shortcut, event| {
        if event.state() == ShortcutState::Pressed {
            let _ = app_handle.emit("opacity-step", 0.1f32);
        }
    })?;

    let app_handle = app.clone();
    app.global_shortcut().on_shortcut(opacity_down, move |_app, _shortcut, event| {
        if event.state() == ShortcutState::Pressed {
            let _ = app_handle.emit("opacity-step", -0.1f32);
        }
    })?;

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
    // 1. Hide window immediately
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
    // 2. Emit event to frontend → frontend calls stop_recording
    let _ = app.emit("panic-stop", ());
    eprintln!("[shortcuts] Panic key triggered: hidden + stop");
}
```

- [ ] **Step 2: 在 lib.rs / main.rs 暴露**

`src-tauri/src/lib.rs`:
```rust
pub mod shortcuts;
```

`src-tauri/src/main.rs`:
```rust
mod shortcuts;
```

- [ ] **Step 3: 编译验证**

Run:
```bash
cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | tail -10
```

Expected: 通过。如果错 `Modifiers::SUPER not found`：在 macOS 上 SUPER = Cmd 键（部分 crate 版本里叫 `META` 或 `CMD`）；查 tauri-plugin-global-shortcut docs。

---

### Task 7: 创建 tray.rs — 菜单栏图标

**Files:**
- Create: `src-tauri/src/tray.rs`

- [ ] **Step 1: 实现菜单栏 + 弹出菜单**

```rust
//! Menu bar icon — only entry point to open the app since LSUIElement=true
//! removes the Dock icon.

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState},
    AppHandle, Manager, Emitter,
};

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let new_interview = MenuItem::with_id(app, "new_interview", "New Interview Meeting", true, None::<&str>)?;
    let new_general = MenuItem::with_id(app, "new_general", "New General Meeting", true, None::<&str>)?;
    let separator1 = PredefinedMenuItem::separator(app)?;
    let toggle_stealth = MenuItem::with_id(app, "toggle_stealth", "Stealth Mode: OFF", true, None::<&str>)?;
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
        .on_menu_event(|app, event| {
            match event.id().as_ref() {
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
            }
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
```

- [ ] **Step 2: 在 lib.rs / main.rs 暴露**

`src-tauri/src/lib.rs`: `pub mod tray;`
`src-tauri/src/main.rs`: `mod tray;`

- [ ] **Step 3: 编译验证**

Run:
```bash
cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | tail -10
```

Expected: 通过。如果 `TrayIconBuilder` API 不同：参考 tauri 2.x docs 的 system-tray 章节，API 在 2.x 各版本可能有 breaking change。

---

### Task 8: 修改 main.rs — 注册 plugin / shortcuts / tray

**Files:**
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: 加 plugin + setup hook**

把整个 `main()` 函数替换为：

```rust
fn main() {
    env_logger::init();

    let _ = crate::advisor::templates::ensure_default_templates(std::path::Path::new("../templates"));

    let recording_state: SharedRecordingState =
        Arc::new(Mutex::new(commands::RecordingState::new()));

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(recording_state)
        .setup(|app| {
            // Stealth tray icon
            tray::setup(app.handle())?;
            // Global shortcuts
            shortcuts::register(app.handle())?;
            // Apply initial stealth state to main window
            if let Some(w) = app.get_webview_window("main") {
                let _ = crate::stealth::apply_to_window(&w);
                let _ = w.show();  // 启动时显示一次
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
            commands::save_template,
            commands::delete_template,
            commands::get_config,
            commands::save_app_config,
            commands::load_document,
            commands::load_reference_doc,
            commands::clear_reference_doc,
            commands::set_active_template,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 2: 编译验证**

Run:
```bash
cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | tail -10
```

Expected: 错 `set_stealth_mode not found`、`is_stealth_on not found`——预期，Task 9 加。

---

### Task 9: 加 stealth 相关 Tauri commands

**Files:**
- Modify: `src-tauri/src/commands.rs`（文件末尾追加）
- Modify: `src/lib/tauri.ts`（前端 wrapper）

- [ ] **Step 1: 在 commands.rs 末尾追加**

```rust
// --- Stealth ---

#[command]
pub fn set_stealth_mode(on: bool, window: tauri::Window) -> Result<(), String> {
    crate::stealth::set_stealth(on);
    if let Some(w) = window.get_webview_window("main") {
        crate::stealth::apply_to_window(&w).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[command]
pub fn is_stealth_on() -> Result<bool, String> {
    Ok(crate::stealth::is_stealth_on())
}
```

注意 import：在文件顶部已有 `use tauri::Manager;` 否则加上。

- [ ] **Step 2: 在 src/lib/tauri.ts 加 wrapper**

```typescript
export const setStealthMode = (on: boolean) =>
  invoke<void>("set_stealth_mode", { on });

export const isStealthOn = () => invoke<boolean>("is_stealth_on");
```

加 event listener wrapper：

```typescript
export const onStealthChanged = (
  handler: (on: boolean) => void,
): Promise<UnlistenFn> =>
  listen<boolean>("stealth-changed", (e) => handler(e.payload));

export const onPanicStop = (handler: () => void): Promise<UnlistenFn> =>
  listen("panic-stop", () => handler());

export const onOpacityStep = (
  handler: (delta: number) => void,
): Promise<UnlistenFn> =>
  listen<number>("opacity-step", (e) => handler(e.payload));

export const onMenuNewMeeting = (
  handler: (kind: "interview" | "general") => void,
): Promise<UnlistenFn> =>
  listen<string>("menu-new-meeting", (e) =>
    handler(e.payload as "interview" | "general"),
  );
```

- [ ] **Step 3: 编译验证**

Run:
```bash
cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | tail -5
pnpm typecheck 2>&1 | tail -5
```

Expected: 都通过。

---

### Task 10: 创建 StealthIndicator 组件 + 接 panic / opacity event

**Files:**
- Create: `src/components/stealth/StealthIndicator.tsx`
- Modify: `src/App.tsx`

- [ ] **Step 1: 写 StealthIndicator 组件**

```tsx
import { useEffect, useState } from "react";
import {
  isStealthOn,
  onStealthChanged,
  setStealthMode,
} from "../../lib/tauri";

export function StealthIndicator() {
  const [on, setOn] = useState(false);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void (async () => {
      try {
        setOn(await isStealthOn());
      } catch {}
      unlisten = await onStealthChanged((v) => setOn(v));
    })();
    return () => unlisten?.();
  }, []);

  return (
    <button
      onClick={() => void setStealthMode(!on)}
      className={`text-xs px-2 py-1 rounded ${
        on
          ? "bg-red-900/40 text-red-300 border border-red-700/50"
          : "bg-gray-800/40 text-gray-400 border border-gray-700/50"
      }`}
      title="⌘⇧H toggle window · ⌘⇧K panic stop"
    >
      {on ? "🛡️ Stealth ON" : "Stealth OFF"}
    </button>
  );
}
```

- [ ] **Step 2: 在 App.tsx 接 panic / opacity / menu-new-meeting**

打开 `src/App.tsx`，加：

```tsx
import { useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  onPanicStop,
  onOpacityStep,
  onMenuNewMeeting,
  stopRecording,
} from "./lib/tauri";

// ...

export default function App() {
  // ... existing state

  // Stealth event handlers
  useEffect(() => {
    let unlistens: Array<() => void> = [];
    void (async () => {
      unlistens.push(
        await onPanicStop(async () => {
          try {
            await stopRecording();
          } catch (e) {
            console.error("panic stop failed:", e);
          }
        }),
      );
      unlistens.push(
        await onOpacityStep(async (delta) => {
          const w = getCurrentWindow();
          // Tauri 2 API: there's no direct getOpacity; track in state instead
          // For MVP, just toggle 0.5 and 1.0 for simplicity
          // (full opacity stepping: track opacity in App state, set via setOpacity)
          // Skipping for MVP; v1.0.5 implement.
          console.log("opacity-step:", delta);
        }),
      );
      unlistens.push(
        await onMenuNewMeeting(async (kind) => {
          // Switch to narrow view + auto-select template
          setView("narrow");
          // TODO Week 3: auto-select template by kind ('interview' | 'general')
        }),
      );
    })();
    return () => unlistens.forEach((u) => u());
  }, []);

  // ... rest of return
}
```

注：opacity stepping MVP 简化只 log，v1.0.5 实现完整。menu-new-meeting 在 Week 3 模板系统就位后接到模板选择。

- [ ] **Step 3: 编译验证**

Run:
```bash
pnpm typecheck 2>&1 | tail -5
```

Expected: 通过。

---

### Task 11: 端到端验证 — 共享屏幕给同事看不到

这是 Week 2 的 acceptance test。

- [ ] **Step 1: pnpm tauri dev 启动**

Run:
```bash
cd /Users/bytedance/meeting-assistant
OPENAI_API_KEY="<key>" pnpm tauri dev
```

第一次启动 Rust 编译 5-10 分钟。

- [ ] **Step 2: 验证 Dock 隐藏**

观察：
- Dock 里**没有** VoiceNote 图标 ✅
- macOS 顶部菜单栏右边出现 VoiceNote 图标 ✅
- Cmd+Tab 切换列表里**没有** VoiceNote ✅

如果 Dock 里仍出现：检查 Info.plist 的 LSUIElement，并确认 dev 模式下 reload。**LSUIElement 改动可能需要完整重启 cargo build**（不只是 hot reload）。

- [ ] **Step 3: 验证菜单栏菜单工作**

点菜单栏图标 → 弹出菜单显示：
```
New Interview Meeting
New General Meeting
─────────────
Stealth Mode: OFF
Show Window  ⌘⇧H
─────────────
Quit VoiceNote
```

点 "Stealth Mode: OFF" → 应变成 ON（但这版菜单 label 不会自动 update，是已知小问题；StealthIndicator 在 UI 里会更新）。

- [ ] **Step 4: 验证 ⌘⇧H 切换可见性**

按 ⌘⇧H：窗口显示 ↔ 隐藏。如果不工作：
- 检查 macOS 设置 > 隐私 > 输入监控（部分系统会要求授权全局快捷键）
- 看 cargo run 日志有没有 `[shortcuts] Registered`

- [ ] **Step 5: 验证 ⌘⇧K panic 工作**

先开始录音（如果 Week 1 跑通了），然后按 ⌘⇧K：
- 窗口立即隐藏 ✅
- 录音停止（看 Tauri terminal 应有 `panic-stop` event 日志） ✅

- [ ] **Step 6: 关键验证 — Zoom 共享屏幕**

打开 Zoom，开 personal meeting（或 join 一个朋友），共享你的整个屏幕。

**让朋友截图发给你**，看他屏幕里：
- ✅ 看不到 VoiceNote 浮层（NSWindowSharingType.None 工作）
- ✅ 看不到菜单栏的 VoiceNote 图标（菜单栏图标不在 sharing window 里默认就抓不到）

如果朋友能看到 VoiceNote：检查 stealth 是否 ON、`apply_to_window` 是否在窗口创建后调用。

- [ ] **Step 7: 验证 macOS screencapture 命令也抓不到**

Run（在 VoiceNote 显示状态下）:
```bash
screencapture -i /tmp/test_capture.png
```

选区圈住 VoiceNote 窗口位置 → 看截图。Expected: 截图里 VoiceNote 区域显示**底层桌面**（VoiceNote 不可见）。

- [ ] **Step 8: 验证进程名是 "VoiceNote"**

打开 Activity Monitor → 搜 "VoiceNote"。Expected: 进程显示 "VoiceNote"（不是 "meeting-assistant" 也不是 "Confide"）。

如果显示其他名字：检查 `tauri.conf.json` 的 `productName` 和 `Info.plist` 的 `CFBundleName` 都是 "VoiceNote"。

- [ ] **Step 9: 标记 Week 2 完成**

在 `decision-log.md` 加：

```
## Week 2 完成
- 日期: <2026-05-XX>
- 验收: ✅ Zoom 共享屏幕朋友看不到 VoiceNote / 4 组快捷键工作 / 进程名 VoiceNote
- 已知问题:
  - 菜单栏 "Stealth Mode: ON/OFF" label 不自动 update（小问题，v1.0.5 修）
  - 透明度调节快捷键 MVP 仅 log，不真改 opacity
- v1.0.5 待办:
  - 透明度真调
  - 菜单 label 动态更新
  - 用户可自定义快捷键
```

---

## Week 2 完成标志（Acceptance Criteria）

对应 design Section 9 AC：
- ✅ D1 NSWindowSharingType 在 Zoom/Meet/Teams 共享屏幕时不可见
- ✅ D2 macOS `screencapture` 命令不抓到
- ✅ D3 Activity Monitor 进程显示 "VoiceNote"
- ✅ D4 不出现在 Dock / Cmd+Tab
- ✅ D5 ⌘⇧H toggle / ⌘⇧K panic（⌘⇧↑↓ 透明度 v1.0.5 完整实现）
- ⏳ D6 模板默认 stealth ON/OFF — Week 3 模板系统才能验证

下一步：进 Week 3 — 模板系统重设计 + 简历 RAG。
