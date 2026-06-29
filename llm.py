import logging
import os
import random
from typing import Optional

import google.generativeai as genai
from openai import OpenAI
from dotenv import load_dotenv

load_dotenv()

class GeminiGenerator:
    def __init__(self, model_name: str = 'gemini-2.5-flash-lite') -> None:
        api_key = os.getenv("GEMINI_API_KEY")
        if not api_key:
            raise ValueError("GEMINI_API_KEY not found in environment variables or .env file")
        genai.configure(api_key=api_key)
        # Some versions of the library expect 'models/' prefix, some don't.
        # list_models() showed 'models/gemini-2.5-flash'
        if not model_name.startswith('models/'):
            model_name = f"models/{model_name}"
        self.model = genai.GenerativeModel(model_name)

    def generate_email_content(self, prompt: str) -> Optional[str]:
        try:
            logging.info(f"  [LLM] Requesting content from Gemini ({self.model.model_name})...")
            response = self.model.generate_content(prompt)
            if response and response.text:
                logging.info("  [LLM] Done.")
                return response.text
            else:
                logging.warning("  [LLM] Failed (Empty response).")
                return None
        except Exception as e:
            logging.warning(f"  [LLM] Failed. Error generating content with Gemini: {e}")
            return None

    def generate_email(
        self,
        sender: dict,
        recipients: list[dict],
        topic: Optional[str],
        context: Optional[str] = None,
        used_subjects: Optional[list[str]] = None,
        is_forward: bool = False,
    ) -> tuple[Optional[str], Optional[str]]:
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

        topic_line = f"Topic: {topic}\n" if topic else ""
        prompt = f"""
        Generate a professional business email.
        Sender: {sender['name']} ({sender['title']} in {sender['department']})
        Recipients: {', '.join([r['name'] for r in recipients])}
        {topic_line}Style/Tone: {style}
        """

        if context and is_forward:
            prompt += f"""

            CONTEXT (Email being forwarded):
            {context}

            INSTRUCTIONS:
            1. You are forwarding the email above to a new recipient who was NOT part of the original thread.
            2. Write 1-3 short sentences of forwarder commentary (e.g., "FYI", "thought you should see this", "can you weigh in?"). It should sound like a brief intro, not a reply.
            3. Do NOT restate, summarize, or rewrite the forwarded email's content.
            4. Do NOT produce a Subject line; one will be set by the caller.
            """
        elif context:
            prompt += f"""

            CONTEXT (Previous Email Thread):
            {context}

            INSTRUCTIONS:
            1. You are replying to the email above.
            2. Address specific points raised in the context.
            3. Do NOT repeat the full context or history. Write ONLY the new body text of your reply.
            4. Do NOT produce a Subject line; the thread's Re: subject will be set by the caller.
            """
        else:
            prompt += f"""

            INSTRUCTIONS:
            1. This is the start of a new email thread.
            2. Create a specific, interesting Subject line relevant to the topic (avoid generic titles like "Update" or "Hello").
            3. Write the body of the email initiating the discussion.
            """
            if used_subjects:
                prompt += f"""
            4. IMPORTANT: Do NOT reuse or closely resemble any of these previously used subjects: {used_subjects}
               Each new thread MUST have a distinctly different subject line.
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


class OpenRouterGenerator:
    def __init__(self, model_name: str = 'anthropic/claude-sonnet-4') -> None:
        api_key = os.getenv("OPENROUTER_API_KEY")
        if not api_key:
            raise ValueError("OPENROUTER_API_KEY not found in environment variables or .env file")
        self.client = OpenAI(
            base_url="https://openrouter.ai/api/v1",
            api_key=api_key,
        )
        self.model_name = model_name

    def generate_email_content(self, prompt: str) -> Optional[str]:
        try:
            logging.info(f"  [LLM] Requesting content from OpenRouter ({self.model_name})...")
            response = self.client.chat.completions.create(
                model=self.model_name,
                messages=[{"role": "user", "content": prompt}],
            )
            if response and response.choices and response.choices[0].message.content:
                logging.info("  [LLM] Done.")
                return response.choices[0].message.content
            else:
                logging.warning("  [LLM] Failed (Empty response).")
                return None
        except Exception as e:
            logging.warning(f"  [LLM] Failed. Error generating content with OpenRouter: {e}")
            return None

    def generate_email(
        self,
        sender: dict,
        recipients: list[dict],
        topic: Optional[str],
        context: Optional[str] = None,
        used_subjects: Optional[list[str]] = None,
        is_forward: bool = False,
    ) -> tuple[Optional[str], Optional[str]]:
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

        topic_line = f"Topic: {topic}\n" if topic else ""
        prompt = f"""
        Generate a professional business email.
        Sender: {sender['name']} ({sender['title']} in {sender['department']})
        Recipients: {', '.join([r['name'] for r in recipients])}
        {topic_line}Style/Tone: {style}
        """

        if context and is_forward:
            prompt += f"""

            CONTEXT (Email being forwarded):
            {context}

            INSTRUCTIONS:
            1. You are forwarding the email above to a new recipient who was NOT part of the original thread.
            2. Write 1-3 short sentences of forwarder commentary (e.g., "FYI", "thought you should see this", "can you weigh in?"). It should sound like a brief intro, not a reply.
            3. Do NOT restate, summarize, or rewrite the forwarded email's content.
            4. Do NOT produce a Subject line; one will be set by the caller.
            """
        elif context:
            prompt += f"""

            CONTEXT (Previous Email Thread):
            {context}

            INSTRUCTIONS:
            1. You are replying to the email above.
            2. Address specific points raised in the context.
            3. Do NOT repeat the full context or history. Write ONLY the new body text of your reply.
            4. Do NOT produce a Subject line; the thread's Re: subject will be set by the caller.
            """
        else:
            prompt += f"""

            INSTRUCTIONS:
            1. This is the start of a new email thread.
            2. Create a specific, interesting Subject line relevant to the topic (avoid generic titles like "Update" or "Hello").
            3. Write the body of the email initiating the discussion.
            """
            if used_subjects:
                prompt += f"""
            4. IMPORTANT: Do NOT reuse or closely resemble any of these previously used subjects: {used_subjects}
               Each new thread MUST have a distinctly different subject line.
            """

        prompt += "\n\nPlease provide the email in the following format:\nSubject: [Subject]\n\n[Body]"

        content = self.generate_email_content(prompt)
        if content:
            lines = content.strip().split('\n')
            subject = "No Subject"
            body = content

            subject_found = False
            for i, line in enumerate(lines):
                if line.lower().startswith("subject:"):
                    subject = line[len("subject:"):].strip()
                    body = '\n'.join(lines[i+1:]).strip()
                    subject_found = True
                    break

            if not subject_found:
                body = content

            return subject, body
        return None, None
