"""Markdown generation for Obsidian."""

import logging
from datetime import datetime
from pathlib import Path

from jinja2 import Template

logger = logging.getLogger("echo.markdown")

MARKDOWN_TEMPLATE = """---
date: {{ date }}
time: "{{ time }}"
duration: "{{ duration }}"
speakers:
{% for speaker in speakers %}  - {{ speaker }}
{% endfor %}languages:
{% for lang in languages %}  - {{ lang }}
{% endfor %}tags:
  - meeting
---

# Созвон {{ date }}

## Участники
{% for speaker in speakers %}- {{ speaker }}
{% endfor %}

## Ключевые моменты
{% for bullet in bullets %}- {{ bullet }}
{% endfor %}

## Решения
| # | Решение | Ответственный | Срок |
|---|---------|---------------|------|
{% for decision in decisions %}| {{ loop.index }} | {{ decision.decision }} | {{ decision.owner }} | {{ decision.deadline }} |
{% endfor %}

## Итог
{{ summary }}

## Расшифровка
{% for segment in segments %}
**[{{ segment.timestamp }}] {{ segment.speaker }}:** {{ segment.text }}
{% endfor %}
"""


def generate_markdown(
    meeting_data: dict,
    output_path: Path,
) -> Path:
    """
    Генерирует Markdown файл для Obsidian.

    Args:
        meeting_data: данные встречи
        output_path: путь для сохранения

    Returns:
        Path к созданному файлу
    """
    logger.info(f"Generating markdown: {output_path}")

    template = Template(MARKDOWN_TEMPLATE)
    content = template.render(**meeting_data)

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(content, encoding="utf-8")

    return output_path
