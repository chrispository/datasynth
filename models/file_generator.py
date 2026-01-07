"""
File generator for creating PDF and DOCX attachments.
"""

import os
import re
import random
from faker import Faker
from docx import Document
from pdf_utils import init_pdf

fake = Faker()


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
        """Create a DOCX file with the given content."""
        doc = Document()
        # Add a title at the top of the Word document
        title_text = filename.replace(".docx", "").replace("_", " ")
        # Clean up common prefixes if any
        title_text = re.sub(r"^\d{4}\s+", "", title_text)
        doc.add_heading(title_text, 0)

        # Split by double newlines to create actual paragraphs
        paragraphs = content_text.split("\n\n")
        for p in paragraphs:
            if p.strip():
                doc.add_paragraph(p.strip())

        filepath = os.path.join(self.output_dir, filename)
        doc.save(filepath)
        return filepath

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
Use appropriate headings, bullet points, and sections as needed for a professional business document.
Keep it under 750 words. Write ONLY the document content."""

            content = self.llm.generate_email_content(prompt)
            if content:
                return content

        # Fallback to topic-based template
        if self.topic:
            templates = [
                f"DOCUMENT: {self.topic}\n\n" + fake.paragraph(nb_sentences=8),
                f"REPORT: Analysis of {self.topic}\n\nExecutive Summary:\n"
                + fake.paragraph(nb_sentences=5)
                + "\n\nDetails:\n"
                + fake.paragraph(nb_sentences=8),
                f"NOTES: {self.topic} Discussion\n\n" + fake.paragraph(nb_sentences=10),
                f"PROPOSAL: {self.topic}\n\nBackground:\n"
                + fake.paragraph(nb_sentences=4)
                + "\n\nRecommendations:\n"
                + fake.paragraph(nb_sentences=6),
            ]
            return random.choice(templates)

        return fake.text(max_nb_chars=1000)

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
