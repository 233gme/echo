"""Whisper transcription using MLX (Apple Silicon optimized)."""

import logging
from pathlib import Path

logger = logging.getLogger("echo.whisper")


def transcribe(audio_path: Path, model: str = "large-v3") -> list[dict]:
    """
    Транскрибирует аудио в текст с таймкодами.

    Returns:
        [{"start": float, "end": float, "text": str, "language": str}]
    """
    logger.info(f"Transcribing: {audio_path} with model {model}")
    # TODO: Implement mlx-whisper
    return []
