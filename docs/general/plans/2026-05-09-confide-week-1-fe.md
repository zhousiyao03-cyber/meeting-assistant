# Confide Week 1 — Audio Pipeline + GPT-Realtime-Whisper

> **For agentic workers:** REQUIRED SUB-SKILL: Use gecc-dev:subagent-driven-development (recommended) or gecc-dev:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** macOS 13+ 用户**不需要装 BlackHole** 即能录到 Zoom / Meet / Teams 的系统音频；GPT-Realtime-Whisper 替代本地 SenseVoice 作为默认 ASR。

**Domain:** general

**Architecture:**
- 把现有 `src-tauri/src/audio/capture.rs`（基于 cpal + BlackHole）拆成 `SystemAudioCapture` trait + `ScreenCaptureKitBackend` 实现
- 新增 `src-tauri/src/asr/` 模块，定义 `AsrProvider` trait + `OpenAiRealtimeWhisperProvider` 实现
- 替换现有 sherpa-onnx 默认调用路径——但保留 `whisper/` 目录代码作为 v1.1 fallback（不删）
- 修改 `commands.rs::start_recording` 让它走新 ASR provider 路径

**Tech Stack:** Rust 2021、`screencapturekit` crate、`tokio-tungstenite`（WebSocket 客户端）、`reqwest` 已有

**Spec reference:** `docs/specs/2026-05-09-overseas-meeting-copilot-design.md` Section 4

**Prerequisite:** Week 0 完成；`docs/general/plans/decision-log.md` 已填 OpenAI key + GPT-Realtime-Whisper model ID 验证

---

## File Structure

新增和修改的文件清单：

```
src-tauri/
├── Cargo.toml                              [Modify] 加 screencapturekit + tokio-tungstenite + futures-util(已有) 依赖
├── Info.plist                              [Modify] 加 NSScreenCaptureDescription 权限
├── capabilities/default.json               [Modify] 不变（screen capture 不需要 Tauri capability）
└── src/
    ├── audio/
    │   ├── mod.rs                          [Modify] 加 mod system_capture; 暴露 trait
    │   ├── system_capture.rs               [Create] SystemAudioCapture trait + create_system_audio_capture()
    │   ├── screen_capture_kit.rs           [Create] ScreenCaptureKitBackend 实现
    │   └── permission.rs                   [Create] Screen Recording 权限检查 / 引导
    ├── asr/                                [Create dir]
    │   ├── mod.rs                          [Create] AsrProvider trait + create_asr_provider()
    │   └── whisper_realtime.rs             [Create] OpenAiRealtimeWhisperProvider (WebSocket client)
    ├── commands.rs                         [Modify] start_recording 改用 system_capture + ASR provider
    ├── main.rs                             [Modify] 加 asr 模块 + 加新 command（check_screen_recording_permission）
    └── lib.rs                              [Modify] 加 pub mod asr
```

不动：`audio/buffer.rs`（SharedBuffer 完全复用）、`audio/capture.rs`（保留 cpal 麦克风路径）、`whisper/`（保留作 v1.1 fallback）

---

### Task 1: 加依赖到 Cargo.toml

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: 在 [dependencies] 段加新 crate**

打开 `src-tauri/Cargo.toml`，在 `[dependencies]` 段末尾（`tar = "0.4"` 之后）插入：

```toml
# Week 1: macOS 13+ system audio capture
screencapturekit = "0.3"
# Week 1: GPT-Realtime-Whisper WebSocket client
tokio-tungstenite = { version = "0.24", features = ["rustls-tls-native-roots"] }
# Week 1: 已有 futures-util，确认存在；若无则加
# futures-util = "0.3"  ← 已存在于现有 Cargo.toml 第 21 行
```

注：`screencapturekit` crate 实际版本号以 Week 0 PoC 验证的为准（写到 decision-log.md 那个）。如果 PoC 用的是 `0.3`，这里就是 `0.3`；如果是 `0.4` 已发布，用最新。

- [ ] **Step 2: 验证依赖能下载**

Run:
```bash
cd /Users/bytedance/meeting-assistant
cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | tail -10
```

Expected: 编译通过或只有 unused import warning。如果错 `failed to resolve: use of undeclared crate or module`：是因为还没在源代码里 use，OK 忽略。如果错 crate 不存在：去 crates.io 确认 crate 名。

---

### Task 2: 加 Screen Recording 权限到 Info.plist

**Files:**
- Modify: `src-tauri/Info.plist`

- [ ] **Step 1: 加 NSScreenCaptureDescription**

把 `src-tauri/Info.plist` 内容替换为：

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>NSMicrophoneUsageDescription</key>
	<string>Confide needs microphone access to capture your voice during meetings for transcription.</string>
	<key>NSScreenCaptureDescription</key>
	<string>Confide needs Screen Recording permission to capture meeting audio (Zoom, Meet, Teams). We never see your screen — only system audio. macOS does not have a separate audio-only permission.</string>
</dict>
</plist>
```

注：MVP 阶段 product 名 codename "Confide"，bundle 名是 "VoiceNote"（design Section 5.2.3 stealth）。这里描述用 "Confide" 给用户看；CFBundleName 改成 "VoiceNote" 是 Week 2 Task 的事，本周不动。

- [ ] **Step 2: 不需要其他权限**

确认 `LSUIElement` 不在这里加（Week 2 stealth 加）。Tauri capabilities 也不动（`capabilities/default.json` 不需要变）。

---

### Task 3: 创建 audio/permission.rs — Screen Recording 权限检查

**Files:**
- Create: `src-tauri/src/audio/permission.rs`

- [ ] **Step 1: 实现权限检查函数**

写 `src-tauri/src/audio/permission.rs`:

```rust
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub enum PermissionStatus {
    Granted,
    Denied,
    NotDetermined,
}

/// Check whether the app has been granted Screen Recording permission.
///
/// macOS gates this via TCC.db. The cleanest way to detect from Rust is to
/// attempt SCShareableContent::current() and check whether it returns a
/// non-empty list. If permission is missing, macOS returns either an error
/// or an empty list (depending on macOS version).
pub fn check_screen_recording_permission() -> PermissionStatus {
    use screencapturekit::shareable_content::SCShareableContent;

    match SCShareableContent::get() {
        Ok(content) => {
            // If we can list at least one display + the running app's own window,
            // permission is granted. If the system silently returns an empty list,
            // permission is denied.
            if content.displays().is_empty() {
                PermissionStatus::Denied
            } else {
                PermissionStatus::Granted
            }
        }
        Err(_) => PermissionStatus::Denied,
    }
}

/// Open System Settings → Privacy & Security → Screen Recording.
/// User must drag Confide.app into the list (or check it) and restart the app.
pub fn open_settings_screen_recording() -> std::io::Result<()> {
    std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture")
        .spawn()?;
    Ok(())
}

