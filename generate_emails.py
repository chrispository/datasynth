import uuid
import random
import datetime
import os
import email
from email.message import EmailMessage
from email.utils import formatdate
import mimetypes
from faker import Faker
from fpdf import FPDF
from docx import Document
from roster import RosterGenerator
from llm import GeminiGenerator

fake = Faker()

class FileGenerator:
    def __init__(self, output_dir="output"):
        self.output_dir = output_dir
        if not os.path.exists(output_dir):
            os.makedirs(output_dir)

    def create_pdf(self, filename, content_text):
        pdf = FPDF()
        pdf.add_page()
        pdf.set_font("Arial", size=12)
        pdf.multi_cell(0, 10, txt=content_text)
        
        filepath = os.path.join(self.output_dir, filename)
        pdf.output(filepath)
        return filepath

    def create_docx(self, filename, content_text):
        doc = Document()
        doc.add_paragraph(content_text)
        
        filepath = os.path.join(self.output_dir, filename)
        doc.save(filepath)
        return filepath

    def generate_random_file(self, base_name):
        ext = random.choice(["pdf", "docx"])
        filename = f"{base_name}.{ext}"
        content = fake.text(max_nb_chars=1000)
        
        if ext == "pdf":
            return self.create_pdf(filename, content)
        else:
            return self.create_docx(filename, content)

class Attachment:
    def __init__(self, filename, filepath, content_type):
        self.id = str(uuid.uuid4())
        self.filename = filename
        self.filepath = filepath
        self.content_type = content_type

    def __repr__(self):
        return f"<Attachment {self.filename}>"

class Email:
    def __init__(self, sender, recipients, subject, body, date, message_id=None, parent_id=None, thread_id=None, msg_type="new"):
        self.id = str(uuid.uuid4())
        self.message_id = message_id if message_id else f"<{self.id}@{fake.free_email_domain()}>"
        self.thread_id = thread_id if thread_id else str(uuid.uuid4())
        self.parent_id = parent_id # Points to the Email object or ID this is a reply/forward to
        
        self.sender = sender
        self.recipients = recipients # List of strings
        self.cc = []
        self.bcc = []
        
        self.subject = subject
        self.body = body
        self.date = date # datetime object
        
        self.type = msg_type # new, reply, forward
        
        self.in_reply_to = parent_id if parent_id else None
        self.references = [] # List of message_ids
        
        self.attachments = []

    def add_attachment(self, attachment):
        self.attachments.append(attachment)

    def __repr__(self):
        return f"[{self.date.strftime('%Y-%m-%d %H:%M')}] {self.sender} -> {', '.join(self.recipients)} | {self.subject} ({self.type})"

