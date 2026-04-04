# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Meeting Assistant (Meeting Copilot) — a Tauri 2 desktop app that provides real-time meeting transcription via local Whisper model, with LLM-powered speaking advice and meeting summarization. The UI is designed as an always-on-top narrow panel (420x840) for use during meetings.

## Commands

```bash
# Frontend
pnpm install          # Install dependencies (use pnpm, not npm)
pnpm dev              # Vite dev server on localhost:1420
pnpm build            # TypeScript check + Vite build
pnpm typecheck        # TypeScript only

# Full app (Tauri + frontend)
pnpm tauri dev        # Dev mode with hot reload (first run compiles Rust ~5min)
pnpm tauri build      # Production build

# Rust backend only
cd src-tauri && cargo build
cd src-tauri && cargo test
cd src-tauri && cargo test -- --test-name  # Single test

# Rust tests exist in src-tauri/src/advisor/rules.rs
```

## Architecture

**Tauri 2 (Rust backend) + React 19 (frontend) + Vite 6 + Tailwind CSS 4**

### Backend (`src-tauri/src/`)

All Tauri commands are registered in `main.rs` and defined in `commands.rs`. The command layer is the single point of frontend-backend communication.

- **audio/** — Audio capture via `cpal`. Captures mic + system audio into separate ring buffers (`SharedBuffer`), resamples to 16kHz mono f32 for Whisper.
- **whisper/** — Local speech-to-text. `downloader.rs` fetches the ggml model, `engine.rs` wraps whisper-rs. Includes silence detection and hallucination filtering (CJK-aware).
- **transcript/** — `TranscriptStore` accumulates segments with timestamps. Shared via `Arc<Mutex<>>`.
- **advisor/** — LLM integration via OpenAI-compatible API (default: Ollama at localhost:11434). `engine.rs` generates summaries and speaking advice. `rules.rs` evaluates trigger conditions (keyword hints, questions, silence pauses). `templates.rs` manages meeting type templates.
- **storage/** — `config.rs` persists `AppConfig` to `~/.meeting-assistant/config.json`. `history.rs` uses SQLite for meeting records.
- **documents/** — Document loader for reference materials during meetings.

### Frontend (`src/`)

- **lib/tauri.ts** — All `invoke()` calls and `listen()` event subscriptions in one file. This is the only file that imports from `@tauri-apps/api`.
- **lib/types.ts** — Shared TypeScript types mirroring Rust structs.
- **hooks/** — `useRecording.ts` (recording lifecycle), `useTauriEvents.ts` (event subscriptions).
- **components/narrow/** — Primary meeting view (compact panel): control bar, transcript mini, advice panel, summary panel.
- **components/full/** — Expanded view: sidebar, full transcript panel, copilot panel.
- **components/settings/** — Audio, LLM, and profile configuration.
- **components/shared/** — Reusable: `SetupGuide` (Whisper model download), `AdviceCard`.

### Data Flow

1. `start_recording` → spawns audio capture streams → ring buffers → Whisper transcription loop (500ms intervals)
2. Whisper results → `TranscriptStore` + emits `new-transcript` event to frontend
3. Advisor loop (parallel): every 30s generates summary (`meeting-summary` event), every 10s checks triggers and generates advice (`speaking-advice` event)
4. Frontend listens to events via `useTauriEvents` hook

### Meeting Templates

JSON files in `templates/` define meeting types (tech-review, code-review, brainstorm, project-sync). Each template has a `system_prompt`, `trigger_hints` (keywords that trigger advice), and `advice_style`.

## Key Details

- Default LLM config points to Ollama (`localhost:11434/v1`) with `llama3.2` model
- Whisper is hardcoded to Chinese (`set_language(Some("zh"))`)
- Audio buffer: 3-second ring buffer at 16kHz sample rate
- App config stored at `~/.meeting-assistant/config.json`
- CSS uses CSS custom properties (`--bg-primary`, `--text-primary`, etc.) not Tailwind theme
- Vite dev server runs on port 1420 (strict, required by Tauri)