/// Check macOS version. Returns Some((major, minor)) or None on non-macOS.
pub fn macos_version() -> Option<(u32, u32)> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let output = Command::new("sw_vers")
            .arg("-productVersion")
            .output()
            .ok()?;
        let s = String::from_utf8(output.stdout).ok()?;
        let parts: Vec<&str> = s.trim().split('.').collect();
        let major: u32 = parts.get(0)?.parse().ok()?;
        let minor: u32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        Some((major, minor))
    }
    #[cfg(not(target_os = "macos"))]
    {
        None
    }
}

pub fn macos_version_at_least(major: u32, minor: u32) -> bool {
    match macos_version() {
        Some((m, n)) => m > major || (m == major && n >= minor),
        None => false,
    }
}
```

- [ ] **Step 2: 在 audio/mod.rs 暴露**

打开 `src-tauri/src/audio/mod.rs`，**当前文件**应该是：

```rust
pub mod buffer;
pub mod capture;
```

改为：

```rust
pub mod buffer;
pub mod capture;
pub mod permission;
pub mod system_capture;
pub mod screen_capture_kit;
```

`system_capture` 和 `screen_capture_kit` 这两个文件还没创建，下面 Task 4-5 会创。**先这样写以保证 mod 树形成**。

- [ ] **Step 3: 临时验证编译（应失败因为缺文件）**

Run:
```bash
cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | tail -10
```

Expected: 错 "file not found for module `system_capture`"——这是预期。Task 4-5 写完后会通过。

---

### Task 4: 创建 audio/system_capture.rs — trait 抽象

**Files:**
- Create: `src-tauri/src/audio/system_capture.rs`

- [ ] **Step 1: 定义 trait + factory**

写 `src-tauri/src/audio/system_capture.rs`:

```rust
use anyhow::Result;
use std::sync::{Arc, Mutex};

use super::buffer::SharedBuffer;
use super::permission::macos_version_at_least;

/// Backend-agnostic system audio capture.
///
/// Implementations push 16kHz mono f32 PCM into `buffer` continuously
/// once `start()` returns Ok.
pub trait SystemAudioCapture: Send {
    fn start(&mut self) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
    fn is_running(&self) -> bool;
    fn backend_name(&self) -> &'static str;
}

/// Filter for which apps' audio gets captured. Default in MVP is hard-coded.
#[derive(Clone, Debug)]
pub struct AppFilter {
    pub included_bundle_ids: Vec<String>,
    pub excluded_bundle_ids: Vec<String>,
}

impl Default for AppFilter {
    fn default() -> Self {
        Self {
            // Confide MVP defaults — see design Section 4.6
            included_bundle_ids: vec![
                "us.zoom.xos".into(),
                "com.microsoft.teams2".into(),
                "com.google.Chrome".into(),
                "com.apple.Safari".into(),
                "com.hnc.Discord".into(),
                "com.apple.FaceTime".into(),
            ],
            excluded_bundle_ids: vec![
                "com.spotify.client".into(),
                "com.apple.Music".into(),
                "com.apple.Notes".into(),
            ],
        }
    }
}

/// Pick the best system-audio backend for this OS / version.
pub fn create_system_audio_capture(
    buffer: SharedBuffer,
    filter: AppFilter,
) -> Result<Box<dyn SystemAudioCapture>> {
    #[cfg(target_os = "macos")]
    {
        if macos_version_at_least(13, 0) {
            return Ok(Box::new(
                super::screen_capture_kit::ScreenCaptureKitBackend::new(buffer, filter)?
            ));
        }
        return Err(anyhow::anyhow!(
            "Confide requires macOS 13.0 or later. Your macOS version is too old. macOS 12 support is planned for v1.1."
        ));
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (buffer, filter);
        return Err(anyhow::anyhow!(
            "Confide currently supports macOS only. Windows support is planned for v1.2."
        ));
    }
}
```

- [ ] **Step 2: 验证编译**

Run:
```bash
cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | tail -15
```

Expected: 仍然错 `screen_capture_kit not found`（因为 Task 5 还没写），但 system_capture.rs 本身的语法应该 OK。

---

### Task 5: 创建 audio/screen_capture_kit.rs — macOS 13+ 实现

**Files:**
- Create: `src-tauri/src/audio/screen_capture_kit.rs`

注：`screencapturekit` crate 的具体 API 可能与下面代码略有差异。**第一次实现以 Week 0 PoC 跑通的代码为准**——把 PoC 的核心逻辑搬过来，按下面 trait 包装。

- [ ] **Step 1: 实现 ScreenCaptureKitBackend**

写 `src-tauri/src/audio/screen_capture_kit.rs`:

```rust
use anyhow::{anyhow, Result};
use std::sync::{Arc, Mutex};

use super::buffer::SharedBuffer;
use super::system_capture::{AppFilter, SystemAudioCapture};

/// macOS 13+ system audio capture via ScreenCaptureKit.
pub struct ScreenCaptureKitBackend {
    buffer: SharedBuffer,
    filter: AppFilter,
    stream: Option<screencapturekit::stream::SCStream>,
    running: Arc<Mutex<bool>>,
}

impl ScreenCaptureKitBackend {
    pub fn new(buffer: SharedBuffer, filter: AppFilter) -> Result<Self> {
        Ok(Self {
            buffer,
            filter,
            stream: None,
            running: Arc::new(Mutex::new(false)),
        })
    }

    fn build_stream(&self) -> Result<screencapturekit::stream::SCStream> {
        use screencapturekit::shareable_content::SCShareableContent;
        use screencapturekit::stream::configuration::SCStreamConfiguration;
        use screencapturekit::stream::content_filter::SCContentFilter;
        use screencapturekit::stream::output_trait::SCStreamOutputTrait;
        use screencapturekit::stream::output_type::SCStreamOutputType;
        use screencapturekit::stream::SCStream;

        let content = SCShareableContent::get()
            .map_err(|e| anyhow!("get shareable content: {:?} (Screen Recording permission missing?)", e))?;

        let displays = content.displays();
        let display = displays.first().ok_or_else(|| anyhow!("No display available"))?;

        // MVP: capture the whole display's audio mix; app whitelist is enforced
        // post-hoc by VAD + transcript filtering. SCContentFilter exclusion API
        // for specific apps requires SCRunningApplication objects — wire this
        // properly in v1.0.5 once we expose user-editable whitelist.
        let cf = SCContentFilter::new().with_display_excluding_windows(display, &[]);

        let config = SCStreamConfiguration::new()
            .set_captures_audio(true)
            .map_err(|e| anyhow!("set captures_audio: {:?}", e))?
            .set_excludes_current_process_audio(true)
            .map_err(|e| anyhow!("exclude self audio: {:?}", e))?;

        let output = SckOutput {
            buffer: self.buffer.clone(),
            running: self.running.clone(),
        };

        let mut stream = SCStream::new(&cf, &config);
        stream.add_output_handler(output, SCStreamOutputType::Audio);

        Ok(stream)
    }
}

