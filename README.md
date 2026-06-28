# Echo — Meeting Assistant

Локальный ассистент для записи и обработки встреч.

## Архитектура

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

## Структура

| Папка      | Назначение                                  |
| ---------- | ------------------------------------------- |
| `src/`     | Rust приложение (system tray, захват аудио) |
| `backend/` | Python backend (VAD, Whisper, LLM)          |
| `scripts/` | Скрипты запуска                             |

## Запуск

### Вариант 1: Скрипт (оба сервиса)

```bash
cd ~/Projects/echo
./scripts/start.sh
```

### Вариант 2: Ручной запуск

Терминал 1 — Backend:

```bash
cd backend
source venv/bin/activate
uvicorn main:app --reload
```

Терминал 2 — Rust:

```bash
cargo run
```

## Горячие клавиши

| Клавиша | Действие                 |
| ------- | ------------------------ |
| `⌘+⇧+R` | Начать/остановить запись |
| `⌘+⇧+O` | Открыть Obsidian         |

## Требования

- macOS 12.3+ (ScreenCaptureKit)
- Apple Silicon (MLX)
- Python 3.10+
- Rust 1.75+

## Pipeline

1. **Запись** — ScreenCaptureKit → `recordings/`
2. **VAD** — Silero (разбиение на чанки)
3. **Транскрибация** — Whisper MLX large-v3
4. **Диаризация** — pyannote-3.1
5. **Speaker ID** — ECAPA-TDNN ("Я" vs "Собеседник")
6. **Перевод** — Qwen2.5-32B (если нужно)
7. **Саммари** — Qwen2.5-32B
8. **Markdown** → Obsidian
