"""API endpoints for Echo backend."""

import asyncio
import logging
from pathlib import Path

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel

logger = logging.getLogger("echo.api")

router = APIRouter()


class ProcessRequest(BaseModel):
    file_path: str


class ProcessResponse(BaseModel):
    status: str
    meeting_id: str | None = None
    message: str


@router.post("/process", response_model=ProcessResponse)
async def process_meeting(request: ProcessRequest):
    """
    Запускает pipeline обработки записи встречи:
    VAD → Whisper → Diarization → Speaker ID → Summary → Markdown
    """
    file_path = Path(request.file_path)

    if not file_path.exists():
        raise HTTPException(status_code=404, detail=f"File not found: {file_path}")

    logger.info(f"Starting pipeline for: {file_path}")

    # TODO: Implement pipeline
    # 1. VAD + chunking
    # 2. Whisper transcription
    # 3. Diarization
    # 4. Speaker identification
    # 5. Translation (if needed)
    # 6. Summary + bullets
    # 7. Save to SQLite + Markdown

    # Заглушка — имитируем обработку
    await asyncio.sleep(2)

    return ProcessResponse(
        status="completed",
        meeting_id="mock-123",
        message="Pipeline completed (stub)",
    )


@router.get("/meetings")
async def list_meetings():
    """Список обработанных встреч."""
    return {"meetings": []}


@router.get("/meetings/{meeting_id}")
async def get_meeting(meeting_id: str):
    """Детали конкретной встречи."""
    return {"meeting_id": meeting_id, "status": "not_found"}
