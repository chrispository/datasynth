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
        import random
        styles = [
            "direct and concise",
            "formal and detailed",
            "casual and friendly",
            "slightly urgent",
            "inquisitive",
            "collaborative",
            "apologetic but firm",
            "enthusiastic"
        ]
        style = random.choice(styles)
        
        prompt = f"""
        Generate a professional business email.
        Sender: {sender['name']} ({sender['title']} in {sender['department']})
        Recipients: {', '.join([r['name'] for r in recipients])}
        Topic: {topic}
        Style/Tone: {style}
        """
        
        if context:
            prompt += f"""
            
            CONTEXT (Previous Email Thread):
            {context}
            
            INSTRUCTIONS:
            1. You are replying to the email above.
            2. Address specific points raised in the context.
            3. Do NOT repeat the full context or history. Write ONLY the new body text of your reply.
            4. Keep the subject consistent with the thread (Re: ...) but if this is a new thread, create a specific, interesting subject line (avoid "General check-in").
            """
        else:
            prompt += f"""
            
            INSTRUCTIONS:
            1. This is the start of a new email thread.
            2. Create a specific, interesting Subject line relevant to the topic (avoid generic titles like "Update" or "Hello").
            3. Write the body of the email initiating the discussion.
            """
        
        prompt += "\n\nPlease provide the email in the following format:\nSubject: [Subject]\n\n[Body]"
        
        content = self.generate_email_content(prompt)
        if content:
            # Basic parsing of Subject and Body
            lines = content.strip().split('\n')
            subject = "No Subject"
            body = content
            
            # Find subject line
            subject_found = False
            for i, line in enumerate(lines):
                if line.lower().startswith("subject:"):
                    subject = line[len("subject:"):].strip()
                    # If it's a reply and the LLM generated a new subject, we might ignore it in the caller, 
                    # but here we just return what we found.
                    # The body is everything after this line
                    body = '\n'.join(lines[i+1:]).strip()
                    subject_found = True
                    break
            
            if not subject_found:
                # If no subject line found, assume all text is body and subject is generic (or handled by caller)
                body = content
                
            return subject, body
        return None, None
