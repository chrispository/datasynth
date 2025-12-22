import os
import argparse
import io
import re
from email import policy
from email.parser import BytesParser
from fpdf import FPDF
from pypdf import PdfWriter, PdfReader

try:
    from docx import Document
    HAS_DOCX = True
except ImportError:
    HAS_DOCX = False

def parse_eml(file_path):
    with open(file_path, 'rb') as f:
        msg = BytesParser(policy=policy.default).parse(f)
    
    subject = msg.get('subject', '(No Subject)')
    from_ = msg.get('from', '(Unknown Sender)')
    to = msg.get('to', '(Unknown Recipient)')
    date = msg.get('date', '(Unknown Date)')
    
    body = ""
    if msg.is_multipart():
        for part in msg.walk():
            ctype = part.get_content_type()
            cdispo = str(part.get('Content-Disposition'))
            if ctype == 'text/plain' and 'attachment' not in cdispo:
                body = part.get_content()
                break
    else:
        body = msg.get_content()
        
    return {
        "subject": subject,
        "from": from_,
        "to": to,
        "sent": date,
        "body": body
    }

def sanitize_text(text, collapse_whitespace=True):
    if not text:
        return ""
    if not isinstance(text, str):
        text = str(text)
    
    if collapse_whitespace:
        # Normalize whitespace (handles folded RFC 5322 headers)
        import re
        text = re.sub(r'\s+', ' ', text).strip()
    
    # Mapping for common non-latin-1 characters
    mapping = {
        0x2013: "-",    # En dash
        0x2014: "--",   # Em dash
        0x2018: "'",    # Left single quote
        0x2019: "'",    # Right single quote
        0x201c: '"',    # Left double quote
        0x201d: '"',    # Right double quote
        0x2022: "*",    # Bullet
        0x2026: "...",  # Ellipsis
        0x00a0: " ",    # Non-breaking space
        0x2122: "(TM)", # Trademark
        0x00ae: "(R)",  # Registered
        0x00a9: "(C)",  # Copyright
        0x2022: "·",    # Middle dot/bullet
    }
    
    chars = []
    for char in text:
        cp = ord(char)
        if cp in mapping:
            chars.append(mapping[cp])
        elif cp < 256:
            # latin-1 compatible
            chars.append(char)
        else:
            # Fallback to ?
            chars.append("?")
            
    # Final pass to ensure no characters outside the range that Helvetica likes
    result = "".join(chars)
    # Most PDF viewers/generators with core fonts only like characters in WinAnsiEncoding or Latin-1
    # We'll just be safe and encode/decode to latin-1
    return result.encode('latin-1', 'replace').decode('latin-1')

