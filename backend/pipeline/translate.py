"""Translation using Qwen2.5-32B MLX."""

import logging

logger = logging.getLogger("echo.translate")


def translate_if_needed(segments: list[dict], target_lang: str = "ru") -> list[dict]:
    """
    Переводит сегменты если язык != target_lang.

    Returns:
        Сегменты с переведённым text_ru
    """
    logger.info(f"Translation check for {len(segments)} segments")
    # TODO: Implement Qwen translation
    return segments
