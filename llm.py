import os
import google.generativeai as genai
from dotenv import load_dotenv

load_dotenv()

class GeminiGenerator:
    def __init__(self, model_name='gemini-2.5-flash'):
        api_key = os.getenv("GEMINI_API_KEY")
        if not api_key:
            raise ValueError("GEMINI_API_KEY not found in environment variables or .env file")
        genai.configure(api_key=api_key)
        # Some versions of the library expect 'models/' prefix, some don't.
        # list_models() showed 'models/gemini-2.5-flash'
        if not model_name.startswith('models/'):
            model_name = f"models/{model_name}"
        self.model = genai.GenerativeModel(model_name)

    def generate_email_content(self, prompt):
        try:
            print(f"  [LLM] Requesting content from Gemini ({self.model.model_name})...", end="", flush=True)
            response = self.model.generate_content(prompt)
            if response and response.text:
                print(" Done.", flush=True)
                return response.text
            else:
                print(" Failed (Empty response).", flush=True)
                return None
        except Exception as e:
            print(f" Failed. Error generating content with Gemini: {e}")
            return None

    def generate_email(self, sender, recipients, topic, context=None):
        prompt = f"""
        Generate a professional email.
        Sender: {sender['name']} ({sender['title']} in {sender['department']})
        Recipients: {', '.join([r['name'] for r in recipients])}
        Topic: {topic}
        """
        if context:
            prompt += f"\nContext/Previous Thread:\n{context}"
        
        prompt += "\n\nPlease provide the email in the following format:\nSubject: [Subject]\n\n[Body]"
        
        content = self.generate_email_content(prompt)
        if content:
            # Basic parsing of Subject and Body
            lines = content.strip().split('\n')
            subject = "No Subject"
            body = content
            for i, line in enumerate(lines):
                if line.lower().startswith("subject:"):
                    subject = line[len("subject:"):].strip()
                    body = '\n'.join(lines[i+1:]).strip()
                    break
            return subject, body
        return None, None