class PDFConverter:
    def __init__(self, output_path):
        from fpdf.enums import XPos, YPos
        self.XPos = XPos
        self.YPos = YPos
        
        self.output_path = output_path
        
        # Simple FPDF without custom footer
        self.pdf = FPDF()
        self.pdf.set_margins(10, 10, 10)  # left, top, right margins
        self.pdf.set_auto_page_break(auto=True, margin=15)
        # Page will be added on first write

    def ensure_page(self):
        if self.pdf.page_no() == 0:
            self.pdf.add_page()
            self.pdf.set_font("helvetica", size=10)

    def add_eml(self, file_path):
        data = parse_eml(file_path)
        
        # Start each email on a new page
        self.pdf.add_page()
        self.pdf.set_font("helvetica", size=10)
        
        # Headers - label bold, value regular (like real email clients)
        headers = [
            ("From:", data['from']),
            ("Sent:", data['sent']),
            ("To:", data['to']),
            ("Subject:", data['subject']),
        ]
        
        for label, value in headers:
            # Bold label
            self.pdf.set_font("helvetica", "B", 10)
            label_width = self.pdf.get_string_width(label) + 2
            self.pdf.cell(label_width, 6, text=label, new_x=self.XPos.RIGHT, new_y=self.YPos.TOP)
            
            # Regular value
            self.pdf.set_font("helvetica", "", 10)
            self.pdf.multi_cell(0, 6, text=sanitize_text(value), new_x=self.XPos.LMARGIN, new_y=self.YPos.NEXT)
        
        self.pdf.ln(3)
        
        # Separator line (like Outlook printed emails)
        self.pdf.set_draw_color(180, 180, 180) # Light gray
        y = self.pdf.get_y()
        self.pdf.line(10, y, 200, y)
        self.pdf.ln(5)
        self.pdf.set_draw_color(0, 0, 0) # Back to black
        
        # Body
        self.pdf.set_font("helvetica", "", 10)
        body = data['body'].strip() if data['body'] else ""
        # Basic markdown-to-print cleanup
        body = body.replace("**", "")
        body = re.sub(r"^\* ", "• ", body, flags=re.MULTILINE)
        
        self.pdf.multi_cell(0, 5, text=sanitize_text(body, collapse_whitespace=False), new_x=self.XPos.LMARGIN, new_y=self.YPos.NEXT)

    def add_md(self, file_path):
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # Parse headers
        headers = {}
        header_patterns = {
            'From:': r'^\*\*From:\*\* (.*)$',
            'Date:': r'^\*\*Date:\*\* (.*)$',
            'To:': r'^\*\*To:\*\* (.*)$',
            'Subject:': r'^\*\*Subject:\*\* (.*)$',
            'Cc:': r'^\*\*Cc:\*\* (.*)$',
            'Attachments:': r'^\*\*Attachments:\*\* (.*)$',
        }
        
        lines = content.split('\n')
        body_start_idx = 0
        for i, line in enumerate(lines):
            if line.strip() == '---':
                body_start_idx = i + 1
                break
            for label, pattern in header_patterns.items():
                match = re.match(pattern, line)
                if match:
                    headers[label] = match.group(1)
        
        body = '\n'.join(lines[body_start_idx:]).strip()

        # Start each email on a new page
        self.pdf.add_page()
        self.pdf.set_font("helvetica", size=10)
        
        # Headers - label bold, value regular
        display_headers = [
            ("From:", headers.get('From:', '(Unknown Sender)')),
            ("Sent:", headers.get('Date:', '(Unknown Date)')), # Use 'Sent:' for consistency
            ("To:", headers.get('To:', '(Unknown Recipient)')),
        ]
        if 'Cc:' in headers:
            display_headers.append(("Cc:", headers['Cc:']))
        display_headers.append(("Subject:", headers.get('Subject:', '(No Subject)')))
        
        for label, value in display_headers:
            self.pdf.set_font("helvetica", "B", 10)
            label_width = self.pdf.get_string_width(label) + 2
            self.pdf.cell(label_width, 6, text=label, new_x=self.XPos.RIGHT, new_y=self.YPos.TOP)
            
            self.pdf.set_font("helvetica", "", 10)
            self.pdf.multi_cell(0, 6, text=sanitize_text(value), new_x=self.XPos.LMARGIN, new_y=self.YPos.NEXT)
        
        self.pdf.ln(3)
        
        # Separator line
        self.pdf.set_draw_color(180, 180, 180) # Light gray
        y = self.pdf.get_y()
        self.pdf.line(10, y, 200, y)
        self.pdf.ln(5)
        self.pdf.set_draw_color(0, 0, 0) # Back to black
        
        self.pdf.set_font("helvetica", "", 10)
        # Basic markdown-to-print cleanup
        body = body.replace("**", "")
        body = re.sub(r"^\* ", "• ", body, flags=re.MULTILINE)
        
        self.pdf.multi_cell(0, 5, text=sanitize_text(body, collapse_whitespace=False), new_x=self.XPos.LMARGIN, new_y=self.YPos.NEXT)

    def add_docx(self, file_path):
        # Always start on a new page for "imaged out natively" effect
        self.pdf.add_page()
        
        if not HAS_DOCX:
            self.pdf.set_font("helvetica", "I", 10)
            self.pdf.multi_cell(0, 6, text=sanitize_text(f"[Attachment: {os.path.basename(file_path)} - python-docx not installed]"), new_x=self.XPos.LMARGIN, new_y=self.YPos.NEXT)
            return

        try:
            doc = Document(file_path)
            
            # Distinctive header: Filename centered
            raw_title = os.path.basename(file_path)
            # Remove prefixes like 0001_Subject_attachment_
            title = re.sub(r'^\d{4}_.*_attachment_', '', raw_title)
            title = title.replace('.docx', '').replace('_', ' ')
            
            # Use Times for a more "document" feel (serif vs sans-serif for emails)
            self.pdf.set_font("times", "B", 14)
            self.pdf.cell(0, 10, text=sanitize_text(title), align="C", new_x=self.XPos.LMARGIN, new_y=self.YPos.NEXT)
            self.pdf.ln(5)
            
            self.pdf.set_font("times", "", 11)
            for para in doc.paragraphs:
                text = para.text.strip()
                if text:
                    # Basic cleanup
                    clean_text = sanitize_text(text, collapse_whitespace=False)
                    clean_text = clean_text.replace("**", "")
                    self.pdf.multi_cell(0, 5, text=clean_text, new_x=self.XPos.LMARGIN, new_y=self.YPos.NEXT)
            self.pdf.ln(10)
        except Exception as e:
            self.pdf.set_font("helvetica", "I", 10)
            self.pdf.multi_cell(0, 6, text=sanitize_text(f"[Error reading docx {os.path.basename(file_path)}: {e}]"), new_x=self.XPos.LMARGIN, new_y=self.YPos.NEXT)





    def save_temp_pdf(self):
        temp_path = "temp_generated.pdf"
        self.pdf.output(temp_path)
        return temp_path

