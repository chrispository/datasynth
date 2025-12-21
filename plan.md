# Synthetic Email Data Generator Plan

## 1. Objective
Build a robust synthetic data generator for emails and attachments, emphasizing realistic threading, replying, forwarding, and branching logic. The system will produce dataset artifacts (e.g., EML files, JSON exports) containing complex email chains and associated PDF/Word attachments.

## 2. Data Model

An email will be modeled as a node in a directed graph.

### Core Attributes
- **ID:** Unique identifier (UUID).
- **Message-ID:** SMTP standard unique identifier.
- **Thread-ID:** Identifier for the conversation thread.
- **Parent-ID:** Reference to the email being replied to or forwarded (for tree reconstruction).
- **Headers:**
  - `From`: Sender email & name.
  - `To`, `Cc`, `Bcc`: Lists of recipients.
  - `Date`: Timestamp (must be strictly sequential within a chain).
  - `Subject`: Subject line (evolving with Re/Fwd).
  - `In-Reply-To`: Message-ID of the parent.
  - `References`: List of Message-IDs in the ancestry.
- **Body:**
  - `Content`: The new text added in this specific message.
  - `Quoted_Block`: The historical text from previous emails (essential for realistic parsing).
- **Attachments:** List of file objects (PDF/DOCX).
- **Type:** `New`, `Reply`, `Reply-All`, `Forward`.

## 3. Threading & Logic

### A. The "Chain" vs. The "Branch"
- **Linear Chain:** A simple back-and-forth (A -> B -> A).
- **Branching:**
  - *Multiple Replies:* Person A sends an email. Person B replies. Person C also replies to Person A (separate from B). This creates a fork.
  - *Forwarding:* Person B forwards Person A's email to Person D. This starts a new "branch" or potentially a new `Thread-ID` depending on the client behavior, but technically links back to the original content.

### B. Reply Logic (`Re:`)
1.  **Headers:**
    -   Append `Re:` to Subject (if not already present). *Edge Case: Handle `Re[2]:` style.*
    -   Set `To` to the original sender.
    -   Set `Cc` based on `Reply` vs `Reply-All`.
    -   Add Parent's `Message-ID` to `In-Reply-To` and append to `References`.
2.  **Body Construction:**
    -   Generate new response text.
    -   Append "On [Date], [Sender] wrote:" or standard quote headers.
    -   Append the parent's body, usually prefixed with `>` characters.
3.  **Attachments:** Usually *dropped* in replies, unless explicitly re-attached (rare).

### C. Forward Logic (`Fwd:`)
1.  **Headers:**
    -   Append `Fwd:` to Subject.
    -   Recipients (`To`) are completely new/different.
    -   References might be preserved to link context, but often treated as a fresh start by some parsers.
2.  **Body Construction:**
    -   Generate "Here is the info you asked for..." type text.
    -   Include a "---------- Forwarded message ----------" delimiter.
    -   Include the original email's header block (From, Date, Subject, To).
    -   Include the original body.
3.  **Attachments:**
    -   **Crucial:** Attachments from the original email must be preserved and carried over to the new email.
    -   New attachments can also be added.

## 4. Enhanced Content Generation

### A. Company Rosters
To ensure realistic interactions, the generator uses a `RosterGenerator`.
- **Roster Attributes:** Name, Email, Title, Department.
- **Organization:** Employees are grouped by department, with hierarchical titles.
- **Persistence:** Rosters are saved to `roster.json` for consistency across runs.

### B. LLM Integration (Google Gemini)
For realistic email bodies and subjects, the generator integrates with Google Gemini.
- **Usage:** Gemini generates content based on the sender's role, recipient list, topic, and thread context.
- **Fallback:** If Gemini is unavailable or disabled, the generator falls back to `Faker` and template-based logic.
- **Configuration:** API keys are stored in a `.env` file.

## 5. Attachment Generation Strategy

We will avoid Excel files as requested and focus on PDF and Word.

### Tools
-   **PDF:** `reportlab` or `fpdf2` (Python).
-   **Word:** `python-docx`.

### Logic
-   **Contextual Naming:** Filenames should relate to the email subject (e.g., Subject: "Invoice Q3", File: "Invoice_Q3_2024.pdf").
-   **Content:**
    -   Generate random lorem ipsum or structured text (e.g., "Invoice", "Contract", "Meeting Notes").
    -   Embed metadata to match the creation date of the email.

## 5. Edge Cases & Complexity

1.  **Broken Chains:** An email in the middle is "missing" from the dataset (simulating data loss), but the `References` header in the child still points to it.
2.  **Date Paradoxes:** ensuring child emails always have a timestamp *after* the parent.
3.  **Subject Line Mutation:**
    -   "Re: Re: Fwd: Meeting" (Messy subjects).
    -   User manually changing the subject in a reply (breaks standard threading in some clients).
4.  **Mixed Clients:**
    -   HTML vs Plain Text bodies.
    -   Top-posting (standard corporate) vs Bottom-posting (old school lists).
5.  **Self-Replies:** Sender replying to their own email to add more info.
6.  **Looping:** (Rare but possible in automated systems) A auto-replies to B, B auto-replies to A. (We should probably prevent infinite loops but simulate short bursts).

## 6. Implementation Plan

1.  **Setup:** Initialize Python project with `faker`, `python-docx`, `fpdf2`.
2.  **Generator Class:** Create a `ThreadGenerator` class.
    -   Method: `create_root_email()`
    -   Method: `reply_to(email_node)`
    -   Method: `forward(email_node)`
3.  **Simulation Loop:**
    -   Start with $N$ root threads.
    -   Iteratively pick a leaf node and apply an action (Reply, Reply-All, Forward, Ignore) based on probability weights.
4.  **Renderer:**
    -   Convert the internal objects to standard `.eml` (MIME) format or a structured JSON export.
    -   Generate physical attachment files on disk and link them.

## 7. Output Example (JSON structure)

```json
{
  "id": "msg-102",
  "thread_id": "thread-5",
  "type": "reply",
  "headers": {
    "from": "alice@example.com",
    "to": ["bob@example.com"],
    "subject": "Re: Project Alpha",
    "date": "2023-10-27T10:05:00Z",
    "in_reply_to": "msg-101"
  },
  "body": "Sounds good.\n\n> On Oct 27, Bob wrote: ...",
  "attachments": []
}
```