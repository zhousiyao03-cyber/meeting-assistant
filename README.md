# Meeting Copilot

Real-time meeting assistant for macOS. Transcribes audio, generates meeting summaries, and suggests when and what to say — all running locally on your Mac.

![Tauri](https://img.shields.io/badge/Tauri-2.0-blue) ![React](https://img.shields.io/badge/React-19-blue) ![Rust](https://img.shields.io/badge/Rust-2021-orange)

[English](#features) | [中文](#功能特性)

---

## Features

- **Real-time transcription** — SenseVoice model (via sherpa-onnx) + Silero VAD, runs locally with support for Chinese, English, Japanese, Korean, and Cantonese
- **Speaker diarization** — Dual-channel independent transcription: microphone ("me") and system audio ("other") are recognized separately with automatic speaker labeling, built-in echo suppression, and cross-channel deduplication
- **Meeting summary** — LLM generates rolling summaries of key discussion points every 30 seconds
- **Speaking suggestions** — AI detects when it's a good time to speak (questions, pauses, keyword triggers) and suggests what to say
- **Dual audio capture** — Records both your microphone and system audio (remote participants via BlackHole)
- **Meeting templates** — Pre-configured prompts for tech reviews, code reviews, brainstorms, and project syncs with full UI editor
- **Reference documents** — Load meeting agendas or docs for context-aware suggestions
- **Meeting history** — Auto-saves transcripts, summaries, and action items to local SQLite database, with Markdown export
- **Meeting minutes** — LLM auto-generates meeting title, key points, and action items on meeting end
- **Privacy first** — ASR runs 100% locally, no audio leaves your machine. Only text is sent to the LLM API

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
4. Click **+ New Meeting** to start recording

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
| tech-review | Architecture & design reviews | opinion requests, proposal mentions |
| code-review | Code review discussions | review questions, feedback prompts |
| brainstorm | Brainstorming sessions | idea solicitation, open suggestions |
| project-sync | Project status meetings | progress checks, blocker mentions |

Each template includes a `system_prompt` that shapes the AI's advice style, and `trigger_hints` — keywords that, when detected in the transcript, prompt the AI to generate a speaking suggestion.

## Speech Recognition Architecture

The transcription engine is built on the **sherpa-onnx** framework with the following core components:

| Component | Model | Description |
|-----------|-------|-------------|
| ASR | **SenseVoice** (int8 quantized) | Multilingual ASR model from Alibaba DAMO Academy, supports zh/en/ja/ko/yue, ~200MB after int8 quantization |
| VAD | **Silero VAD** | Lightweight voice activity detection with 512-sample windows for automatic speech segmentation |
| Runtime | **ONNX Runtime** (via sherpa-onnx) | Cross-platform inference engine, CPU-only, no GPU required |

### How It Works

1. **Dual-channel capture** — Microphone and system audio (BlackHole) are captured separately as 16kHz mono f32 PCM streams
2. **Independent VAD + ASR** — Each channel has its own Silero VAD + SenseVoice engine instance, fully isolated
3. **Speech segmentation** — Silero VAD uses 512-sample (32ms) windows to detect voice activity; accumulates after ≥250ms of speech, segments after ≥250ms of silence
4. **Offline recognition** — Segmented speech is fed to SenseVoice for offline (non-streaming) recognition, max 8 seconds per segment
5. **Echo suppression** — When the system audio channel is active, microphone channel results are suppressed (eliminates speaker-to-mic leakage)
6. **Cross-channel dedup** — When both channels produce text with >50% similarity within a 3-second window, the duplicate is automatically filtered

Models are stored in `~/.meeting-assistant/models/sherpa-onnx-sense-voice-zh-en-ja-ko-yue-2024-07-17/` and downloaded automatically on first launch.

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
│   │   ├── history/              # Meeting history view
│   │   └── shared/               # Reusable components
│   ├── hooks/                    # useRecording, useTauriEvents
│   └── lib/                      # Tauri API wrapper, types
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── audio/                # Audio capture & buffering
│   │   ├── whisper/              # ASR engine (SenseVoice + VAD)
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

---

## 功能特性

- **实时转写** — SenseVoice 模型（via sherpa-onnx）+ Silero VAD，本地运行，支持中/英/日/韩/粤语
- **说话人识别** — 双通道独立转写：麦克风（我）和系统音频（对方）分别识别，自动标记说话人，内置回声抑制和跨通道去重
- **会议摘要** — LLM 每 30 秒自动生成讨论要点滚动摘要
- **发言建议** — AI 检测适合发言的时机（提问、停顿、关键词触发），并建议说什么
- **双通道音频采集** — 同时录制麦克风和系统音频（通过 BlackHole 采集远端参会者声音）
- **会议模板** — 内置技术评审、代码评审、头脑风暴、项目同步等场景模板，支持完整 UI 编辑器
- **参考文档** — 加载会议议程或文档，为 AI 提供上下文感知建议
- **会议历史** — 自动保存转写、摘要和行动项到本地 SQLite 数据库，支持 Markdown 导出
- **会议纪要** — 会议结束时 LLM 自动生成会议标题、要点和行动项
- **隐私优先** — ASR 100% 本地运行，音频不离开你的设备。仅文本发送至 LLM API

## License

Private project.