def combine_files(folder_path, output_file):
    # Find all numbered files
    files = []
    for f in os.listdir(folder_path):
        match = re.match(r'^(\d{4})_', f)
        if match:
            files.append((match.group(1), f))
    
    files.sort() # Sort by prefix (0001, 0002...)
    
    writer = PdfWriter()
    converter = PDFConverter(output_file)
    
    for _, filename in files:
        file_path = os.path.join(folder_path, filename)
        ext = filename.lower().split('.')[-1]
        
        if ext == 'eml':
            converter.add_eml(file_path)
        elif ext == 'md':
            converter.add_md(file_path)
        elif ext == 'docx':
            converter.add_docx(file_path)
        elif ext == 'pdf':
            # Handle PDF merging
            # First, flush current converter to a temp PDF and add it to writer
            if len(converter.pdf.pages) > 0:
                temp_pdf = converter.save_temp_pdf()
                with open(temp_pdf, "rb") as f:
                    writer.append(f)
                os.remove(temp_pdf)
                # Reset converter for fresh start after this PDF
                converter = PDFConverter(output_file)
            
            # Now add the attachment PDF
            try:
                with open(file_path, "rb") as f:
                    writer.append(f)
            except Exception as e:
                print(f"Error merging PDF {filename}: {e}")

    # Final flush
    if len(converter.pdf.pages) > 0:
        temp_pdf = converter.save_temp_pdf()
        with open(temp_pdf, "rb") as f:
            writer.append(f)
        os.remove(temp_pdf)

    with open(output_file, "wb") as f:
        writer.write(f)
    print(f"Created combined PDF: {output_file}")

def convert_individual(folder_path):
    # This logic still exists if user doesn't want combine, but usually they do
    files = [f for f in os.listdir(folder_path) if f.lower().endswith(('.eml', '.md'))]
    files.sort()
    for filename in files:
        file_path = os.path.join(folder_path, filename)
        output_name = os.path.splitext(filename)[0] + ".pdf"
        output_path = os.path.join(folder_path, output_name)
        
        converter = PDFConverter(output_path)
        ext = filename.lower().split('.')[-1]
        if ext == 'eml':
            converter.add_eml(file_path)
        elif ext == 'md':
            converter.add_md(file_path)
        converter.pdf.output(output_path)
        print(f"Converted: {output_name}")

def main():
    parser = argparse.ArgumentParser(description="Convert EML and attachments to PDF")
    parser.add_argument("--folder", required=True, help="Path to folder containing files")
    parser.add_argument("--combine", action="store_true", help="Combine all into one PDF")
    
    args = parser.parse_args()
    
    if not os.path.exists(args.folder):
        print(f"Error: Folder does not exist: {args.folder}")
        return

    if args.combine:
        folder_name = os.path.basename(os.path.normpath(args.folder))
        output_path = os.path.join(args.folder, f"{folder_name}_combined.pdf")
        combine_files(args.folder, output_path)
    else:
        convert_individual(args.folder)

if __name__ == "__main__":
    main()

