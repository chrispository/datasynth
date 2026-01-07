#!/usr/bin/env python3
"""
Synthetic Email Data Generator

Generates realistic email threads with attachments for testing and training data.
Uses LLM (Gemini) for content generation when available, with faker-based fallbacks.
"""

import os
import sys
import random
import datetime
import argparse
import traceback
from dotenv import load_dotenv

from models import ThreadGenerator, Attachment, save_as_markdown
from roster import RosterGenerator
from llm import GeminiGenerator


def main():
    load_dotenv()

    try:
        print("Starting generator...", flush=True)
        parser = argparse.ArgumentParser(
            description="Generate synthetic email threads with attachments"
        )
        parser.add_argument(
            "--files",
            type=int,
            default=5,
            help="Number of inclusive email threads to generate",
        )
        parser.add_argument(
            "--steps", type=int, default=None, help="Deprecated, use --files instead"
        )
        parser.add_argument(
            "--output", type=str, default="output", help="Output directory"
        )
        parser.add_argument("--topic", type=str, default=None, help="Topic to focus on")
        parser.add_argument(
            "--attachments",
            type=int,
            default=30,
            help="Percentage of emails with attachments (0-100)",
        )
        parser.add_argument(
            "--roster", type=str, default="roster.json", help="Path to roster file"
        )
        parser.add_argument(
            "--gemini", action="store_true", help="Use Gemini LLM for email generation"
        )
        parser.add_argument(
            "--model", type=str, default="gemini-2.5-flash", help="Gemini model to use"
        )
        # Kept for compatibility but ignored
        parser.add_argument("--pdf", action="store_true", help="Ignored")
        args = parser.parse_args()

        # Handle Roster
        roster_gen = RosterGenerator()
        if os.path.exists(args.roster):
            print(f"Loading roster from {args.roster}...", flush=True)
            roster = roster_gen.load_roster(args.roster)
        else:
            print(f"Generating new roster...", flush=True)
            roster = roster_gen.generate_roster(25)
            roster_gen.save_roster(args.roster)
            print(f"Saved roster to {args.roster}", flush=True)

        # Handle LLM
        llm = None
        if args.gemini:
            if not os.getenv("GEMINI_API_KEY"):
                print("\nGemini API key not found.")
                key = input("Please paste your Gemini API key: ").strip()
                if key:
                    with open(".env", "a" if os.path.exists(".env") else "w") as f:
                        f.write(f"\nGEMINI_API_KEY={key}\n")
                    os.environ["GEMINI_API_KEY"] = key
                    print("API key saved to .env\n")
                else:
                    print(
                        "Error: Gemini API key is required for --gemini mode.",
                        file=sys.stderr,
                    )
                    sys.exit(1)

            print(f"Initializing Gemini LLM with model: {args.model}...", flush=True)
            llm = GeminiGenerator(model_name=args.model)

        # Create a topic-based subfolder for this run's output
        if args.topic:
            # Extract first two words from topic for folder name
            words = args.topic.split()[:2]
            folder_name = "_".join(w.lower() for w in words)
            # Clean up any non-alphanumeric chars
            folder_name = "".join(
                c if c.isalnum() or c == "_" else "_" for c in folder_name
            )
        else:
            # Fallback to timestamp if no topic
            folder_name = datetime.datetime.now().strftime("%Y%m%d_%H%M%S")

        run_output_dir = os.path.join(args.output, folder_name)
        os.makedirs(run_output_dir, exist_ok=True)
        print(f"Output folder: {run_output_dir}", flush=True)

        gen = ThreadGenerator(
            roster=roster,
            llm=llm,
            output_dir=run_output_dir,
            topic=args.topic,
            attachment_percent=args.attachments,
        )

        # Handle backwards compatibility: --steps is deprecated
        target_files = args.files
        if args.steps is not None:
            print(f"Warning: --steps is deprecated, use --files instead", flush=True)
            target_files = args.steps

        print(f"Generating {target_files} inclusive email threads...", flush=True)
        print(f"Attachment rate: {args.attachments}%", flush=True)
        if args.topic:
            print(f"Topic: {args.topic}", flush=True)

        gen.simulate(target_inclusive=target_files)

        print(f"Generated {len(gen.emails)} emails.", flush=True)

        # Find all emails that are parents of other emails (non-inclusive)
        parent_message_ids = set()
        for email_obj in gen.emails:
            if email_obj.parent_id:
                parent_message_ids.add(email_obj.parent_id)

        # Count inclusive emails
        inclusive_emails = [
            e for e in gen.emails if e.message_id not in parent_message_ids
        ]
        print(f"Inclusive (leaf) emails: {len(inclusive_emails)}", flush=True)

        # Sort inclusive emails by thread_id first, then by date
        inclusive_emails.sort(key=lambda e: (e.thread_id, e.date))

        # Save inclusive emails with attachments
        all_attachments = set()
        inclusive_idx = 0
        for email_obj in inclusive_emails:
            inclusive_idx += 1

            # Generate attachment for this inclusive email based on percentage
            if random.random() < args.attachments / 100.0:
                safe_subject = "".join(
                    [c if c.isalnum() else "_" for c in email_obj.subject]
                )[:40]
                doc_types = ["report", "proposal", "notes", "analysis", "summary"]
                doc_type = random.choice(doc_types)
                filepath = gen.file_gen.generate_random_file(
                    safe_subject, doc_type=doc_type, context=email_obj.body[:200]
                )
                filename = os.path.basename(filepath)
                ctype = (
                    "application/pdf"
                    if filename.endswith(".pdf")
                    else "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                )
                email_obj.attachments = [Attachment(filename, filepath, ctype)]

            for att in email_obj.attachments:
                all_attachments.add(att.filepath)

            md_path = save_as_markdown(
                email_obj, gen.file_gen.output_dir, index=inclusive_idx
            )

            print(f"Saved: {md_path}", flush=True)

        # Cleanup original unnumbered attachment files
        for att_path in all_attachments:
            if os.path.exists(att_path):
                try:
                    os.remove(att_path)
                except Exception as e:
                    print(
                        f"Warning: Could not remove original attachment {att_path}: {e}"
                    )

    except Exception as e:
        print(f"\nCRITICAL ERROR: {e}", file=sys.stderr)
        traceback.print_exc(file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
