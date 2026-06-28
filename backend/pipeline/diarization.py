"""Speaker diarization using pyannote.audio."""

import logging
from pathlib import Path

logger = logging.getLogger("echo.diarization")


def diarize(audio_path: Path) -> list[dict]:
    """
    Определяет кто говорит и когда.

    Returns:
        [{"start": float, "end": float, "speaker": str}]
    """
    logger.info(f"Diarization: {audio_path}")
    # TODO: Implement pyannote-3.1
    return []
