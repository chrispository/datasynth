"""
Thread generator for simulating realistic email threads.
"""

import os
import random
import datetime
import logging
from faker import Faker

from .email import Email, Attachment, parse_display
from .file_generator import FileGenerator

fake = Faker()


class ThreadGenerator:
    """
    Generates realistic email threads with replies, forwards, and attachments.
    
    Simulates multi-party email conversations with proper threading,
    quoting, and reference chains.
    
    Email Threading Semantics (eDiscovery):
    - A "thread" is a group of related emails sharing a thread_id
    - Replies stay in the same thread as their parent
    - Forwards START A NEW THREAD (new thread_id) since they introduce new recipients
    - An "inclusive" email is a leaf node: no other email replies to it
    - Branching threads (A->B, A->C) produce multiple inclusive emails (B and C)
    - Each inclusive email contains the full quoted history of its branch
    """
    
    def __init__(
        self,
        roster=None,
        llm=None,
        start_date=None,
        output_dir="output",
        topic=None,
        attachment_percent=30,
    ):
        self.emails = []  # Flat list of all emails generated
        self.threads = {}  # Map thread_id -> list of Email objects
        self._replied_parent_ids = set()  # Track messages that have been replied to
        self._used_subjects = []  # Track subjects to avoid duplicates
        self.current_date = (
            start_date
            if start_date
            else datetime.datetime.now() - datetime.timedelta(days=30)
        )
        self.roster = (
            roster
            if roster
            else [
                {
                    "name": fake.name(),
                    "email": fake.email(),
                    "title": "Employee",
                    "department": "General",
                }
                for _ in range(10)
            ]
        )
        self.llm = llm
        self.file_gen = FileGenerator(output_dir, llm=llm, topic=topic)
        self.topic = topic
        self.attachment_percent = attachment_percent / 100.0

    def _tick_time(self):
        """Advance time by a random increment."""
        increment = datetime.timedelta(minutes=random.randint(1, 120))
        self.current_date += increment
        return self.current_date

    def get_person_display(self, person):
        """Format a person dict as 'Name <email>'."""
        return f"{person['name']} <{person['email']}>"

    def _get_thread_participants(self, thread_id):
        """Get all unique participants (senders and recipients) from a thread."""
        participants = set()
        if thread_id in self.threads:
            for email in self.threads[thread_id]:
                participants.add(email.sender)
                participants.update(email.recipients)
        return list(participants)

    def _can_forward_to_new_recipients(self, thread_id):
        """Check if there are roster members not in the current thread."""
        return len(self._get_available_recipients(thread_id)) > 0

    def _get_available_recipients(self, thread_id):
        """Get roster members not yet in the thread - useful for branching."""
        thread_participants = self._get_thread_participants(thread_id)
        participant_emails = set()
        for p in thread_participants:
            if " <" in p:
                participant_emails.add(p.split(" <")[1].rstrip(">"))
            else:
                participant_emails.add(p)

        return [p for p in self.roster if p["email"] not in participant_emails]

    def _has_reply(self, message_id):
        """Check if an email has already been replied to."""
        return message_id in self._replied_parent_ids

    def create_root_email(self):
        """Create a new root email starting a fresh thread."""
        sender = random.choice(self.roster)
        recipients = random.sample(
            [p for p in self.roster if p != sender], k=random.randint(1, 3)
        )

        subject = None
        body = None

        if self.llm:
            subject, body = self.llm.generate_email(
                sender, recipients, self.topic if self.topic else "General check-in",
                used_subjects=self._used_subjects,
            )

        if not body:
            if self.topic:
                subject_start = random.choice(
                    [
                        "Regarding",
                        "Update on",
                        "Question about",
                        "Notes for",
                        "Discussion:",
                    ]
                )
                subject = f"{subject_start} {self.topic}"
                body_start = f"Hi all,\n\nI wanted to discuss {self.topic}."
                body = f"{body_start}\n\n" + fake.paragraph(nb_sentences=5)
            else:
                subject = fake.sentence(nb_words=4).rstrip(".")
                body = fake.paragraph(nb_sentences=5)

        # Dedup fallback: if subject exactly matches an existing one, add suffix
        if subject and subject in self._used_subjects:
            suffix = random.choice([
                "- Follow Up", "- Continued", "- Revisited",
                "- Additional Thoughts", "- Part II",
            ])
            subject = f"{subject} {suffix}"

        if subject:
            self._used_subjects.append(subject)

        email = Email(
            sender=self.get_person_display(sender),
            recipients=[self.get_person_display(r) for r in recipients],
            subject=subject,
            body=body,
            date=self._tick_time(),
            msg_type="new",
        )

        self._store_email(email)
        return email

    def reply_to(self, parent_email, reply_all=True):
        """
        Create a reply to an existing email, staying in the SAME thread.
        
        The reply quotes the parent's full body, creating a nested conversation.
        If this reply is not replied to, it becomes an "inclusive" email
        (leaf node containing the full thread history for its branch).
        """
        parent_recipients = [parse_display(r) for r in parent_email.recipients]
        parent_sender = parse_display(parent_email.sender)

        # Find the person in the roster that matches a recipient
        sender_info = random.choice(parent_recipients)
        # Try to find their full info in roster
        roster_sender = next(
            (p for p in self.roster if p["email"] == sender_info["email"]), None
        )
        if not roster_sender:
            roster_sender = {
                "name": sender_info["name"],
                "email": sender_info["email"],
                "title": "Employee",
                "department": "General",
            }

        # Primary recipient is always the original sender
        recipients = [parent_sender]
        roster_recipients = [
            next((p for p in self.roster if p["email"] == r["email"]), r)
            for r in recipients
        ]

        # Reply-all: CC other original recipients (excluding the new sender)
        cc_recipients = []
        if reply_all and len(parent_recipients) > 1:
            for r in parent_recipients:
                if r["email"] != sender_info["email"]:
                    cc_recipients.append(r)

        subject = parent_email.subject
        if not subject.lower().startswith("re:"):
            subject = f"Re: {subject}"

        new_body = None
        if self.llm:
            _, new_body = self.llm.generate_email(
                roster_sender, roster_recipients, self.topic, context=parent_email.body
            )

        if not new_body:
            new_body = fake.paragraph(nb_sentences=3)
            if self.topic and random.random() < 0.2:
                new_body += f"\n\nRegarding the {self.topic} aspect, I agree."

        # Recursively quote the ENTIRE parent body, indented
        parent_body_lines = parent_email.body.split("\n")
        quoted_lines = [f"> {line}" for line in parent_body_lines]
        quoted_block = "\n".join(quoted_lines)

        full_body = f"{new_body}\n\nOn {parent_email.date.strftime('%Y-%m-%d %H:%M')}, {parent_email.sender} wrote:\n{quoted_block}"

        email = Email(
            sender=self.get_person_display(roster_sender),
            recipients=[
                self.get_person_display(r)
                if isinstance(r, dict) and "email" in r
                else r
                for r in recipients
            ],
            subject=subject,
            body=full_body,
            date=self._tick_time(),
            message_id=None,
            parent_id=parent_email.message_id,
            thread_id=parent_email.thread_id,
            msg_type="reply",
        )

        # Add CC recipients for reply-all
        email.cc = [f"{r['name']} <{r['email']}>" for r in cc_recipients]

        # Handle Headers
        email.references = parent_email.references + [parent_email.message_id]

        self._store_email(email)
        return email

    def forward(self, parent_email):
        """
        Forward an email to new recipients, starting a NEW thread.
        
        In eDiscovery terms, a forward starts a new thread because:
        1. It introduces new recipients who weren't part of the original conversation
        2. The forwarder typically adds new commentary/context
        3. The forwarded thread may spawn its own separate reply chain
        
        The forwarded email contains the parent's full body as quoted content,
        making it "inclusive" of the original thread's content.
        """
        import uuid
        
        # Pick sender from thread participants for realism
        thread_participants = self._get_thread_participants(parent_email.thread_id)

        # Find roster entries for participants
        participant_emails = [parse_display(p)["email"] for p in thread_participants]
        roster_participants = [
            p for p in self.roster if p["email"] in participant_emails
        ]

        if roster_participants:
            sender = random.choice(roster_participants)
        else:
            sender = random.choice(self.roster)

        # Forward to someone NOT in the original thread
        potential_recipients = [
            p
            for p in self.roster
            if self.get_person_display(p) not in thread_participants
        ]
        if not potential_recipients:
            potential_recipients = [random.choice(self.roster)]
        recipients = [random.choice(potential_recipients)]

        subject = parent_email.subject
        if not subject.lower().startswith("fwd:"):
            subject = f"Fwd: {subject}"

        new_body = None
        if self.llm:
            _, new_body = self.llm.generate_email(
                sender,
                recipients,
                f"Forwarding: {self.topic if self.topic else parent_email.subject}",
                context=parent_email.body,
            )

        if not new_body:
            new_body = f"FYI."
            if self.topic and random.random() < 0.3:
                new_body = (
                    f"Thought you should see this regarding {self.topic}.\n\n"
                    + new_body
                )

        # Include the full parent body as quoted forwarded content
        forward_block = f"---------- Forwarded message ----------\nFrom: {parent_email.sender}\nDate: {parent_email.date}\nSubject: {parent_email.subject}\nTo: {', '.join(parent_email.recipients)}\n\n{parent_email.body}"

        full_body = f"{new_body}\n\n{forward_block}"

        # IMPORTANT: Forward starts a NEW thread (new thread_id)
        # This is correct eDiscovery behavior since forwards introduce
        # new recipients and may spawn their own reply chains
        new_thread_id = str(uuid.uuid4())

        email = Email(
            sender=self.get_person_display(sender),
            recipients=[self.get_person_display(r) for r in recipients],
            subject=subject,
            body=full_body,
            date=self._tick_time(),
            message_id=None,
            parent_id=parent_email.message_id,  # Still references parent for traceability
            thread_id=new_thread_id,  # NEW thread
            msg_type="forward",
        )

        # Keep references chain for full traceability back to original
        email.references = parent_email.references + [parent_email.message_id]

        # Carry over attachments from parent
        for att in parent_email.attachments:
            email.add_attachment(att)

        self._store_email(email)
        return email

    def _store_email(self, email):
        """Store an email and update thread tracking."""
        self.emails.append(email)
        if email.thread_id not in self.threads:
            self.threads[email.thread_id] = []
        self.threads[email.thread_id].append(email)

        # Track that the parent has been replied to (prevents branching)
        if email.parent_id:
            self._replied_parent_ids.add(email.parent_id)

        inclusive_count = self._count_inclusive_emails()
        logging.info(
            f"  [Progress] Total emails: {len(self.emails)} | Inclusive emails: {inclusive_count}"
        )

    def _count_inclusive_emails(self):
        """
        Count "inclusive" emails (leaf nodes in the thread tree).
        
        An inclusive email is one that is NOT the parent of any other email.
        In eDiscovery terms, inclusive emails are the most comprehensive
        versions of their conversation branch - they contain all quoted
        history and are not superseded by any reply.
        
        Example thread tree:
            A (root)
            ├── B (reply to A)
            │   └── C (reply to B) <- INCLUSIVE (no replies)
            └── D (reply to A) <- INCLUSIVE (no replies)
        
        This thread has 2 inclusive emails: C and D.
        """
        parent_message_ids = set()
        for email_obj in self.emails:
            if email_obj.parent_id:
                parent_message_ids.add(email_obj.parent_id)
        return sum(1 for e in self.emails if e.message_id not in parent_message_ids)

    def simulate(
        self, target_inclusive=5, max_emails_per_thread=9, early_end_chance=0.15
    ):
        """Simulate multiple email threads until we have target_inclusive leaf emails."""
        logging.info(f"Simulation started. Target: {target_inclusive} inclusive emails.")

        while self._count_inclusive_emails() < target_inclusive:
            # Create a new thread with a root email
            self.create_root_email()

            # Get the thread we just created
            tid = list(self.threads.keys())[-1]
            thread_msgs = self.threads[tid]

            # Determine this thread's target length
            thread_target_length = random.randint(2, max_emails_per_thread)

            # Build this thread
            while (
                self._count_inclusive_emails() < target_inclusive
                and len(thread_msgs) < thread_target_length
            ):
                # Refresh thread_msgs reference
                thread_msgs = self.threads[tid]

                # Check if thread should end early
                if len(thread_msgs) >= 2 and random.random() < early_end_chance:
                    break

                # Pick a message to reply to - always use most recent unreplied message
                # to ensure linear threads (no branching with same recipients)
                unreplied_msgs = [m for m in thread_msgs if not self._has_reply(m.message_id)]
                if not unreplied_msgs:
                    break  # All messages have replies, thread is complete
                
                parent = unreplied_msgs[-1]  # Most recent unreplied message

                # Action weights: 80% reply, 10% forward, 10% skip
                action = random.choices(
                    ["reply", "forward", "nothing"], weights=[0.8, 0.1, 0.1]
                )[0]

                # Convert forward to reply if no new recipients available
                if action == "forward" and not self._can_forward_to_new_recipients(tid):
                    action = "reply"

                if action == "reply":
                    self.reply_to(parent)
                elif action == "forward":
                    self.forward(parent)
        
        logging.info("Simulation complete.")