impl SystemAudioCapture for ScreenCaptureKitBackend {
    fn start(&mut self) -> Result<()> {
        let mut stream = self.build_stream()?;
        stream
            .start_capture()
            .map_err(|e| anyhow!("start_capture: {:?}", e))?;
        *self.running.lock().unwrap() = true;
        self.stream = Some(stream);
        eprintln!("[sckit] Capture started");
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        *self.running.lock().unwrap() = false;
        if let Some(mut s) = self.stream.take() {
            s.stop_capture().map_err(|e| anyhow!("stop_capture: {:?}", e))?;
        }
        eprintln!("[sckit] Capture stopped");
        Ok(())
    }

    fn is_running(&self) -> bool {
        *self.running.lock().unwrap()
    }

    fn backend_name(&self) -> &'static str {
        "ScreenCaptureKit"
    }
}

/// SCStreamOutputTrait handler — receives every audio sample buffer from SCKit
/// and pushes resampled 16kHz mono f32 into the shared buffer.
struct SckOutput {
    buffer: SharedBuffer,
    running: Arc<Mutex<bool>>,
}

impl screencapturekit::stream::output_trait::SCStreamOutputTrait for SckOutput {
    fn did_output_sample_buffer(
        &self,
        sample_buffer: screencapturekit::output::CMSampleBuffer,
        of_type: screencapturekit::stream::output_type::SCStreamOutputType,
    ) {
        if of_type != screencapturekit::stream::output_type::SCStreamOutputType::Audio {
            return;
        }
        if !*self.running.lock().unwrap() {
            return;
        }

        // Extract PCM. ScreenCaptureKit gives 48kHz interleaved stereo f32 by default.
        // Convert to 16kHz mono f32 to match the buffer format.
        match extract_pcm(&sample_buffer) {
            Ok((pcm_48k_stereo, source_rate, channels)) => {
                let mono_16k = downsample_to_16k_mono(&pcm_48k_stereo, source_rate, channels);
                if let Ok(mut buf) = self.buffer.lock() {
                    buf.push(&mono_16k);
                }
            }
            Err(e) => {
                eprintln!("[sckit] extract_pcm failed: {}", e);
            }
        }
    }
}

/// Extract f32 PCM from a CMSampleBuffer + return (pcm, sample_rate, channels).
///
/// ScreenCaptureKit's exact API for getting PCM bytes from a CMSampleBuffer
/// changes between crate versions. The reference implementation lives in your
/// Week 0 PoC; copy that code into this function.
fn extract_pcm(
    sample_buffer: &screencapturekit::output::CMSampleBuffer,
) -> anyhow::Result<(Vec<f32>, f64, usize)> {
    // Implementation note (Task 6 fills this in based on Week 0 PoC):
    //
    // ScreenCaptureKit delivers audio as CMSampleBuffer. The crate may expose this
    // via different methods depending on version. Reference the docs.rs page of the
    // exact version locked in Cargo.toml, then implement using ONE of these patterns:
    //
    // Pattern A (if crate exposes get_audio_buffer_list):
    //   let abl = sample_buffer.get_audio_buffer_list().ok_or(anyhow!("no abl"))?;
    //   let buffer = abl.buffers().get(0).ok_or(anyhow!("no buffer"))?;
    //   let ptr = buffer.data() as *const f32;
    //   let len = buffer.data_byte_size() as usize / std::mem::size_of::<f32>();
    //   let pcm: Vec<f32> = unsafe { std::slice::from_raw_parts(ptr, len) }.to_vec();
    //   let format = sample_buffer.get_format_description()?.audio_stream_basic_description();
    //   Ok((pcm, format.mSampleRate, format.mChannelsPerFrame as usize))
    //
    // Pattern B (if crate exposes get_av_audio_pcm_buffer):
    //   let pcm_buffer = sample_buffer.get_av_audio_pcm_buffer()?;
    //   let f32_data = pcm_buffer.float_channel_data();
    //   ...
    //
    // Use whichever pattern matches the crate. Validate via Step 3 manual test
    // (5 second capture should yield ~480000 samples at 48kHz stereo).
    Err(anyhow!("extract_pcm: implement per Pattern A or B above using your locked screencapturekit crate version"))
}