class ThreadGenerator:
    def __init__(self, roster=None, llm=None, start_date=None, output_dir="output", topic=None):
        self.emails = [] # Flat list of all emails generated
        self.threads = {} # Map thread_id -> list of Email objects
        self.current_date = start_date if start_date else datetime.datetime.now() - datetime.timedelta(days=30)
        self.roster = roster if roster else [{"name": fake.name(), "email": fake.email(), "title": "Employee", "department": "General"} for _ in range(10)]
        self.llm = llm
        self.file_gen = FileGenerator(output_dir)
        self.topic = topic

    def _tick_time(self):
        # Advance time by random minutes/hours
        increment = datetime.timedelta(minutes=random.randint(1, 120))
        self.current_date += increment
        return self.current_date

    def get_person_display(self, person):
        return f"{person['name']} <{person['email']}>"

    def create_root_email(self):
        sender = random.choice(self.roster)
        recipients = random.sample([p for p in self.roster if p != sender], k=random.randint(1, 3))
        
        subject = None
        body = None

        if self.llm:
            subject, body = self.llm.generate_email(sender, recipients, self.topic if self.topic else "General check-in")
        
        if not body:
            if self.topic:
                subject_start = random.choice(["Regarding", "Update on", "Question about", "Notes for", "Discussion:"])
                subject = f"{subject_start} {self.topic}"
                body_start = f"Hi all,\n\nI wanted to discuss {self.topic}."
                body = f"{body_start}\n\n" + fake.paragraph(nb_sentences=5)
            else:
                subject = fake.sentence(nb_words=4).rstrip('.')
                body = fake.paragraph(nb_sentences=5)
        
        email = Email(
            sender=self.get_person_display(sender),
            recipients=[self.get_person_display(r) for r in recipients],
            subject=subject,
            body=body,
            date=self._tick_time(),
            msg_type="new"
        )
        
        # Chance to add attachment
        if random.random() < 0.3:
            safe_subject = "".join([c if c.isalnum() else "_" for c in subject])
            filepath = self.file_gen.generate_random_file(safe_subject)
            filename = os.path.basename(filepath)
            ctype = "application/pdf" if filename.endswith(".pdf") else "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            email.add_attachment(Attachment(filename, filepath, ctype))

        self._store_email(email)
        return email

    def reply_to(self, parent_email):
        # Extract name from display string "Name <email>"
        def parse_display(display):
            if " <" in display:
                name = display.split(" <")[0]
                email = display.split(" <")[1].rstrip(">")
                return {"name": name, "email": email}
            return {"name": display, "email": display}

        parent_recipients = [parse_display(r) for r in parent_email.recipients]
        parent_sender = parse_display(parent_email.sender)

        # Find the person in the roster that matches a recipient
        sender_info = random.choice(parent_recipients)
        # Try to find their full info in roster
        roster_sender = next((p for p in self.roster if p['email'] == sender_info['email']), None)
        if not roster_sender:
            roster_sender = {"name": sender_info['name'], "email": sender_info['email'], "title": "Employee", "department": "General"}

        recipients = [parent_sender]
        roster_recipients = [next((p for p in self.roster if p['email'] == r['email']), r) for r in recipients]

        subject = parent_email.subject
        if not subject.lower().startswith("re:"):
            subject = f"Re: {subject}"
            
        new_body = None
        if self.llm:
            _, new_body = self.llm.generate_email(roster_sender, roster_recipients, self.topic, context=parent_email.body)

        if not new_body:
            new_body = fake.paragraph(nb_sentences=3)
            if self.topic and random.random() < 0.2:
                 new_body += f"\n\nRegarding the {self.topic} aspect, I agree."

        quoted_text = f"\n\nOn {parent_email.date.strftime('%Y-%m-%d %H:%M')}, {parent_email.sender} wrote:\n> " + parent_email.body.replace("\n", "\n> ")
        full_body = new_body + quoted_text
        
        email = Email(
            sender=self.get_person_display(roster_sender),
            recipients=[self.get_person_display(r) if isinstance(r, dict) and 'email' in r else r for r in recipients],
            subject=subject,
            body=full_body,
            date=self._tick_time(),
            message_id=None,
            parent_id=parent_email.message_id,
            thread_id=parent_email.thread_id,
            msg_type="reply"
        )
        
        # Handle Headers
        email.references = parent_email.references + [parent_email.message_id]
        
        self._store_email(email)
        return email

    def forward(self, parent_email):
        # Simpler logic for forward for now, but use roster
        sender = random.choice(self.roster)
        potential_recipients = [p for p in self.roster if self.get_person_display(p) not in parent_email.recipients and self.get_person_display(p) != parent_email.sender]
        if not potential_recipients:
            potential_recipients = [random.choice(self.roster)]
        recipients = [random.choice(potential_recipients)]
        
        subject = parent_email.subject
        if not subject.lower().startswith("fwd:"):
            subject = f"Fwd: {subject}"

        new_body = None
        if self.llm:
             _, new_body = self.llm.generate_email(sender, recipients, f"Forwarding: {self.topic if self.topic else parent_email.subject}", context=parent_email.body)

        if not new_body:
            new_body = f"FYI.\n\n---------- Forwarded message ----------\nFrom: {parent_email.sender}\nDate: {parent_email.date}\nSubject: {parent_email.subject}\nTo: {', '.join(parent_email.recipients)}\n\n" + parent_email.body
            if self.topic and random.random() < 0.3:
                 new_body = f"Thought you should see this regarding {self.topic}.\n\n" + new_body

        email = Email(
            sender=self.get_person_display(sender),
            recipients=[self.get_person_display(r) for r in recipients],
            subject=subject,
            body=new_body,
            date=self._tick_time(),
            message_id=None,
            parent_id=parent_email.message_id,
            thread_id=parent_email.thread_id,
            msg_type="forward"
        )
        
        email.references = parent_email.references + [parent_email.message_id]
        
        # Carry over attachments
        for att in parent_email.attachments:
            email.add_attachment(att)
            
        self._store_email(email)
        return email

    def _store_email(self, email):
        self.emails.append(email)
        if email.thread_id not in self.threads:
            self.threads[email.thread_id] = []
        self.threads[email.thread_id].append(email)

    def simulate(self, num_roots=5, max_steps=20):
        # Create initial roots
        for _ in range(num_roots):
            self.create_root_email()
            
        # Iteratively evolve
        for _ in range(max_steps):
            # Pick a random thread
            tid = random.choice(list(self.threads.keys()))
            thread_msgs = self.threads[tid]
            
            # Pick a random message to act upon (simulating branching)
            # Bias towards recent messages to simulate active conversation, but allow old replies
            parent = thread_msgs[-1] if random.random() > 0.2 else random.choice(thread_msgs)
            
            action = random.choices(["reply", "forward", "nothing"], weights=[0.6, 0.1, 0.3])[0]
            
            if action == "reply":
                self.reply_to(parent)
            elif action == "forward":
                self.forward(parent)
            else:
                # Start a new root thread occasionally
                if random.random() < 0.1:
                    self.create_root_email()

