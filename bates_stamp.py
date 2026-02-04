#!/usr/bin/env python3
"""Standalone Bates numbering script for stamping combined PDFs."""

import argparse
import io
import sys
from pathlib import Path

from PyPDF2 import PdfReader, PdfWriter
from reportlab.pdfgen import canvas
from reportlab.lib.pagesizes import letter


def stamp_bates(file_path: str, prefix: str, separator: str, start: int, padding: int) -> None:
    pdf_path = Path(file_path)
    if not pdf_path.exists():
        print(f"ERROR: File not found: {pdf_path}")
        sys.exit(1)

    print(f"Reading {pdf_path.name}...")
    reader = PdfReader(str(pdf_path))
    writer = PdfWriter()
    num_pages = len(reader.pages)
    print(f"Found {num_pages} pages")

    for page_num in range(num_pages):
        page = reader.pages[page_num]

        # Create overlay with Bates number
        packet = io.BytesIO()
        can = canvas.Canvas(packet, pagesize=letter)
        bates_number = f"{prefix}{separator}{str(start + page_num).zfill(padding)}"
        can.setFont("Helvetica", 10)
        can.drawString(450, 30, bates_number)
        can.save()

        packet.seek(0)
        overlay = PdfReader(packet)
        page.merge_page(overlay.pages[0])
        writer.add_page(page)

        if (page_num + 1) % 10 == 0 or page_num == num_pages - 1:
            print(f"Stamped page {page_num + 1}/{num_pages}: {bates_number}")

    # Write to temp then replace
    temp_path = pdf_path.parent / f"bates_temp_{pdf_path.name}"
    with open(temp_path, "wb") as f:
        writer.write(f)
    temp_path.replace(pdf_path)

    last_number = f"{prefix}{separator}{str(start + num_pages - 1).zfill(padding)}"
    print(f"Bates stamping complete: {num_pages} pages ({prefix}{separator}{str(start).zfill(padding)} to {last_number})")


def main():
    parser = argparse.ArgumentParser(description="Add Bates numbers to a PDF")
    parser.add_argument("--file", required=True, help="Path to the PDF file")
    parser.add_argument("--prefix", default="BATES", help="Bates number prefix")
    parser.add_argument("--separator", default="-", help="Separator between prefix and number")
    parser.add_argument("--start", type=int, default=1, help="Starting number")
    parser.add_argument("--padding", type=int, default=7, help="Zero-padding digits")
    args = parser.parse_args()

    stamp_bates(args.file, args.prefix, args.separator, args.start, args.padding)


if __name__ == "__main__":
    main()
