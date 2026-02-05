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
import re
import traceback
import logging
from dotenv import load_dotenv

from models import ThreadGenerator, Attachment, save_as_markdown
from roster import RosterGenerator
from llm import GeminiGenerator, OpenRouterGenerator
from utils import sanitize_filename

DEFAULT_ROSTER_SIZE = 25


def main():
    load_dotenv()

    # Early console-only logging until output dir is known
    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s [%(levelname)s] %(message)s",
        handlers=[logging.StreamHandler(sys.stdout)],
    )

    try:
        logging.info("Starting generator...")
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
            "--gemini", action="store_true", help="Use Gemini LLM (shorthand for --provider gemini)"
        )
        parser.add_argument(
            "--provider", type=str, default=None, choices=["gemini", "openrouter"],
            help="LLM provider to use"
        )
        parser.add_argument(
            "--model", type=str, default="gemini-2.5-flash", help="Model to use"
        )
        parser.add_argument(
            "--reply-pct", type=int, default=80, help="Reply probability (0-100)"
        )
        parser.add_argument(
            "--forward-pct", type=int, default=10, help="Forward probability (0-100)"
        )
        args = parser.parse_args()

        # Resolve provider
        if args.gemini and not args.provider:
            args.provider = "gemini"

        # Handle Roster
        roster_gen = RosterGenerator()
        if os.path.exists(args.roster):
            logging.info(f"Loading roster from {args.roster}...")
            roster = roster_gen.load_roster(args.roster)
        else:
            logging.info(f"Generating new roster...")
            roster = roster_gen.generate_roster(DEFAULT_ROSTER_SIZE)
            roster_gen.save_roster(args.roster)
            logging.info(f"Saved roster to {args.roster}")

        # Handle LLM
        llm = None
        if args.provider == "gemini":
            if not os.getenv("GEMINI_API_KEY"):
                logging.warning("Gemini API key not found.")
                key = input("Please paste your Gemini API key: ").strip()
                if key:
                    with open(".env", "a" if os.path.exists(".env") else "w") as f:
                        f.write(f"\nGEMINI_API_KEY={key}\n")
                    os.environ["GEMINI_API_KEY"] = key
                    logging.info("API key saved to .env")
                else:
                    logging.error("Gemini API key is required.")
                    sys.exit(1)

            logging.info(f"Initializing Gemini LLM with model: {args.model}...")
            llm = GeminiGenerator(model_name=args.model)

        elif args.provider == "openrouter":
            if not os.getenv("OPENROUTER_API_KEY"):
                logging.error("OPENROUTER_API_KEY not found in environment.")
                sys.exit(1)

            logging.info(f"Initializing OpenRouter LLM with model: {args.model}...")
            llm = OpenRouterGenerator(model_name=args.model)

        # Create a topic-based subfolder for this run's output
        if args.topic:
            # Extract first two words from topic for folder name
            words = args.topic.split()[:2]
            folder_name = sanitize_filename("_".join(w.lower() for w in words))
        else:
            # Fallback to timestamp if no topic
            folder_name = datetime.datetime.now().strftime("%Y%m%d_%H%M%S")

        run_output_dir = os.path.join(args.output, folder_name)
        os.makedirs(run_output_dir, exist_ok=True)

        # Clean previous numbered files so re-runs don't interleave
        for old_file in os.listdir(run_output_dir):
            if re.match(r"^\d{4}[a-z]?_", old_file):
                os.remove(os.path.join(run_output_dir, old_file))

        # Add file handler now that output dir is known
        log_file = os.path.join(run_output_dir, "generator.log")
        file_handler = logging.FileHandler(log_file)
        file_handler.setFormatter(logging.Formatter("%(asctime)s [%(levelname)s] %(message)s"))
        logging.getLogger().addHandler(file_handler)
        logging.info(f"Output folder: {run_output_dir}")

        # Build action weights from CLI args
        terminate_pct = max(0, 100 - args.reply_pct - args.forward_pct)
        action_weights = {
            "reply": args.reply_pct / 100.0,
            "forward": args.forward_pct / 100.0,
            "nothing": terminate_pct / 100.0,
        }

        gen = ThreadGenerator(
            roster=roster,
            llm=llm,
            output_dir=run_output_dir,
            topic=args.topic,
            attachment_percent=args.attachments,
            action_weights=action_weights,
        )

        target_files = args.files

        logging.info(f"Generating {target_files} inclusive email threads...")
        logging.info(f"Attachment rate: {args.attachments}%")
        if args.topic:
            logging.info(f"Topic: {args.topic}")

        gen.simulate(target_inclusive=target_files)

        logging.info(f"Generated {len(gen.emails)} emails.")

        # Find all emails that are parents of other emails (non-inclusive)
        parent_message_ids = set()
        for email_obj in gen.emails:
            if email_obj.parent_id:
                parent_message_ids.add(email_obj.parent_id)

        # Count inclusive emails
        inclusive_emails = [
            e for e in gen.emails if e.message_id not in parent_message_ids
        ]
        logging.info(f"Inclusive (leaf) emails: {len(inclusive_emails)}")

        # Sort inclusive emails by thread_id first, then by date
        inclusive_emails.sort(key=lambda e: (e.thread_id, e.date))

        # Save inclusive emails with attachments
        all_attachments = set()
        logging.info(f"Saving {len(inclusive_emails)} inclusive emails...")
        inclusive_idx = 0
        for email_obj in inclusive_emails:
            inclusive_idx += 1
            logging.info(f"[{inclusive_idx}/{len(inclusive_emails)}] Processing email: {email_obj.subject}")

            # Generate attachment for this inclusive email based on percentage
            if random.random() < args.attachments / 100.0:
                doc_types = ["report", "proposal", "notes", "analysis", "summary"]
                doc_type = random.choice(doc_types)
                logging.info(f"  Generating attachment (type: {doc_type})...")
                filepath = gen.file_gen.generate_random_file(
                    doc_type=doc_type, context=email_obj.body[:200]
                )
                logging.info(f"  Attachment generated: {filepath}")
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

            logging.info(f"Saved: {md_path}")

        # Cleanup original unnumbered attachment files
        for att_path in all_attachments:
            if os.path.exists(att_path):
                try:
                    os.remove(att_path)
                except Exception as e:
                    logging.warning(f"Could not remove original attachment {att_path}: {e}")

    except Exception as e:
        logging.error(f"CRITICAL ERROR: {e}")
        logging.error(traceback.format_exc())
        sys.exit(1)


if __name__ == "__main__":
    main()
