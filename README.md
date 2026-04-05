# Meeting Copilot

Real-time meeting assistant for macOS. Transcribes audio, generates meeting summaries, and suggests when and what to say — all running locally on your Mac.

![Tauri](https://img.shields.io/badge/Tauri-2.0-blue) ![React](https://img.shields.io/badge/React-19-blue) ![Rust](https://img.shields.io/badge/Rust-2021-orange)

## Features

- **Real-time transcription** — SenseVoice model (via sherpa-onnx) + Silero VAD，本地实时转写，支持中/英/日/韩/粤语
- **说话人识别** — 双通道独立转写：麦克风（我）和系统音频（对方）分别识别，自动标记说话人，内置回声抑制和跨通道去重
- **Meeting summary** — LLM generates rolling summaries of key discussion points every 30 seconds
- **Speaking suggestions** — AI detects when it's a good time to speak (questions, pauses, keyword triggers) and suggests what to say
- **Dual audio capture** — Records both your microphone and system audio (remote participants via BlackHole)
- **Meeting templates** — Pre-configured prompts for tech reviews, code reviews, brainstorms, and project syncs with full UI editor
- **Reference documents** — Load meeting agendas or docs for context-aware suggestions
- **Meeting history** — Auto-saves transcripts, summaries, and action items to local SQLite database, with Markdown export
- **Meeting minutes** — LLM auto-generates meeting title, key points, and action items on meeting end
- **Privacy first** — ASR runs 100% locally, no audio leaves your machine. Only text is sent to the LLM API.

## Screenshots

The app runs as a compact always-on-top panel alongside your video conferencing app:

- **Narrow view** — Compact panel with real-time transcript, summary, and advice
- **Full view** — Expanded layout with template selector, full transcript, and resizable AI copilot panel

## Prerequisites

- **macOS** (uses CoreAudio via cpal)
- **Rust** (latest stable)
- **Node.js** 18+ and **pnpm**
- **BlackHole 2ch** — Virtual audio driver for capturing system audio ([download](https://existential.audio/blackhole/))

### BlackHole Setup

1. Install BlackHole 2ch
2. Open **Audio MIDI Setup** (macOS built-in)
3. Create a **Multi-Output Device** with your headphones + BlackHole 2ch
4. Set the Multi-Output Device as system audio output
5. In Meeting Copilot settings, select BlackHole 2ch as the capture device

## Getting Started

```bash
# Clone and install
git clone <repo-url>
cd meeting-assistant
pnpm install

# Run in dev mode (first run downloads SenseVoice model ~200MB and compiles Rust ~5min)
pnpm tauri dev
```

On first launch:
1. The app will prompt you to download the SenseVoice model (~200MB) + Silero VAD
2. Configure your audio devices in Settings (mic + BlackHole)
3. Set your LLM API key in Settings
4. Click **+ 新建会议** to start recording

## Configuration

App settings are stored in `~/.meeting-assistant/config.json`:

```json
{
  "llm": {
    "base_url": "https://llmgate.io/v1",
    "api_key": "your-api-key",
    "model": "gpt-5.4"
  },
  "audio": {
    "mic_device": "MacBook Pro Microphone",
    "capture_device": "BlackHole 2ch",
    "noise_reduction": true
  },
  "language_preference": "auto",
  "analysis_mode": "balanced"
}
```

The LLM endpoint must be OpenAI-compatible (chat completions API). Any provider that supports `/v1/chat/completions` will work.

## Meeting Templates

Templates define how the AI assists during different meeting types. Located in `templates/`:

| Template | Description | Trigger Examples |
|----------|-------------|-----------------|
| tech-review | Architecture & design reviews | "大家觉得怎么样", "这个方案" |
| code-review | Code review discussions | "有没有问题", "还有其他意见吗" |
| brainstorm | Brainstorming sessions | "谁有想法", "我们可以" |
| project-sync | Project status meetings | "进度怎么样", "有什么blocker" |

Each template includes a `system_prompt` that shapes the AI's advice style, and `trigger_hints` — keywords that, when detected in the transcript, prompt the AI to generate a speaking suggestion.

## Speech Recognition Architecture

转写引擎采用 **sherpa-onnx** 框架，核心组件：

| Component | Model | Description |
|-----------|-------|-------------|
| ASR | **SenseVoice** (int8 quantized) | 阿里达摩院开源的多语言 ASR 模型，支持中/英/日/韩/粤语，int8 量化后约 200MB |
| VAD | **Silero VAD** | 轻量级语音活动检测，512 采样窗口，自动分割语音段落 |
| Runtime | **ONNX Runtime** (via sherpa-onnx) | 跨平台推理引擎，CPU 运行，无需 GPU |

### 工作原理

1. **双通道采集** — 麦克风和系统音频（BlackHole）分别采集为 16kHz mono f32 PCM 流
2. **独立 VAD + ASR** — 每个通道有独立的 Silero VAD + SenseVoice 引擎实例，互不干扰
3. **语音段落检测** — Silero VAD 以 512 采样（32ms）为窗口检测语音活动，当检测到 ≥250ms 语音后开始积累，≥250ms 静音后切段
4. **离线识别** — 切出的语音段送入 SenseVoice 进行离线识别（非流式），每段最长 8 秒
5. **回声抑制** — 当系统音频通道活跃时，自动抑制麦克风通道的识别结果（消除扬声器漏入麦克风的回声）
6. **跨通道去重** — 3 秒内两通道产出相似度 >50% 的文本时，自动过滤重复条目

模型文件存储在 `~/.meeting-assistant/models/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-2024-07-17/`，首次启动时自动下载。

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop framework | Tauri 2 |
| Frontend | React 19, TypeScript, Tailwind CSS 4 |
| Build tool | Vite 6 |
| Backend | Rust (2021 edition) |
| Audio capture | cpal + BlackHole 2ch |
| Speech-to-text | sherpa-onnx (SenseVoice + Silero VAD) |
| LLM | OpenAI-compatible API |
| Database | SQLite (rusqlite) |
| Package manager | pnpm |

## Development

```bash
# Frontend only (hot reload)
pnpm dev

# Full app with Tauri
pnpm tauri dev

# Type checking
pnpm typecheck              # Frontend
cd src-tauri && cargo check  # Backend

# Tests
cd src-tauri && cargo test

# Production build
pnpm tauri build
```

## Project Structure

```
meeting-assistant/
├── src/                          # React frontend
│   ├── components/
│   │   ├── narrow/               # Compact panel view
│   │   ├── full/                 # Expanded view
│   │   ├── settings/             # Settings tabs
│   │   └── shared/               # Reusable components
│   ├── hooks/                    # useRecording, useTauriEvents
│   └── lib/                      # Tauri API wrapper, types
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── audio/                # Audio capture & buffering
│   │   ├── whisper/              # Speech-to-text engine
│   │   ├── advisor/              # LLM integration & triggers
│   │   ├── transcript/           # Transcript storage
│   │   ├── storage/              # Config & history persistence
│   │   ├── documents/            # Reference doc loading
│   │   └── commands.rs           # All Tauri commands
│   └── icons/                    # App icons
└── templates/                    # Meeting type templates (JSON)
```

## Data Storage

All data stays on your machine:

| Data | Location |
|------|----------|
| App config | `~/.meeting-assistant/config.json` |
| ASR models | `~/.meeting-assistant/models/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-2024-07-17/` |
| Meeting history | `~/.meeting-assistant/history.db` |

## License

Private project.
