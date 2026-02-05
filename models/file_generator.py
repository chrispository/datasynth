"""
File generator for creating PDF and DOCX attachments.
"""

import os
import re
import random
from docx import Document
from docx.enum.section import WD_ORIENT
from docx.shared import Inches
from pdf_utils import init_pdf

from faker_instance import fake


class FileGenerator:
    """
    Generates PDF and DOCX file attachments.
    
    Uses LLM for content generation when available, with faker-based
    fallbacks for offline operation.
    """
    
    def __init__(self, output_dir="output", llm=None, topic=None):
        self.output_dir = output_dir
        self.llm = llm
        self.topic = topic
        if not os.path.exists(output_dir):
            os.makedirs(output_dir)

    def create_pdf(self, filename, content_text):
        """Create a PDF file with the given content."""
        pdf = init_pdf()
        pdf.add_page()
        pdf.set_font("DejaVu", size=12)
        pdf.multi_cell(0, 10, txt=content_text)

        filepath = os.path.join(self.output_dir, filename)
        pdf.output(filepath)
        return filepath

    def create_docx(self, filename, content_text):
        """Create a DOCX file with rich formatting parsed from markdown."""
        doc = Document()

        # 25% chance of landscape orientation
        if random.random() < 0.25:
            section = doc.sections[0]
            section.orientation = WD_ORIENT.LANDSCAPE
            # Swap width and height for landscape
            new_width, new_height = section.page_height, section.page_width
            section.page_width = new_width
            section.page_height = new_height

        # Add a title at the top of the Word document
        title_text = filename.replace(".docx", "").replace("_", " ")
        title_text = re.sub(r"^\d{4}\s+", "", title_text)
        doc.add_heading(title_text, 0)

        lines = content_text.split("\n")
        for line in lines:
            stripped = line.strip()
            if not stripped:
                continue

            # Headings: ## or ###
            if stripped.startswith("### "):
                doc.add_heading(stripped[4:].strip("# "), level=2)
            elif stripped.startswith("## "):
                doc.add_heading(stripped[3:].strip("# "), level=1)
            elif stripped.startswith("# "):
                doc.add_heading(stripped[2:].strip("# "), level=1)
            # Bullet list: - item or * item
            elif re.match(r"^[-*]\s+", stripped):
                text = re.sub(r"^[-*]\s+", "", stripped)
                para = doc.add_paragraph(style="List Bullet")
                self._add_runs_with_bold(para, text)
            # Numbered list: 1. item
            elif re.match(r"^\d+\.\s+", stripped):
                text = re.sub(r"^\d+\.\s+", "", stripped)
                para = doc.add_paragraph(style="List Number")
                self._add_runs_with_bold(para, text)
            else:
                para = doc.add_paragraph()
                self._add_runs_with_bold(para, stripped)

        filepath = os.path.join(self.output_dir, filename)
        doc.save(filepath)
        return filepath

    def _add_runs_with_bold(self, paragraph, text):
        """Parse **bold** markers in text and add runs to paragraph."""
        parts = re.split(r"(\*\*[^*]+\*\*)", text)
        for part in parts:
            if part.startswith("**") and part.endswith("**"):
                run = paragraph.add_run(part[2:-2])
                run.bold = True
            else:
                paragraph.add_run(part)

    def _generate_content(self, doc_type, context=None):
        """Generate document content using LLM or fallback templates."""
        if self.llm:
            prompt = f"Generate a realistic {doc_type} document"
            if self.topic:
                prompt += f" related to {self.topic}"
            if context:
                prompt += f". Context from related email thread: {context}"
            prompt += """. 

CRITICAL RULES - This is a standalone business document (Word/PDF attachment), NOT AN EMAIL:
- NO email headers or metadata (no "Date:", "From:", "To:", "Subject:", "Re:", etc.)
- NO email greetings or signatures ("Dear", "Hi", "Best regards", "Sincerely", etc.)
- NO date headers at the top of the document
- NO "Prepared by" or "Prepared for" lines
- NO document metadata blocks

FORMAT: Start directly with the document title/heading, then the content.
Use markdown formatting for structure:
- Use ## for section headings and ### for sub-headings
- Use - or * for bullet lists
- Use 1. 2. 3. for numbered lists
- Use **bold** for key terms and emphasis
Mix these elements for a rich, professional layout.
Keep it under 750 words. Write ONLY the document content."""

            content = self.llm.generate_email_content(prompt)
            if content:
                return content

        # Fallback to topic-based template with markdown structure
        if self.topic:
            templates = [
                f"## Overview\n\n{fake.paragraph(nb_sentences=4)}\n\n"
                f"## Key Points\n\n"
                f"- {fake.sentence()}\n- {fake.sentence()}\n- {fake.sentence()}\n\n"
                f"## Details\n\n{fake.paragraph(nb_sentences=6)}",

                f"## Executive Summary\n\n{fake.paragraph(nb_sentences=5)}\n\n"
                f"## Analysis of {self.topic}\n\n{fake.paragraph(nb_sentences=4)}\n\n"
                f"### Findings\n\n"
                f"1. {fake.sentence()}\n2. {fake.sentence()}\n3. {fake.sentence()}\n\n"
                f"## Conclusion\n\n{fake.paragraph(nb_sentences=3)}",

                f"## {self.topic} Discussion Notes\n\n"
                f"**Attendees:** {fake.name()}, {fake.name()}, {fake.name()}\n\n"
                f"### Agenda Items\n\n"
                f"- {fake.sentence()}\n- {fake.sentence()}\n\n"
                f"### Action Items\n\n"
                f"1. {fake.sentence()}\n2. {fake.sentence()}\n\n"
                f"## Summary\n\n{fake.paragraph(nb_sentences=4)}",

                f"## Proposal: {self.topic}\n\n"
                f"### Background\n\n{fake.paragraph(nb_sentences=4)}\n\n"
                f"### Recommendations\n\n"
                f"- **Option A:** {fake.sentence()}\n"
                f"- **Option B:** {fake.sentence()}\n\n"
                f"### Next Steps\n\n{fake.paragraph(nb_sentences=3)}",
            ]
            return random.choice(templates)

        # Structured fallback instead of random faker text
        return (
            f"## Business Document\n\n"
            f"{fake.paragraph(nb_sentences=4)}\n\n"
            f"## Key Points\n\n"
            f"- {fake.sentence()}\n"
            f"- {fake.sentence()}\n"
            f"- {fake.sentence()}\n\n"
            f"## Summary\n\n"
            f"{fake.paragraph(nb_sentences=3)}"
        )

    def _generate_document_title(self, doc_type, context=None):
        """Generate a professional document title using LLM or fallback."""
        if self.llm:
            prompt = f"""Generate a short, professional document filename (no extension) for a {doc_type}.
Context: {self.topic if self.topic else 'general business'}
Rules:
- Use 2-5 words maximum
- Use Title_Case_With_Underscores
- No dates, no special characters, no spaces
- Examples: Quarterly_Budget_Analysis, Project_Proposal, Meeting_Notes, Vendor_Agreement
Return ONLY the filename, nothing else."""
            title = self.llm.generate_email_content(prompt)
            if title:
                # Clean up any extra whitespace or quotes
                title = title.strip().strip('"').strip("'").strip()
                # Ensure proper formatting
                title = "_".join(word.capitalize() for word in title.replace("_", " ").split())
                # Remove any remaining non-alphanumeric chars except underscores
                title = "".join(c if c.isalnum() or c == "_" else "" for c in title)
                if title:
                    return title
        
        # Fallback: combine topic and doc_type
        if self.topic:
            # Take first 2-3 words of topic
            words = self.topic.split()[:3]
            topic_part = "_".join(w.capitalize() for w in words)
            topic_part = "".join(c if c.isalnum() or c == "_" else "" for c in topic_part)
            doc_type_cap = doc_type.capitalize()
            return f"{topic_part}_{doc_type_cap}"
        
        # Ultimate fallback
        return f"Business_{doc_type.capitalize()}"

    def generate_random_file(self, base_name, doc_type="document", context=None):
        """Generate a random PDF or DOCX file with LLM or fallback content."""
        ext = random.choice(["pdf", "docx"])
        # Generate a clean, professional document title
        doc_title = self._generate_document_title(doc_type, context)
        filename = f"{doc_title}.{ext}"
        content = self._generate_content(doc_type, context)

        if ext == "pdf":
            return self.create_pdf(filename, content)
        else:
            return self.create_docx(filename, content)
