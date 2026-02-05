# datasynth

Synthetic email data generator with a terminal UI. Produces realistic, multi-party email threads with attachments for eDiscovery testing, ML training data, and email processing system validation.

Emails are generated as `.eml` files with proper threading semantics (replies, forwards, branching conversations) using LLM-generated content via Google Gemini or OpenRouter. The pipeline includes PDF conversion and Bates numbering for legal workflows.

## Quick Start

### Prerequisites

- Rust toolchain
- Python 3
- API key for at least one provider:
  - **Gemini** -- `GEMINI_API_KEY`
  - **OpenRouter** -- `OPENROUTER_API_KEY`

### Setup

```bash
pip install -r requirements.txt
cargo build --release

cp .env.example .env
# Add your API key(s) to .env
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

**TUI** -- Interactive terminal interface built with Ratatui. Includes provider/model selection, topic management, company/roster generation, prompt preview, and settings persistence. Ships with five color themes (Default, Nord, Dracula, Gruvbox, Tokyo Night).

**LLM Providers** -- Supports Google Gemini and OpenRouter. Toggle between providers in the TUI with `Tab`. OpenRouter currently exposes `moonshotai/kimi-k2`; Gemini exposes 2.5-flash, 2.5-pro, 3-pro-preview, and 3-flash-preview. Each email is generated with a randomly selected writing style (direct, formal, casual, urgent, inquisitive, collaborative, apologetic, enthusiastic).

**Prompt Preview** -- The TUI includes a prompt preview screen that shows the full thread generation logic, a sample email prompt built from your current settings (selected topic, company roster), and the active provider/model.

**Email Generation** -- Creates threaded email conversations between employees of two fictional companies. Thread actions are weighted: replies (default 80%), forwards (10%), and end-of-chain (10%). At each step the generator rolls against these weights -- a reply continues the current thread, a forward spawns a new thread with different recipients, and end-of-chain means no further response is generated for that branch. Attachments (PDF, DOCX) are generated and embedded at a configurable rate.

**PDF Conversion** -- Parses `.eml` files and renders them as PDFs using DejaVuSans (bundled in `fonts/`) with headers, body text, and quoted history. Optionally combines all emails in a topic folder into a single document.

**Bates Stamping** -- Overlays sequential reference numbers on combined PDFs with configurable prefix, separator, start number, and zero-padding.

## TUI Workflow

1. **Select a provider and model** -- Use `Tab` to toggle between Gemini and OpenRouter, arrow keys to pick a model.
2. **Load topics** -- Go to the Topics screen and press Enter to pull 25 random topics from `topics.txt` (309 total). Select which topics to generate.
3. **Generate companies** -- Use the Companies screen to generate two fictional companies with employee rosters sourced from `companies.txt` and `people.txt`.
4. **Configure** -- Set file count, attachment percentage, reply/forward/end-of-chain weights, and Bates options.
5. **Preview prompt** -- Check the Prompt Preview screen to see exactly what will be sent to the LLM.
6. **Generate** -- Run the pipeline. Output lands in `output/<topic_name>/`.
7. **Convert & stamp** -- Convert to PDF and apply Bates numbering from within the TUI.

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
llm.py                  Gemini & OpenRouter LLM integration

fonts/                  Bundled fonts (DejaVuSans) for PDF rendering
topics.txt              Email topic list (309 topics)
companies.txt           Company name list
people.txt              Person name list
settings.json           Persisted UI configuration
```

## Configuration

Settings are persisted in `settings.json` and editable through the TUI. Key options:

- **Provider** -- Gemini or OpenRouter (toggle with `Tab`)
- **Model** -- Provider-specific model list (Gemini: 2.5-flash, 2.5-pro, 3-pro-preview, 3-flash-preview; OpenRouter: moonshotai/kimi-k2)
- **Quantity** -- Total file count and attachment percentage
- **Topics** -- 25 random topics loaded from `topics.txt`, selectable in the UI
- **Companies** -- Two auto-generated companies with employee rosters from `companies.txt` and `people.txt`
- **Thread weights** -- Reply, forward, and end-of-chain percentages
- **Bates** -- Prefix, separator, start number, and padding width

## License

All rights reserved.
