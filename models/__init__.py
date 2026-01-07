"""
Email generation models package.
Provides Email, Attachment, FileGenerator, and ThreadGenerator classes.
"""

from .email import Email, Attachment, parse_display
from .file_generator import FileGenerator
from .thread_generator import ThreadGenerator, save_as_markdown

__all__ = [
    "Email",
    "Attachment",
    "parse_display",
    "FileGenerator",
    "ThreadGenerator",
    "save_as_markdown",
]
