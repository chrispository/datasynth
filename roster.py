import json
import logging
import os
import random
from typing import Optional

from faker_instance import fake


class RosterGenerator:
    def __init__(self, company_name: Optional[str] = None) -> None:
        self.company_name = company_name if company_name else fake.company()
        self.domain = self.company_name.lower().replace(" ", "").replace(",", "") + ".com"
        self.employees = []

    def generate_roster(self, count: int = 20) -> list[dict]:
        self.employees = []
        departments = ["Engineering", "Marketing", "Sales", "Human Resources", "Finance", "Legal", "Product"]
        titles = {
            "Engineering": ["Software Engineer", "Senior Software Engineer", "Engineering Manager", "CTO", "DevOps Engineer"],
            "Marketing": ["Marketing Specialist", "Marketing Manager", "CMO", "Content Creator"],
            "Sales": ["Sales Representative", "Account Manager", "VP of Sales"],
            "Human Resources": ["HR Generalist", "HR Manager", "Recruiter"],
            "Finance": ["Accountant", "Finance Director", "CFO"],
            "Legal": ["General Counsel", "Legal Assistant"],
            "Product": ["Product Manager", "Director of Product", "UX Designer"]
        }

        for _ in range(count):
            first_name = fake.first_name()
            last_name = fake.last_name()
            name = f"{first_name} {last_name}"
            email = f"{first_name.lower()}.{last_name.lower()}@{self.domain}"
            dept = random.choice(departments)
            title = random.choice(titles[dept])
            
            self.employees.append({
                "name": name,
                "email": email,
                "department": dept,
                "title": title
            })
        return self.employees

    def save_roster(self, filepath: str = "roster.json") -> None:
        with open(filepath, "w") as f:
            json.dump({
                "company_name": self.company_name,
                "domain": self.domain,
                "employees": self.employees
            }, f, indent=4)

    def load_roster(self, filepath: str = "roster.json") -> Optional[list[dict]]:
        if os.path.exists(filepath):
            with open(filepath, "r") as f:
                data = json.load(f)
                self.company_name = data["company_name"]
                self.domain = data["domain"]
                self.employees = data["employees"]
            return self.employees
        return None

if __name__ == "__main__":
    gen = RosterGenerator()
    gen.generate_roster(25)
    gen.save_roster()
    logging.basicConfig(level=logging.INFO, format="%(asctime)s [%(levelname)s] %(message)s")
    logging.info(f"Generated roster for {gen.company_name} with {len(gen.employees)} employees.")