def save_as_eml(email_obj, output_dir="output"):
    msg = EmailMessage()
    msg['Subject'] = email_obj.subject
    msg['From'] = email_obj.sender
    msg['To'] = ", ".join(email_obj.recipients)
    msg['Date'] = formatdate(email_obj.date.timestamp())
    msg['Message-ID'] = email_obj.message_id
    
    if email_obj.in_reply_to:
        msg['In-Reply-To'] = email_obj.in_reply_to
    
    if email_obj.references:
        msg['References'] = " ".join(email_obj.references)
        
    msg.set_content(email_obj.body)
    
    # Add attachments
    for att in email_obj.attachments:
        # Check if file exists (it should)
        if os.path.exists(att.filepath):
            with open(att.filepath, 'rb') as f:
                file_data = f.read()
                maintype, subtype = att.content_type.split('/', 1)
                msg.add_attachment(file_data, maintype=maintype, subtype=subtype, filename=att.filename)
    
    # Save to file
    # Use a safe filename
    safe_subject = "".join([c if c.isalnum() else "_" for c in email_obj.subject])[:30]
    filename = f"{email_obj.date.strftime('%Y%m%d_%H%M')}_{safe_subject}_{email_obj.id[:6]}.eml"
    filepath = os.path.join(output_dir, filename)
    
    with open(filepath, 'wb') as f:
        f.write(msg.as_bytes())
    return filepath

def save_as_printed_pdf(email_obj, output_dir="output"):
    pdf = FPDF()
    pdf.add_page()
    pdf.set_auto_page_break(auto=True, margin=15)
    
    # Use standard font (Helvetica)
    pdf.set_font("helvetica", size=11)
    
    def clean_text(text):
        # FPDF standard fonts only support Latin-1. Replace unsupported chars.
        return text.encode('latin-1', 'replace').decode('latin-1')

    # Headers
    headers = [
        ("From", email_obj.sender),
        ("Sent", email_obj.date.strftime("%A, %B %d, %Y %I:%M %p")),
        ("To", ", ".join(email_obj.recipients)),
        ("Subject", email_obj.subject),
    ]
    
    if email_obj.attachments:
        att_list = ", ".join([att.filename for att in email_obj.attachments])
        headers.append(("Attachments", att_list))
    
    for label, value in headers:
        pdf.set_font("helvetica", 'B', 11)
        pdf.write(6, f"{label}: ")
        
        pdf.set_font("helvetica", '', 11)
        pdf.write(6, clean_text(value) + "\n")
        
    pdf.ln(2)
    # Draw separator line
    pdf.line(10, pdf.get_y(), 200, pdf.get_y())
    pdf.ln(6)
    
    # Body
    pdf.set_font("helvetica", size=11)
    pdf.multi_cell(0, 5, text=clean_text(email_obj.body))
    
    # Save to file
    safe_subject = "".join([c if c.isalnum() else "_" for c in email_obj.subject])[:30]
    filename = f"{email_obj.date.strftime('%Y%m%d_%H%M')}_{safe_subject}_{email_obj.id[:6]}_printed.pdf"
    filepath = os.path.join(output_dir, filename)
    
    pdf.output(filepath)
    return filepath

if __name__ == "__main__":
    import argparse
    import sys
    import traceback
    from dotenv import load_dotenv

    load_dotenv()

    try:
        print("Starting generator...", flush=True)
        parser = argparse.ArgumentParser()
        parser.add_argument("--roots", type=int, default=5, help="Number of root threads")
        parser.add_argument("--steps", type=int, default=20, help="Number of simulation steps")
        parser.add_argument("--output", type=str, default="output", help="Output directory")
        parser.add_argument("--topic", type=str, default=None, help="Topic to focus on")
        parser.add_argument("--pdf", action="store_true", help="Generate printed PDF versions of emails")
        parser.add_argument("--roster", type=str, default="roster.json", help="Path to roster file")
        parser.add_argument("--gemini", action="store_true", help="Use Gemini LLM for email generation")
        parser.add_argument("--model", type=str, default="gemini-1.5-flash", help="Gemini model to use")
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
                    print("Error: Gemini API key is required for --gemini mode.", file=sys.stderr)
                    sys.exit(1)
            
            print(f"Initializing Gemini LLM with model: {args.model}...", flush=True)
            llm = GeminiGenerator(model_name=args.model)

        gen = ThreadGenerator(roster=roster, llm=llm, output_dir=args.output, topic=args.topic)
        print(f"Simulating email traffic with {args.roots} roots and {args.steps} steps...", flush=True)
        if args.topic:
            print(f"Topic: {args.topic}", flush=True)
            
        gen.simulate(num_roots=args.roots, max_steps=args.steps)
        
        print(f"Generated {len(gen.emails)} emails.", flush=True)
        for email_obj in gen.emails:
            eml_path = save_as_eml(email_obj, gen.file_gen.output_dir)
            msg = f"Saved: {eml_path}"
            
            if args.pdf:
                pdf_path = save_as_printed_pdf(email_obj, gen.file_gen.output_dir)
                msg += f" & {os.path.basename(pdf_path)}"
            
            print(msg, flush=True)
            
    except Exception as e:
        print(f"\nCRITICAL ERROR: {e}", file=sys.stderr)
        traceback.print_exc(file=sys.stderr)
        sys.exit(1)