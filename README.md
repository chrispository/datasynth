# gsyn

Synthetic email data generator with a terminal UI. Produces realistic, multi-party email threads with attachments for eDiscovery testing, ML training data, and email processing system validation.

Emails are generated as `.eml` files with proper threading semantics (replies, forwards, branching conversations), optionally using Google Gemini for realistic content. The pipeline includes PDF conversion and Bates numbering for legal workflows.

## Quick Start

### Prerequisites

- Rust toolchain
- Python 3
- Gemini API key (optional, for LLM-generated content)

### Setup

```bash
pip install -r requirements.txt
cargo build --release

cp .env.example .env
# Add your GEMINI_API_KEY to .env
```

### Run

```bash
cargo run
```

This launches the TUI where you can configure all generation parameters, run the pipeline, and convert output.

### CLI Usage

The Python scripts can also be run directly:

```bash
# Generate emails
python generate_emails.py \
  --files 25 \
  --attachments 30 \
  --topic "Quarterly Review" \
  --roster roster.json \
  --gemini \
  --model gemini-2.5-flash

# Convert .eml to PDF (with optional combine)
python convert_to_pdf.py --folder output/quarterly_review --combine

# Add Bates numbering
python bates_stamp.py --file output/quarterly_review_combined.pdf --prefix ROBOT
```

## Features

**TUI** -- Interactive terminal interface built with Ratatui. Includes model selection, topic management, company/roster generation, and settings persistence. Ships with five color themes (Default, Nord, Dracula, Gruvbox, Tokyo Night).

**Email Generation** -- Creates threaded email conversations between employees of two fictional companies. Thread actions are weighted: replies (80%), forwards (10%), and termination (10%). Forwards spawn new threads with different recipients. Attachments (PDF, DOCX) are generated and embedded at a configurable rate.

**PDF Conversion** -- Parses `.eml` files and renders them as PDFs with headers, body text, and quoted history. Optionally combines all emails in a topic folder into a single document.

**Bates Stamping** -- Overlays sequential reference numbers on combined PDFs with configurable prefix, separator, start number, and zero-padding.

## Project Structure

```
src/                    Rust TUI application
  main.rs               Entry point and event loop
  app.rs                Application state and logic
  ui.rs                 Rendering and themes

models/                 Python generation core
  thread_generator.py   Email thread simulation
  email.py              Email and attachment data models
  file_generator.py     PDF/DOCX attachment generation

generate_emails.py      Main generation entrypoint
convert_to_pdf.py       EML-to-PDF conversion
bates_stamp.py          Bates number overlay
roster.py               Company roster generation
llm.py                  Gemini LLM integration

topics.txt              Email topic list
companies.txt           Company name list
people.txt              Person name list
settings.json           Persisted UI configuration
```

## Configuration

Settings are persisted in `settings.json` and editable through the TUI. Key options:

- **Model** -- Gemini model variant (2.5-flash, 2.5-pro, 3-pro-preview, 3-flash-preview)
- **Quantity** -- Total file count and attachment percentage
- **Topics** -- Loaded from `topics.txt`, selectable in the UI
- **Companies** -- Two auto-generated companies with employee rosters
- **Bates** -- Prefix, separator, start number, and padding width

## License

All rights reserved.
