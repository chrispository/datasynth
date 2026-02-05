"""Shared utility helpers for the synthetic email generator."""

import re


def sanitize_filename(text: str, max_length: int | None = None) -> str:
    """Replace non-alphanumeric characters with underscores.

    Args:
        text: Raw text to sanitize for use in file/folder names.
        max_length: If provided, truncate the result to this many characters.

    Returns:
        A filesystem-safe string.
    """
    cleaned = "".join(c if c.isalnum() or c == "_" else "_" for c in text)
    if max_length is not None:
        cleaned = cleaned[:max_length]
    return cleaned


def strip_markdown(text: str) -> str:
    """Basic markdown-to-plaintext cleanup.

    Removes bold markers and converts ``* `` list items to bullet points.

    Args:
        text: Markdown-formatted string.

    Returns:
        Cleaned plain-text string.
    """
    text = text.replace("**", "")
    text = re.sub(r"^\* ", "\u2022 ", text, flags=re.MULTILINE)
    return text
