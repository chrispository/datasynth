"""
Email and Attachment data models.
"""

import uuid

from faker_instance import fake


def parse_display(display):
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
    
    def __init__(self, filename, filepath, content_type):
        self.id = str(uuid.uuid4())
        self.filename = filename
        self.filepath = filepath
        self.content_type = content_type

    def __repr__(self):
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
        sender,
        recipients,
        subject,
        body,
        date,
        message_id=None,
        parent_id=None,
        thread_id=None,
        msg_type="new",
    ):
        self.id = str(uuid.uuid4())
        self.message_id = (
            message_id if message_id else f"<{self.id}@{fake.free_email_domain()}>"
        )
        self.thread_id = thread_id if thread_id else str(uuid.uuid4())
        self.parent_id = parent_id

        self.sender = sender
        self.recipients = recipients
        self.cc = []
        self.bcc = []

        self.subject = subject
        self.body = body
        self.date = date

        self.type = msg_type

        self.in_reply_to = parent_id if parent_id else None
        self.references = []

        self.attachments = []

    def add_attachment(self, attachment):
        """Add an attachment to this email."""
        self.attachments.append(attachment)

    def __repr__(self):
        return (
            f"[{self.date.strftime('%Y-%m-%d %H:%M')}] "
            f"{self.sender} -> {', '.join(self.recipients)} | "
            f"{self.subject} ({self.type})"
        )
