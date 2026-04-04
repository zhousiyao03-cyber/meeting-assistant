# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Meeting Copilot — a macOS desktop app for real-time meeting assistance. Captures mic + system audio, transcribes via local Whisper, and provides LLM-powered meeting summaries and speaking advice. Built as an always-on-top narrow panel (420x840) designed to sit alongside video conferencing apps.

## Commands

```bash
# Package manager: pnpm (not npm)
pnpm install              # Install frontend dependencies
pnpm dev                  # Vite dev server on localhost:1420
pnpm build                # TypeScript check + Vite production build
pnpm typecheck            # TypeScript only

# Full app (Tauri + frontend)
pnpm tauri dev             # Dev mode with hot reload (first Rust compile ~5min)
pnpm tauri build           # Production build (.dmg / .app)

# Rust backend
cd src-tauri && cargo build
cd src-tauri && cargo check   # Fast type check without linking
cd src-tauri && cargo test    # Run all tests
cd src-tauri && cargo test rules  # Run specific test module
```

## Architecture

**Tauri 2 (Rust) + React 19 + Vite 6 + Tailwind CSS 4**

### Backend (`src-tauri/src/`)

All Tauri commands registered in `main.rs`, implemented in `commands.rs`. This is the single bridge between frontend and backend.

- **commands.rs** — Central command file. Contains `RecordingState` (shared via `Arc<TokioMutex<>>`), recording lifecycle, and all `#[command]` functions. Audio capture streams are held in a dedicated `std::thread` (because `cpal::Stream` is `!Send`).
- **audio/** — Audio capture via `cpal`. Ring buffer (`SharedBuffer`) accumulates 2-second chunks at 16kHz mono f32. `capture.rs` resamples from device native format. Mic and system audio (via BlackHole) are mixed before transcription.
- **whisper/** — Local speech-to-text. `downloader.rs` fetches ggml-medium model (~1.5GB) with atomic `.tmp` rename on completion. `engine.rs` wraps whisper-rs with silence detection (RMS threshold), CJK-aware hallucination filtering, and `initial_prompt` for cross-chunk context.
- **advisor/** — LLM integration via OpenAI-compatible chat API (default: LLMGate). `engine.rs` generates summaries (every 30s) and speaking advice (every 10s, trigger-gated). `rules.rs` evaluates triggers: keyword hints from templates, question detection, silence pauses. `templates.rs` manages meeting type JSON files.
- **transcript/** — `TranscriptStore` with timestamped segments, shared via `Arc<Mutex<>>`. Supports full text and recent-N-seconds queries.
- **storage/** — `config.rs` persists `AppConfig` to `~/.meeting-assistant/config.json`. `history.rs` saves meeting records to SQLite (`~/.meeting-assistant/history.db`).
- **documents/** — Loads `.md`/`.txt`/`.pdf` reference documents, chunks them, and selects relevant chunks by keyword overlap with transcript.

### Frontend (`src/`)

- **lib/tauri.ts** — All `invoke()` and `listen()` calls. Only file importing `@tauri-apps/api`.
- **lib/types.ts** — TypeScript interfaces mirroring Rust structs.
- **hooks/useRecording.ts** — Recording lifecycle (start/stop/pause/resume). Syncs with backend state on mount via `get_recording_status`. Auto-saves meeting history on stop.
- **hooks/useTauriEvents.ts** — Subscribes to `new-transcript`, `meeting-summary`, `speaking-advice` events. Includes client-side dedup (same text within 1s).
- **components/narrow/** — Primary compact view: `ControlBar` (template name, timer, buttons), `TranscriptMini` (last 10 segments), `SummaryPanel`, `AdvicePanel`.
- **components/full/** — Expanded view: `Sidebar` (template selector, document list), `TranscriptPanel` (full transcript), `CopilotPanel` (resizable 280-700px, summary + advice).
- **components/settings/** — `AudioSettings`, `LLMSettings`, `ProfileSettings` tabs.

### Data Flow

```
User clicks Record
  -> start_recording command
  -> std::thread spawns cpal streams (mic + capture -> ring buffers)
  -> tokio::spawn Whisper loop (every 500ms):
       drain buffers -> mix audio -> silence check -> transcribe
       -> TranscriptStore.add() + emit "new-transcript"
  -> tokio::spawn Advisor loop:
       every 30s: full transcript -> LLM summary -> emit "meeting-summary"
       every 10s: recent text -> evaluate_triggers -> LLM advice -> emit "speaking-advice"

Frontend:
  useTauriEvents listens to events -> updates React state -> renders
```

### Meeting Templates

JSON files in `templates/` define meeting types. Each has:
- `system_prompt` — LLM system message for advice generation
- `trigger_hints` — keywords that trigger advice (e.g., "大家觉得怎么样")
- `advice_style` — style hint (e.g., "leadership")

Available: tech-review, code-review, brainstorm, project-sync. User can select active template in FullView sidebar.

## Key Details

- **LLM**: Default `https://llmgate.io/v1` with `gpt-5.4`. OpenAI-compatible chat completions API.
- **Whisper**: `ggml-medium.bin` (~1.5GB), language hardcoded to `zh`, 2-second audio chunks, `initial_prompt` carries previous transcription for context continuity.
- **Audio**: BlackHole 2ch virtual device for system audio capture. Mic + capture mixed (averaged) before transcription.
- **Config**: `~/.meeting-assistant/config.json` — LLM settings, audio device names, language preference.
- **History**: `~/.meeting-assistant/history.db` — SQLite, auto-saved on recording stop.
- **Models**: `~/.meeting-assistant/models/` — Whisper model files.
- **CSS**: Custom properties (`--bg-primary`, `--text-primary`, `--accent-purple`, etc.), not Tailwind theme tokens.
- **Port**: Vite dev server on 1420 (strict, required by Tauri).
- **Window**: Always-on-top, 420x840, resizable.
