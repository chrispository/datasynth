"""
Email and Attachment data models.
"""

import datetime
import uuid
from typing import Optional

from faker_instance import fake


def parse_display(display: str) -> dict[str, str]:
    """
    Parse a display string like 'Name <email>' into a dict.

    Args:
        display: String in format "Name <email@example.com>" or just "email@example.com"

    Returns:
        Dict with 'name' and 'email' keys
    """
    if " <" in display:
        name = display.split(" <")[0]
        email = display.split(" <")[1].rstrip(">")
        return {"name": name, "email": email}
    return {"name": display, "email": display}


class Attachment:
    """Represents an email attachment (PDF or DOCX)."""

    def __init__(self, filename: str, filepath: str, content_type: str) -> None:
        self.id: str = str(uuid.uuid4())
        self.filename = filename
        self.filepath = filepath
        self.content_type = content_type

    def __repr__(self) -> str:
        return f"<Attachment {self.filename}>"


class Email:
    """
    Represents an email message in a thread.

    Attributes:
        id: Unique identifier (UUID)
        message_id: SMTP-style message ID
        thread_id: Identifier for the conversation thread
        parent_id: Reference to parent email (for replies/forwards)
        sender: Sender display string
        recipients: List of recipient display strings
        cc: List of CC recipient display strings
        bcc: List of BCC recipient display strings
        subject: Email subject line
        body: Email body text
        date: Datetime of the email
        type: Message type (new, reply, forward)
        in_reply_to: Parent message ID
        references: List of ancestor message IDs
        attachments: List of Attachment objects
    """

    def __init__(
        self,
        sender: str,
        recipients: list[str],
        subject: str,
        body: str,
        date: datetime.datetime,
        message_id: Optional[str] = None,
        parent_id: Optional[str] = None,
        thread_id: Optional[str] = None,
        msg_type: str = "new",
    ) -> None:
        self.id: str = str(uuid.uuid4())
        self.message_id: str = (
            message_id if message_id else f"<{self.id}@{fake.free_email_domain()}>"
        )
        self.thread_id: str = thread_id if thread_id else str(uuid.uuid4())
        self.parent_id: Optional[str] = parent_id

        self.sender = sender
        self.recipients = recipients
        self.cc: list[str] = []
        self.bcc: list[str] = []

        self.subject = subject
        self.body = body
        self.date = date

        self.type = msg_type

        self.in_reply_to: Optional[str] = parent_id if parent_id else None
        self.references: list[str] = []

        self.attachments: list[Attachment] = []

    def add_attachment(self, attachment: Attachment) -> None:
        """Add an attachment to this email."""
        self.attachments.append(attachment)

    def __repr__(self) -> str:
        return (
            f"[{self.date.strftime('%Y-%m-%d %H:%M')}] "
            f"{self.sender} -> {', '.join(self.recipients)} | "
            f"{self.subject} ({self.type})"
        )
