"""Voice Activity Detection using Silero VAD."""

import logging
from pathlib import Path

logger = logging.getLogger("echo.vad")


def split_on_silence(
    audio_path: Path,
    min_chunk_sec: float = 5.0,
    max_chunk_sec: float = 15.0,
    silence_threshold_sec: float = 1.0,
):
    """
    Разбивает аудио на чанки по паузам.

    Args:
        audio_path: путь к аудиофайлу
        min_chunk_sec: минимальная длина чанка
        max_chunk_sec: максимальная длина чанка
        silence_threshold_sec: порог тишины для разреза

    Returns:
        list of (start_sec, end_sec, audio_segment)
    """
    logger.info(f"VAD processing: {audio_path}")
    # TODO: Implement Silero VAD
    return []
