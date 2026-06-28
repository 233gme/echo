"""Summary and bullet points using Qwen2.5-32B MLX."""

import logging

logger = logging.getLogger("echo.summary")


def generate_summary(transcript: str) -> dict:
    """
    Генерирует саммари и bullet points.

    Returns:
        {
            "summary": str,
            "bullets": [str],
            "decisions": [{"decision": str, "owner": str, "deadline": str}],
        }
    """
    logger.info("Generating summary...")
    # TODO: Implement Qwen summarization
    return {
        "summary": "",
        "bullets": [],
        "decisions": [],
    }