def save_as_markdown(email_obj, output_dir="output", index=0):
    """
    Save an email as a markdown file with proper naming for sort order.
    
    Uses 'a' suffix for emails and 'b' suffix for attachments to ensure
    alphabetical sorting places emails before their attachments.
    
    Args:
        email_obj: Email object to save
        output_dir: Directory to write files to
        index: Numeric index for file ordering
    
    Returns:
        Path to the saved markdown file
    """
    import shutil
    
    # Create safe subject for filename
    safe_subject = "".join([c if c.isalnum() else "_" for c in email_obj.subject])[:40]
    # Use 'a' suffix for emails to sort before attachments
    filename = f"{index:04d}a_{safe_subject}.md"
    filepath = os.path.join(output_dir, filename)

    # Handle Attachments (move/rename them with 'b' suffix)
    att_list = []
    for att in email_obj.attachments:
        if os.path.exists(att.filepath):
            att_ext = os.path.splitext(att.filename)[1]
            att_base = os.path.splitext(att.filename)[0]
            # Use 'b' suffix for attachments to sort after email
            new_att_name = f"{index:04d}b_{att_base}{att_ext}"
            new_att_path = os.path.join(output_dir, new_att_name)

            if not os.path.exists(new_att_path) and os.path.exists(att.filepath):
                shutil.copy(att.filepath, new_att_path)

            att_list.append(new_att_name)
        else:
            att_list.append(att.filename)

    with open(filepath, "w", encoding="utf-8") as f:
        f.write(f"**From:** {email_obj.sender}\n")
        f.write(f"**Date:** {email_obj.date.strftime('%A, %B %d, %Y %I:%M %p')}\n")
        f.write(f"**To:** {', '.join(email_obj.recipients)}\n")
        if email_obj.cc:
            f.write(f"**Cc:** {', '.join(email_obj.cc)}\n")
        f.write(f"**Subject:** {email_obj.subject}\n")

        if att_list:
            f.write(f"**Attachments:** {', '.join(att_list)}\n")

        f.write("\n---\n\n")
        f.write(email_obj.body)

    return filepath
