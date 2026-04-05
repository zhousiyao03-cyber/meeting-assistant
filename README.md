# Meeting Copilot

Real-time meeting assistant for macOS. Transcribes audio, generates live summaries, and suggests when and what to say — all running locally on your Mac.

![Tauri](https://img.shields.io/badge/Tauri-2.0-blue) ![React](https://img.shields.io/badge/React-19-blue) ![Rust](https://img.shields.io/badge/Rust-2021-orange)

**[中文文档](README_CN.md)**

---

## Features

- **Real-time transcription** — Local SenseVoice model (sherpa-onnx) + Silero VAD, supporting Chinese, English, Japanese, Korean, and Cantonese
- **Speaker diarization** — Dual-channel recognition: mic ("me") vs. system audio ("other"), with echo suppression and cross-channel dedup
- **Live meeting summary** — LLM-generated rolling summaries every 30 seconds
- **Speaking suggestions** — Detects opportune moments to speak (questions, pauses, keyword triggers) and suggests what to say
- **Dual audio capture** — Records both your mic and remote participants' audio (via BlackHole virtual device)
- **Meeting templates** — Pre-built prompts for tech reviews, code reviews, brainstorms, and project syncs, with a full template editor UI
- **Reference documents** — Load agendas or docs for context-aware AI suggestions
- **Meeting history** — Auto-saves transcripts, summaries, and action items to local SQLite, with Markdown export
- **Meeting minutes** — Auto-generates title, key points, and action items when a meeting ends
- **Privacy first** — Speech recognition runs 100% locally. No audio leaves your machine — only transcript text is sent to the LLM API

## Screenshots

The app runs as a compact always-on-top panel (420×840) alongside your video conferencing app:

- **Narrow view** — Compact panel with real-time transcript, summary, and speaking advice
- **Full view** — Expanded layout with template selector, full transcript, and resizable AI copilot panel

## Prerequisites

- **macOS** (CoreAudio via cpal)
- **Rust** (latest stable)
- **Node.js** 18+ and **pnpm**
- **BlackHole 2ch** — Virtual audio driver for system audio capture ([download](https://existential.audio/blackhole/))

### BlackHole Setup

1. Install BlackHole 2ch
2. Open **Audio MIDI Setup** (built into macOS)
3. Create a **Multi-Output Device** combining your headphones + BlackHole 2ch
4. Set the Multi-Output Device as your system audio output
5. In Meeting Copilot settings, select BlackHole 2ch as the capture device

## Getting Started

```bash
# Clone and install
git clone <repo-url>
cd meeting-assistant
pnpm install

# Run in dev mode (first build compiles Rust, ~5 min)
pnpm tauri dev
```

On first launch:
1. The app downloads the SenseVoice model (~200 MB) + Silero VAD automatically
2. Configure audio devices in Settings (mic + BlackHole)
3. Enter your LLM API key in Settings
4. Click **+ New Meeting** to start

## Configuration

Settings are stored in `~/.meeting-assistant/config.json`:

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

The LLM endpoint must be OpenAI-compatible (`/v1/chat/completions`). Any compatible provider works.

## Meeting Templates

Templates customize AI behavior for different meeting types. Located in `templates/`:

| Template | Use Case | Trigger Examples |
|----------|----------|-----------------|
| tech-review | Architecture & design reviews | Opinion requests, proposal mentions |
| code-review | Code review discussions | Review questions, feedback prompts |
| brainstorm | Brainstorming sessions | Idea solicitation, open-ended prompts |
| project-sync | Status & standup meetings | Progress checks, blocker mentions |

Each template includes a `system_prompt` for shaping the AI's advice style, and `trigger_hints` — keywords that activate speaking suggestions when detected in the transcript.

## Speech Recognition

Built on **sherpa-onnx** with three core components:

| Component | Model | Description |
|-----------|-------|-------------|
| ASR | **SenseVoice** (int8) | Multilingual model by Alibaba DAMO Academy (zh/en/ja/ko/yue), ~200 MB |
| VAD | **Silero VAD** | Lightweight voice activity detection, 512-sample (32 ms) windows |
| Runtime | **ONNX Runtime** | CPU-only inference via sherpa-onnx, no GPU required |

### Pipeline

1. **Dual-channel capture** — Mic and system audio captured separately as 16 kHz mono f32 PCM
2. **Independent VAD + ASR** — Each channel runs its own Silero VAD → SenseVoice pipeline
3. **Speech segmentation** — VAD triggers after ≥250 ms of speech, segments after ≥250 ms of silence, max 8 s per segment
4. **Echo suppression** — Mic results are suppressed when system audio is active (prevents speaker-to-mic leakage)
5. **Cross-channel dedup** — Segments with >50% character similarity within a 3 s window are filtered

Models are stored in `~/.meeting-assistant/models/` and downloaded automatically on first launch.

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
# Frontend only (hot reload on :1420)
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
│   │   ├── history/              # Meeting history
│   │   └── shared/               # Reusable components
│   ├── hooks/                    # useRecording, useTauriEvents
│   └── lib/                      # Tauri API wrapper, types
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── audio/                # Audio capture & buffering
│   │   ├── whisper/              # SenseVoice + Silero VAD engine
│   │   ├── advisor/              # LLM integration & trigger rules
│   │   ├── transcript/           # Transcript storage
│   │   ├── storage/              # Config & history (SQLite)
│   │   ├── documents/            # Reference doc loader
│   │   └── commands.rs           # All Tauri commands
│   └── icons/                    # App icons
└── templates/                    # Meeting type templates (JSON)
```

## Data Storage

All data stays on your machine:

| Data | Location |
|------|----------|
| App config | `~/.meeting-assistant/config.json` |
| ASR models | `~/.meeting-assistant/models/` |
| Meeting history | `~/.meeting-assistant/history.db` |

## License

Private project.