/// Downsample interleaved 48kHz stereo to 16kHz mono using simple averaging.
/// Reuses the same algorithm as audio/capture.rs:resample_mono.
fn downsample_to_16k_mono(data: &[f32], source_rate: f64, channels: usize) -> Vec<f32> {
    // 1. interleaved → mono (average across channels)
    let mono: Vec<f32> = data
        .chunks(channels)
        .map(|frame| frame.iter().sum::<f32>() / channels as f32)
        .collect();

    // 2. resample 48k → 16k (ratio 1/3)
    let target_rate: f64 = 16000.0;
    if (source_rate - target_rate).abs() < 1.0 {
        return mono;
    }
    let ratio = target_rate / source_rate;
    let resampled_len = (mono.len() as f64 * ratio) as usize;
    let mut resampled = Vec::with_capacity(resampled_len);
    for i in 0..resampled_len {
        let src_idx = i as f64 / ratio;
        let idx = src_idx as usize;
        let frac = src_idx - idx as f64;
        let s = if idx + 1 < mono.len() {
            mono[idx] * (1.0 - frac as f32) + mono[idx + 1] * frac as f32
        } else if idx < mono.len() {
            mono[idx]
        } else {
            0.0
        };
        resampled.push(s);
    }
    resampled
}
```

注：**`extract_pcm` 函数体在这里给了 Pattern A / B 两种参考写法**——是整个 Week 1 唯一需要"按 crate 版本对齐"的地方。Task 6 会基于 Week 0 PoC 选定 pattern 填入完整实现并通过 manual test 验证。

- [ ] **Step 2: 编译检查**

Run:
```bash
cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | tail -20
```

Expected: 编译通过（带 unused 警告 OK）。如果错 `extract_pcm not implemented` 这是 runtime error 不影响编译，OK。

---

### Task 6: 实现 extract_pcm（基于 Week 0 PoC）

**Files:**
- Modify: `src-tauri/src/audio/screen_capture_kit.rs`（替换 `extract_pcm` 函数体）

- [ ] **Step 1: 把 Week 0 PoC 的 sample 处理代码搬过来**

打开你 Week 0 写的 `/tmp/sckit-poc/src/main.rs`。如果 PoC 只数了 sample buffer 数量没真正读 PCM，**先去 PoC 里把读 PCM 的代码写出来跑通**——5 秒录音应该能读出 48000 × 5 × 2 ≈ 480000 个 f32 sample。

把 working PCM 提取代码替换 `screen_capture_kit.rs::extract_pcm` 函数体。返回 `(Vec<f32>, 48000.0, 2)`（典型 macOS 系统音频参数）。

参考 `screencapturekit` crate 的 docs.rs 页面看 `CMSampleBuffer` 的 audio 接口：
- 可能是 `sample_buffer.get_audio_buffer_list()` → 得 `AudioBufferList`
- 可能是 `sample_buffer.get_av_audio_pcm_buffer()` → 得 `AVAudioPCMBuffer`
- 取决于 crate 版本

- [ ] **Step 2: 验证编译 + 单元功能**

Run:
```bash
cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | tail -5
```

Expected: 编译通过。

- [ ] **Step 3: 写一个 manual test 跑 5 秒系统音频捕获**

新建临时测试文件 `/Users/bytedance/meeting-assistant/src-tauri/examples/sckit_test.rs`:

```rust
use meeting_assistant::audio::buffer::create_shared_buffer;
use meeting_assistant::audio::system_capture::{create_system_audio_capture, AppFilter};
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    let buffer = create_shared_buffer(2, 16000);
    let mut backend = create_system_audio_capture(buffer.clone(), AppFilter::default())?;

    eprintln!("[test] Backend: {}", backend.backend_name());
    eprintln!("[test] Starting 5 second capture. Play any audio (Spotify, YouTube, etc)...");
    backend.start()?;

    std::thread::sleep(Duration::from_secs(5));
    backend.stop()?;

    let pcm_count = buffer.lock().unwrap().len();
    eprintln!("[test] PCM samples buffered: {}", pcm_count);
    eprintln!("[test] Expected ~80000 (16kHz × 5s) — you got: {}", pcm_count);
    if pcm_count < 16000 {
        eprintln!("[test] ⚠️ Less than 1 second of audio — likely permission or extract_pcm bug");
        std::process::exit(1);
    }
    eprintln!("[test] ✅ Audio capture working");
    Ok(())
}
```

需要 `src-tauri/Cargo.toml` 加：
```toml
[[example]]
name = "sckit_test"
path = "examples/sckit_test.rs"
```

但 `meeting-assistant` 当前是 binary crate（Cargo.toml 里 `[package]` 没 `lib.rs` 暴露为 lib）。**先验证**：

Run:
```bash
ls src-tauri/src/lib.rs
cat src-tauri/src/lib.rs
```

如果 lib.rs 已经 `pub mod audio` 了（按你 demo 现有的它确实是），那 example 可用。如果没有则改 Cargo.toml 加 `[lib]` 段：

```toml
[lib]
name = "meeting_assistant"
path = "src/lib.rs"
```

然后跑：

```bash
cd src-tauri
cargo run --example sckit_test 2>&1
```

Expected:
```
[sckit] Capture started
[test] PCM samples buffered: 80000
[test] ✅ Audio capture working
```

如果首次跑弹"Confide 想录制屏幕"权限请求 → 同意 → 重启 cargo run。

如果 `< 16000` samples：要么权限问题、要么 `extract_pcm` 有 bug。**这个 task block Week 1 推进——必须解决再继续**。

---

### Task 7: 创建 asr/mod.rs — AsrProvider trait

**Files:**
- Create: `src-tauri/src/asr/mod.rs`

- [ ] **Step 1: 定义 trait 和工厂**

写 `src-tauri/src/asr/mod.rs`:

```rust
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

pub mod whisper_realtime;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AsrConfig {
    pub provider: String,        // "openai-realtime-whisper" | (v1.1: "local-sensevoice")
    pub openai_api_key: String,  // Used by openai-realtime-whisper
    pub openai_model: String,    // e.g. "gpt-realtime-whisper"
    pub language_hint: String,   // "auto" | "zh" | "en" | ...
}

