"""Speaker identification using ECAPA-TDNN."""

import logging
from pathlib import Path

logger = logging.getLogger("echo.speaker_id")


def identify_speakers(
    diarization: list[dict],
    reference_path: Path | None = None,
    threshold: float = 0.85,
) -> list[dict]:
    """
    Определяет "Я" vs "Собеседник" по reference sample.

    Args:
        diarization: результат diarization
        reference_path: путь к образцу голоса пользователя
        threshold: cosine similarity threshold

    Returns:
        [{"start": float, "end": float, "speaker": "Я" | "Собеседник"}]
    """
    logger.info(f"Speaker ID with reference: {reference_path}")
    # TODO: Implement ECAPA-TDNN
    return []
