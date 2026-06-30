# Echo — Meeting Assistant

Local assistant for recording and processing meetings.

## Architecture

```
┌─────────────┐     HTTP      ┌─────────────┐
│  Rust App   │ ◀───────────▶ │  Python     │
│  (System    │               │  Backend    │
│   Tray)     │               │  (FastAPI)  │
└─────────────┘               └─────────────┘
       │                            │
       ▼                            ▼
~/.meeting_assistant/         ~/Obsidian/Meetings/
```

## Structure

| Folder     | Purpose                                      |
| ---------- | -------------------------------------------- |
| `src/`     | Rust application (system tray, audio capture) |
| `backend/` | Python backend (VAD, Whisper, LLM)           |
| `scripts/` | Launch scripts                               |

## Launch

### Option 1: Script (both services)

```bash
cd ~/Projects/echo
./scripts/start.sh
```

### Option 2: Manual launch

Terminal 1 — Backend:

```bash
cd backend
source venv/bin/activate
uvicorn main:app --reload
```

Terminal 2 — Rust:

```bash
cargo run
```

## Hotkeys

| Key       | Action                    |
| --------- | ------------------------- |
| `⌘+⇧+R`   | Start/stop recording      |
| `⌘+⇧+O`   | Open Obsidian             |

## Requirements

- macOS 12.3+ (ScreenCaptureKit)
- Apple Silicon (MLX)
- Python 3.10+
- Rust 1.75+

## Pipeline

1. **Recording** — ScreenCaptureKit → `recordings/`
2. **VAD** — Silero (chunk splitting on silence)
3. **Transcription** — Whisper MLX large-v3
4. **Diarization** — pyannote-3.1
5. **Speaker ID** — ECAPA-TDNN ("Me" vs "Other")
6. **Translation** — Qwen2.5-32B (if needed)
7. **Summary** — Qwen2.5-32B
8. **Markdown** → Obsidian