impl Default for AsrConfig {
    fn default() -> Self {
        Self {
            provider: "openai-realtime-whisper".into(),
            openai_api_key: String::new(),
            openai_model: "gpt-realtime-whisper".into(),
            language_hint: "auto".into(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct TranscriptChunk {
    pub text: String,
    pub speaker: String,           // "me" | "other"
    pub offset_secs: f64,
    pub is_final: bool,
}

/// Streaming ASR provider. Caller pushes 16kHz mono f32 PCM via `send_audio()`,
/// receives transcript chunks via the callback registered at construction time.
#[async_trait::async_trait]
pub trait AsrProvider: Send {
    async fn start(&mut self) -> Result<()>;
    async fn send_audio(&mut self, pcm_16k_mono: &[f32], speaker: &str) -> Result<()>;
    async fn stop(&mut self) -> Result<()>;
    fn provider_name(&self) -> &'static str;
}

pub type TranscriptCallback = Arc<dyn Fn(TranscriptChunk) + Send + Sync>;

pub fn create_asr_provider(
    config: &AsrConfig,
    on_transcript: TranscriptCallback,
) -> Result<Box<dyn AsrProvider>> {
    match config.provider.as_str() {
        "openai-realtime-whisper" => Ok(Box::new(
            whisper_realtime::OpenAiRealtimeWhisperProvider::new(config, on_transcript)?
        )),
        other => Err(anyhow::anyhow!(
            "Unknown ASR provider: '{}'. MVP supports only 'openai-realtime-whisper'.",
            other
        )),
    }
}
```

- [ ] **Step 2: 加 async_trait 依赖**

`screencapturekit` 之外，AsrProvider trait 用了 `async_trait`。打开 `src-tauri/Cargo.toml` 加：

```toml
async-trait = "0.1"
```

（或者去掉 async_trait 改用 `Pin<Box<dyn Future>>` 返回，但 async_trait 简单很多。）

- [ ] **Step 3: 在 lib.rs / main.rs 暴露 mod**

打开 `src-tauri/src/lib.rs`，加 `pub mod asr;`：

```rust
pub mod audio;
pub mod advisor;
pub mod commands;
pub mod documents;
pub mod storage;
pub mod transcript;
pub mod whisper;
pub mod asr;  // ← 新增
```

打开 `src-tauri/src/main.rs`，加 `mod asr;`：

```rust
mod audio;
mod advisor;
mod commands;
mod documents;
mod storage;
mod transcript;
mod whisper;
mod asr;  // ← 新增
```

- [ ] **Step 4: 验证编译（仍会缺 whisper_realtime 文件）**

Run:
```bash
cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | tail -10
```

Expected: 错 `file not found for module whisper_realtime` —— 预期。Task 8 写完会通过。

---

### Task 8: 实现 asr/whisper_realtime.rs — OpenAI Realtime API WebSocket 客户端

**Files:**
- Create: `src-tauri/src/asr/whisper_realtime.rs`

- [ ] **Step 1: 实现 OpenAiRealtimeWhisperProvider**

写 `src-tauri/src/asr/whisper_realtime.rs`:

```rust
use anyhow::{anyhow, Result};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::protocol::Message;

use super::{AsrConfig, AsrProvider, TranscriptCallback, TranscriptChunk};

/// OpenAI Realtime API streaming transcription client.
///
/// Protocol reference: https://platform.openai.com/docs/guides/realtime-transcription
/// Endpoint: wss://api.openai.com/v1/realtime?model=gpt-realtime-whisper
///
/// Frame schema (client→server):
/// {
///   "type": "input_audio_buffer.append",
///   "audio": "<base64 PCM16 LE 16kHz mono>"
/// }
///
/// Frame schema (server→client):
/// {
///   "type": "conversation.item.input_audio_transcription.delta",
///   "delta": "<text>"
/// }
/// or
/// {
///   "type": "conversation.item.input_audio_transcription.completed",
///   "transcript": "<final text>"
/// }
pub struct OpenAiRealtimeWhisperProvider {
    config: AsrConfig,
    on_transcript: TranscriptCallback,
    sender: Option<Arc<TokioMutex<futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
        Message,
    >>>>,
    reader_handle: Option<JoinHandle<()>>,
    start_time: Option<std::time::Instant>,
}

impl OpenAiRealtimeWhisperProvider {
    pub fn new(config: &AsrConfig, on_transcript: TranscriptCallback) -> Result<Self> {
        if config.openai_api_key.is_empty() {
            return Err(anyhow!("OpenAI API key not configured"));
        }
        Ok(Self {
            config: config.clone(),
            on_transcript,
            sender: None,
            reader_handle: None,
            start_time: None,
        })
    }
}

#[async_trait::async_trait]
impl AsrProvider for OpenAiRealtimeWhisperProvider {
    async fn start(&mut self) -> Result<()> {
        let url = format!(
            "wss://api.openai.com/v1/realtime?model={}",
            self.config.openai_model
        );
        let mut request = tokio_tungstenite::tungstenite::client::IntoClientRequest::into_client_request(url.as_str())?;
        request.headers_mut().insert(
            "Authorization",
            format!("Bearer {}", self.config.openai_api_key).parse().unwrap(),
        );
        request.headers_mut().insert(
            "OpenAI-Beta",
            "realtime=v1".parse().unwrap(),
        );

        let (ws, _) = tokio_tungstenite::connect_async(request)
            .await
            .map_err(|e| anyhow!("WebSocket connect to OpenAI Realtime failed: {}", e))?;

        let (sink, mut stream) = ws.split();
        let sink = Arc::new(TokioMutex::new(sink));

        // Configure session: language hint, transcription model
        let session_update = serde_json::json!({
            "type": "transcription_session.update",
            "session": {
                "input_audio_format": "pcm16",
                "input_audio_transcription": {
                    "model": self.config.openai_model,
                    "language": if self.config.language_hint == "auto" {
                        serde_json::Value::Null
                    } else {
                        serde_json::Value::String(self.config.language_hint.clone())
                    }
                },
                "turn_detection": {
                    "type": "server_vad",
                    "threshold": 0.5,
                    "silence_duration_ms": 250
                }
            }
        });
        sink.lock().await
            .send(Message::Text(session_update.to_string()))
            .await
            .map_err(|e| anyhow!("session update send: {}", e))?;

        let on_transcript = self.on_transcript.clone();
        let start_time = std::time::Instant::now();
        let reader = tokio::spawn(async move {
            while let Some(msg) = stream.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        match serde_json::from_str::<serde_json::Value>(&text) {
                            Ok(v) => handle_event(&v, &on_transcript, start_time),
                            Err(e) => eprintln!("[asr] non-json frame: {} ({})", text, e),
                        }
                    }
                    Ok(Message::Close(_)) => {
                        eprintln!("[asr] WebSocket closed by server");
                        break;
                    }
                    Err(e) => {
                        eprintln!("[asr] WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });

        self.sender = Some(sink);
        self.reader_handle = Some(reader);
        self.start_time = Some(start_time);
        eprintln!("[asr] OpenAI Realtime Whisper session started");
        Ok(())
    }

    async fn send_audio(&mut self, pcm_16k_mono: &[f32], _speaker: &str) -> Result<()> {
        // OpenAI Realtime API expects PCM16 LE base64-encoded.
        // Note: ASR provider abstracts the speaker channel by tagging in TranscriptChunk
        // when results return. MVP: speaker is determined client-side by which audio
        // pipeline (mic vs system) called send_audio. Passing it here is a hint, but
        // the OpenAI API doesn't natively distinguish — we tag transcripts on the way out.
        //
        // For dual-channel transcription you actually need TWO concurrent WebSocket
        // sessions (one per speaker). MVP: single session, mix audio before send.
        let pcm16_bytes = f32_to_pcm16_bytes(pcm_16k_mono);
        let b64 = base64_encode(&pcm16_bytes);

        let frame = serde_json::json!({
            "type": "input_audio_buffer.append",
            "audio": b64
        });

        let sender = self.sender.as_ref().ok_or_else(|| anyhow!("ASR not started"))?;
        sender.lock().await
            .send(Message::Text(frame.to_string()))
            .await
            .map_err(|e| anyhow!("audio send: {}", e))?;
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        if let Some(sender) = self.sender.take() {
            let _ = sender.lock().await.close().await;
        }
        if let Some(handle) = self.reader_handle.take() {
            let _ = handle.await;
        }
        eprintln!("[asr] OpenAI Realtime Whisper session stopped");
        Ok(())
    }

    fn provider_name(&self) -> &'static str {
        "openai-realtime-whisper"
    }
}

fn handle_event(
    v: &serde_json::Value,
    on_transcript: &TranscriptCallback,
    start_time: std::time::Instant,
) {
    let event_type = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
    let offset = start_time.elapsed().as_secs_f64();

    match event_type {
        "conversation.item.input_audio_transcription.delta" => {
            if let Some(delta) = v.get("delta").and_then(|d| d.as_str()) {
                if !delta.is_empty() {
                    on_transcript(TranscriptChunk {
                        text: delta.to_string(),
                        speaker: "other".to_string(), // MVP: single-channel, default to "other"
                        offset_secs: offset,
                        is_final: false,
                    });
                }
            }
        }
        "conversation.item.input_audio_transcription.completed" => {
            if let Some(text) = v.get("transcript").and_then(|t| t.as_str()) {
                if !text.is_empty() {
                    on_transcript(TranscriptChunk {
                        text: text.to_string(),
                        speaker: "other".to_string(),
                        offset_secs: offset,
                        is_final: true,
                    });
                }
            }
        }
        "error" => {
            eprintln!("[asr] OpenAI error event: {}", v);
        }
        _ => {
            // Ignore other events for MVP: session.created, session.updated, etc.
        }
    }
}

/// Convert f32 [-1.0, 1.0] PCM to little-endian PCM16 bytes.
fn f32_to_pcm16_bytes(samples: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(samples.len() * 2);
    for &s in samples {
        let clamped = s.max(-1.0).min(1.0);
        let i16_val = (clamped * 32767.0) as i16;
        bytes.extend_from_slice(&i16_val.to_le_bytes());
    }
    bytes
}

/// Minimal base64 encoder (avoid pulling in base64 crate for one fn).
fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((data.len() + 2) / 3 * 4);
    let mut i = 0;
    while i + 3 <= data.len() {
        let n = ((data[i] as u32) << 16) | ((data[i + 1] as u32) << 8) | (data[i + 2] as u32);
        out.push(CHARS[((n >> 18) & 63) as usize] as char);
        out.push(CHARS[((n >> 12) & 63) as usize] as char);
        out.push(CHARS[((n >> 6) & 63) as usize] as char);
        out.push(CHARS[(n & 63) as usize] as char);
        i += 3;
    }
    let rem = data.len() - i;
    if rem == 1 {
        let n = (data[i] as u32) << 16;
        out.push(CHARS[((n >> 18) & 63) as usize] as char);
        out.push(CHARS[((n >> 12) & 63) as usize] as char);
        out.push('=');
        out.push('=');
    } else if rem == 2 {
        let n = ((data[i] as u32) << 16) | ((data[i + 1] as u32) << 8);
        out.push(CHARS[((n >> 18) & 63) as usize] as char);
        out.push(CHARS[((n >> 12) & 63) as usize] as char);
        out.push(CHARS[((n >> 6) & 63) as usize] as char);
        out.push('=');
    }
    out
}
```

- [ ] **Step 2: 验证编译**

Run:
```bash
cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | tail -15
```

Expected: 编译通过（带 unused warning OK）。如果错 trait method 签名不匹配：检查 `async_trait` 是否加进 dependencies。

---

### Task 9: 修改 commands.rs::start_recording 走新路径

**Files:**
- Modify: `src-tauri/src/commands.rs`（重写 `start_recording`）

这是 Week 1 改动最大的一处——把现有 cpal + sherpa 调用换成 ScreenCaptureKit + GPT-Realtime-Whisper。

- [ ] **Step 1: 加新 imports**

打开 `src-tauri/src/commands.rs`，在文件顶部 imports 段后加：

```rust
use crate::asr::{create_asr_provider, AsrConfig, TranscriptChunk};
use crate::audio::permission;
use crate::audio::system_capture::{create_system_audio_capture, AppFilter};
```

- [ ] **Step 2: 在 RecordingState 加 ASR provider 句柄**

找到 `commands.rs:86-110` 的 `RecordingState` struct + `new()`，在结构体加一个字段（暂时不需要持有 ASR、由 spawned task 管理生命周期）。

实际改动：**不改 `RecordingState`**，但 `start_recording` 内部 spawn 的 task 取代原来的 sherpa 循环。

- [ ] **Step 3: 替换 start_recording 函数体**

找到 `commands.rs:114` 处的 `pub async fn start_recording`。整个函数从第 114 行到第 473 行（`Ok(())` 结束）替换为：

```rust
#[command]
pub async fn start_recording(
    mic_device: String,
    state: tauri::State<'_, SharedRecordingState>,
    window: tauri::Window,
) -> Result<(), String> {
    // ScreenCaptureKit permission gate
    match permission::check_screen_recording_permission() {
        permission::PermissionStatus::Granted => {}
        _ => {
            return Err(
                "Screen Recording permission required. Open System Settings → Privacy & Security → Screen Recording, enable Confide, and restart the app.".into()
            );
        }
    }
    if !permission::macos_version_at_least(13, 0) {
        return Err("Confide requires macOS 13.0 or later.".into());
    }

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
    let start_time = rec.start_time.unwrap();

    let state_for_advisor: SharedRecordingState = Arc::clone(&state);
    let state_for_audio_loop: SharedRecordingState = Arc::clone(&state);
    let state_for_streams: SharedRecordingState = Arc::clone(&state);

    drop(rec);

    // === 1. Mic capture via existing cpal path ===
    let mic_buf_for_thread = mic_buffer.clone();
    let win_for_error = window.clone();
    std::thread::spawn(move || {
        let mic_stream = match capture::start_capture(&mic_device, mic_buf_for_thread) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[audio] Mic capture failed: {}", e);
                let _ = win_for_error.emit("backend-error", serde_json::json!({
                    "source": "audio",
                    "message": format!("Microphone start failed: {}", e)
                }));
                return;
            }
        };
        eprintln!("[audio] Mic stream started, holding alive...");
        loop {
            std::thread::sleep(std::time::Duration::from_millis(200));
            if let Ok(rec) = state_for_streams.try_lock() {
                if !rec.is_recording { break; }
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
        let mut sckit = match create_system_audio_capture(cap_buf_for_sckit, AppFilter::default()) {
            Ok(b) => b,
            Err(e) => {
                let _ = win_for_sckit.emit("backend-error", serde_json::json!({
                    "source": "audio",
                    "message": format!("System audio capture init failed: {}", e)
                }));
                return;
            }
        };
        if let Err(e) = sckit.start() {
            let _ = win_for_sckit.emit("backend-error", serde_json::json!({
                "source": "audio",
                "message": format!("System audio capture start failed: {}", e)
            }));
            return;
        }
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            let rec = state_for_sckit.lock().await;
            if !rec.is_recording { break; }
        }
        let _ = sckit.stop();
        eprintln!("[audio] System audio capture stopped");
    });

    // === 3. ASR loop: drain both buffers, mix, send to GPT-Realtime-Whisper ===
    let asr_config = {
        let cfg = config::load_config().unwrap_or_default();
        AsrConfig {
            provider: "openai-realtime-whisper".into(),
            // For Week 1: read from llm config slot (alpha repurposes it). Week 5 splits into proper asr config.
            openai_api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default(),
            openai_model: "gpt-realtime-whisper".into(),
            language_hint: cfg.language_preference.clone(),
        }
    };
    let win_for_asr = window.clone();
    let transcript_for_asr = transcript.clone();
    tokio::spawn(async move {
        let on_transcript: crate::asr::TranscriptCallback = std::sync::Arc::new(move |chunk: TranscriptChunk| {
            // Push final-only chunks into transcript store; emit deltas as live captions
            if chunk.is_final {
                {
                    let mut store = transcript_for_asr.lock().unwrap();
                    store.add(chunk.text.clone(), chunk.offset_secs, &chunk.speaker);
                }
                let segment = crate::transcript::store::TranscriptSegment {
                    timestamp: chrono::Utc::now(),
                    text: chunk.text.clone(),
                    offset_secs: chunk.offset_secs,
                    speaker: chunk.speaker.clone(),
                };
                let _ = win_for_asr.emit("new-transcript", &segment);
            } else {
                // Live partial — emit on a separate event for UI typewriter effect
                let _ = win_for_asr.emit("transcript-delta", &chunk);
            }
        });

        let mut asr = match create_asr_provider(&asr_config, on_transcript) {
            Ok(p) => p,
            Err(e) => {
                let _ = win_for_asr.emit("backend-error", serde_json::json!({
                    "source": "asr",
                    "message": format!("ASR provider init failed: {}", e)
                }));
                return;
            }
        };
        if let Err(e) = asr.start().await {
            let _ = win_for_asr.emit("backend-error", serde_json::json!({
                "source": "asr",
                "message": format!("ASR session start failed: {}", e)
            }));
            return;
        }

        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            let (is_recording, is_paused) = {
                let rec = state_for_audio_loop.lock().await;
                (rec.is_recording, rec.is_paused)
            };
            if !is_recording { break; }
            if is_paused { continue; }

            // Drain mic
            let mic_data = {
                let mut buf = mic_buffer.lock().unwrap();
                if buf.len() > 0 { buf.drain_all() } else { vec![] }
            };
            // Drain system
            let cap_data = {
                let mut buf = capture_buffer.lock().unwrap();
                if buf.len() > 0 { buf.drain_all() } else { vec![] }
            };

            // MVP: mix both channels (average) before sending to single ASR session.
            // Speaker diarization deferred to v1.0.5 (requires dual sessions).
            if !mic_data.is_empty() || !cap_data.is_empty() {
                let mixed = mix_audio(&mic_data, &cap_data);
                if let Err(e) = asr.send_audio(&mixed, "mixed").await {
                    eprintln!("[asr] send_audio error: {}", e);
                }
            }
        }
        let _ = asr.stop().await;
    });

    // === 4. Advisor loop (unchanged from existing demo, see commands.rs lines 354-470) ===
    spawn_advisor_loop(state_for_advisor, transcript.clone(), window.clone(), start_time);

    Ok(())
}

/// Mix mic and capture into single channel by averaging where both have data,
/// or using whichever is non-empty otherwise.
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

/// Extract original advisor loop into a free function, called from start_recording.
fn spawn_advisor_loop(
    state: SharedRecordingState,
    transcript: SharedTranscriptStore,
    window: tauri::Window,
    start_time: std::time::Instant,
) {
    // ... copy lines 356-470 of the original commands.rs verbatim, with `state_for_advisor` renamed `state`
    //
    // For Week 1, copy the entire tokio::spawn(async move { ... }) block from the
    // existing start_recording's advisor-loop section into this function body.
    // Do NOT change the advisor logic — that's Week 5's job (switching to Anthropic + caching).
    tokio::spawn(async move {
        let _ = (state, transcript, window, start_time);
        // PASTE original advisor loop body here.
        eprintln!("[advisor] (Week 1: paste original advisor loop body from commands.rs:356-470)");
    });
}
```

- [ ] **Step 4: 把原来的 advisor 循环代码搬到 spawn_advisor_loop 函数**

去 git history 或当前 commands.rs 找到行 354-470 的 `tokio::spawn(async move {` 块，把它整体复制进 `spawn_advisor_loop` 函数体里替换 placeholder。

注意：原代码里用 `state_for_advisor`、`transcript_for_advisor`、`win_for_advisor`——重命名为 `state`、`transcript`、`window` 以匹配新函数签名。`start_time` 变量参数化传入。

- [ ] **Step 5: 删掉 start_recording 函数签名里的 `capture_device` 参数**

新签名是 `start_recording(mic_device: String, ...)`，没有 `capture_device`——因为 ScreenCaptureKit 不需要用户选设备。

需要同步改前端 `src/lib/tauri.ts:28`：

```typescript
export const startRecording = (micDevice: string) =>
  invoke<void>("start_recording", {
    micDevice,
  });
```

以及任何调用 `startRecording(mic, capture)` 两参数的地方。MVP 阶段最小改动：搜索调用点，去掉 capture 参数。

Run:
```bash
cd /Users/bytedance/meeting-assistant
grep -n "startRecording" src/ -r
```

把每个调用点的 capture 参数去掉。

- [ ] **Step 6: 验证编译**

Run:
```bash
cd /Users/bytedance/meeting-assistant
cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | tail -15
pnpm typecheck 2>&1 | tail -15
```

Expected: 两个都通过。如果有 type errors：通常是前端 startRecording 调用点漏改。

---

### Task 10: 加新 Tauri command — check_screen_recording_permission

**Files:**
- Modify: `src-tauri/src/commands.rs`（加新 command）
- Modify: `src-tauri/src/main.rs`（注册到 invoke_handler）
- Modify: `src/lib/tauri.ts`（前端 wrapper）

- [ ] **Step 1: 在 commands.rs 文件末尾加 command**

```rust
// --- Permission ---

#[derive(Serialize)]
pub struct ScreenRecordingPermissionStatus {
    pub status: String,  // "granted" | "denied" | "not-determined"
    pub macos_version_ok: bool,
}

#[command]
pub fn check_screen_recording_permission() -> Result<ScreenRecordingPermissionStatus, String> {
    use crate::audio::permission;
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
    crate::audio::permission::open_settings_screen_recording()
        .map_err(|e| e.to_string())
}
```

- [ ] **Step 2: 在 main.rs 注册**

打开 `src-tauri/src/main.rs:26-50`，在 `invoke_handler!` 列表里加：

```rust
            commands::check_screen_recording_permission,
            commands::open_screen_recording_settings,
```

- [ ] **Step 3: 在 src/lib/tauri.ts 加 wrapper**

打开 `src/lib/tauri.ts`，在 commands 段加：

```typescript
export const checkScreenRecordingPermission = () =>
  invoke<{ status: string; macos_version_ok: boolean }>(
    "check_screen_recording_permission"
  );

export const openScreenRecordingSettings = () =>
  invoke<void>("open_screen_recording_settings");
```

- [ ] **Step 4: 验证编译**

Run:
```bash
cargo check --manifest-path src-tauri/Cargo.toml 2>&1 | tail -5
pnpm typecheck 2>&1 | tail -5
```

Expected: 都通过。

---

### Task 11: 加 onboarding 弹窗组件（首次启动检查权限）

**Files:**
- Create: `src/components/onboarding/PermissionGate.tsx`
- Modify: `src/App.tsx`（在顶层包一层）

- [ ] **Step 1: 创建 PermissionGate 组件**

写 `src/components/onboarding/PermissionGate.tsx`:

```tsx
import { useEffect, useState } from "react";
import {
  checkScreenRecordingPermission,
  openScreenRecordingSettings,
} from "../../lib/tauri";

interface Props {
  children: React.ReactNode;
}

export function PermissionGate({ children }: Props) {
  const [status, setStatus] = useState<
    "checking" | "ok" | "denied" | "macos-too-old"
  >("checking");

  useEffect(() => {
    void check();
  }, []);

  async function check() {
    try {
      const r = await checkScreenRecordingPermission();
      if (!r.macos_version_ok) {
        setStatus("macos-too-old");
        return;
      }
      setStatus(r.status === "granted" ? "ok" : "denied");
    } catch (e) {
      console.error("permission check failed:", e);
      setStatus("denied");
    }
  }

  if (status === "checking") {
    return <div className="p-8 text-center">Checking permissions…</div>;
  }

  if (status === "macos-too-old") {
    return (
      <div className="p-8 max-w-md mx-auto text-[var(--text-primary)]">
        <h2 className="text-xl font-bold mb-3">macOS 13 or later required</h2>
        <p className="text-sm mb-4 leading-relaxed">
          Confide uses Apple's ScreenCaptureKit framework to capture meeting
          audio without requiring you to install third-party drivers like
          BlackHole. This framework is only available on macOS 13.0 (Ventura)
          and later.
        </p>
        <p className="text-sm">
          macOS 12 support is on our roadmap — see{" "}
          <a
            href="https://confide.knosi.xyz/roadmap"
            className="underline"
            target="_blank"
            rel="noreferrer"
          >
            roadmap
          </a>
          .
        </p>
      </div>
    );
  }

  if (status === "denied") {
    return (
      <div className="p-8 max-w-md mx-auto text-[var(--text-primary)]">
        <h2 className="text-xl font-bold mb-3">
          Screen Recording permission required
        </h2>
        <p className="text-sm mb-4 leading-relaxed">
          Confide needs Screen Recording access to capture meeting audio
          (Zoom, Meet, Teams). We never see your screen — only system audio.
          macOS doesn't provide a separate audio-only permission, which is why
          this looks scarier than it is.
        </p>
        <p className="text-sm mb-6">
          After enabling, you must <b>quit and restart Confide</b> for the
          permission to take effect.
        </p>
        <div className="flex gap-3">
          <button
            className="px-4 py-2 bg-[var(--accent-purple)] rounded text-white text-sm"
            onClick={async () => {
              await openScreenRecordingSettings();
            }}
          >
            Open System Settings
          </button>
          <button
            className="px-4 py-2 border border-[var(--border)] rounded text-sm"
            onClick={() => void check()}
          >
            Re-check
          </button>
        </div>
      </div>
    );
  }

  return <>{children}</>;
}
```

- [ ] **Step 2: 在 App.tsx 包一层**

打开 `src/App.tsx`，把整个 return 包在 `<PermissionGate>` 里：

```tsx
import { PermissionGate } from "./components/onboarding/PermissionGate";

// ... existing code

  return (
    <PermissionGate>
      <div className="h-screen bg-[var(--bg-primary)] text-[var(--text-primary)]">
        {view === "narrow" && (...)}
        ...
      </div>
    </PermissionGate>
  );
```

- [ ] **Step 3: 验证类型**

Run:
```bash
pnpm typecheck 2>&1 | tail -5
```

Expected: 通过。

---

### Task 12: 端到端验证 — 跑通"打开 Confide → 录 30 秒 Zoom 通话 → 看到英文 transcript"

这是 Week 1 的 acceptance test。**必须通过才算 Week 1 完成**。

- [ ] **Step 1: 把 OPENAI_API_KEY 设到环境变量**

```bash
export OPENAI_API_KEY="<from Week 0 decision-log: 1P/confide-week1-asr>"
```

或写到 `~/.confide/.env` 让 Tauri 启动时读（MVP 简化：直接走 env var）。

- [ ] **Step 2: 启动 Tauri dev**

Run:
```bash
cd /Users/bytedance/meeting-assistant
OPENAI_API_KEY="<key>" pnpm tauri dev
```

第一次启动 Rust 编译 5-10 分钟。

- [ ] **Step 3: 应用启动后看 PermissionGate**

如果第一次启动：会弹"Screen Recording permission required" → 点 "Open System Settings" → 在 Privacy & Security > Screen Recording 列表里找到 Confide / Tauri Dev → 勾选 → 退出 Confide → 重启。

第二次启动：PermissionGate 应该自动通过、进 NarrowView。

- [ ] **Step 4: 启动 Zoom，加入测试会议（自己 host 一个 + 用手机加）**

Zoom 必须真的在播放音频（让手机端说话或开 Zoom 自带的 test meeting）。

- [ ] **Step 5: 在 Confide 点录音**

NarrowView 上的 Start 按钮 → 应该立刻开始。

观察 Tauri terminal 输出，应看到：
```
[sckit] Capture started
[asr] OpenAI Realtime Whisper session started
```

- [ ] **Step 6: 等 10-15 秒，看 UI 上是否出现 transcript**

Expected: NarrowView TranscriptMini 区域出现实时字幕，内容是 Zoom 里说的话。**可能有 1-2 秒延迟**。

如果 30 秒后**没有任何 transcript**：
- 看 Tauri terminal 有没有 `[asr]` 错误日志
- 看 ASR session 是否成功 start（没看到 "session started" 日志说明 Task 8 WebSocket 没连上）
- 看 audio 是否真的进 buffer：临时在 mix_audio 后加 `eprintln!("[asr] sending {} samples", mixed.len())` 验证
- OpenAI key 是否正确（401 错误会在 Tauri terminal 输出）

- [ ] **Step 7: 点 Stop，看清理是否干净**

Stop 后看 Tauri terminal：
```
[audio] Mic stream dropped
[audio] System audio capture stopped
[asr] OpenAI Realtime Whisper session stopped
```

不应该有 panic 或继续刷的日志。

- [ ] **Step 8: 标记 Week 1 完成**

在 `docs/general/plans/decision-log.md` 加：

```
## Week 1 完成
- 日期: <2026-05-XX>
- 验收: ✅ 不需 BlackHole 录到 Zoom 30 秒英文 transcript
- 已知问题:
  - <填问题>
- 推迟事项:
  - 单 ASR session 混 mic+system，speaker diarization 推到 v1.0.5
  - <填其他>
```

---

## Week 1 完成标志（Acceptance Criteria）

对应 design Section 9 的 AC：
- ✅ A3 不需要 BlackHole / 第三方驱动（ScreenCaptureKit 替代）
- ✅ B1 系统音频 + 麦克风双通道捕获正确（双 buffer）
- ✅ B6 首次启动到能开始录音 ≤30 秒（无模型下载）
- ✅ B7 OpenAI Realtime API outage 时弹错误（backend-error event）
- ⚠️ B2 转录质量 WER ≤7% — 体感验证，不严格 benchmark
- ⚠️ B3 转录延迟 ≤500ms — 体感验证
- ⚠️ B5 CPU 占用 ≤15% on M1/M2 — 用 Activity Monitor 看一眼
- ❌ B1 transcript 区分 me/other — **MVP 做不到**（单 ASR session），推迟 v1.0.5

下一步：进 Week 2 — Stealth 模式 + 模板基座。
