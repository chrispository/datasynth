use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Model,
    Quantity,
    Topics,
    Companies,
    Tone,
    Prompts,
    Run,
}

impl Section {
    pub fn as_str(&self) -> &str {
        match self {
            Section::Model => "Model",
            Section::Quantity => "Quantity",
            Section::Topics => "Topics",
            Section::Companies => "Companies",
            Section::Tone => "Tone",
            Section::Prompts => "Prompts",
            Section::Run => "Run",
        }
    }

    pub fn all() -> [Section; 7] {
        [
            Section::Model,
            Section::Quantity,
            Section::Topics,
            Section::Companies,
            Section::Tone,
            Section::Prompts,
            Section::Run,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Sidebar,
    Main,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Employee {
    pub name: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Company {
    pub name: String,
    pub employees: Vec<Employee>,
}

pub struct App {
    // Navigation
    pub current_section: Section,
    pub sidebar_index: usize,
    pub focus: Focus,
    
    // Model
    pub api_key: String,
    pub available_models: Vec<String>,
    pub selected_model_index: usize,
    
    // Quantity
    pub total_files: u32,
    pub chains: u32,
    pub percent_attachments: u32,
    
    // Topics
    pub generated_topics: Vec<String>,
    pub selected_topics: Vec<String>,
    pub topic_cursor: usize,
    
    // Companies
    pub company_size: u32,
    pub companies: Vec<Company>,
    
    // Logs
    pub logs: Vec<String>,
    
    // Control
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        // Load API key from .env
        let api_key = dotenv::var("GEMINI_API_KEY").unwrap_or_default();
        
        let available_models = vec![
            "gemini-3-pro-preview".to_string(),
            "gemini-3-flash-preview".to_string(),
            "gemini-2.5-flash".to_string(),
        ];
        
        let mut app = Self {
            current_section: Section::Model,
            sidebar_index: 0,
            focus: Focus::Sidebar,
            api_key,
            available_models,
            selected_model_index: 0,
            total_files: 25,
            chains: 5,
            percent_attachments: 30,
            generated_topics: Vec::new(),
            selected_topics: Vec::new(),
            topic_cursor: 0,
            company_size: 10,
            companies: Vec::new(),
            logs: vec!["Application initialized".to_string()],
            should_quit: false,
        };
        
        app.log("Loaded environment");
        app
    }
    
    pub fn log(&mut self, msg: impl Into<String>) {
        self.logs.push(msg.into());
        if self.logs.len() > 100 {
            self.logs.remove(0);
        }
    }
    
    pub fn navigate_up(&mut self) {
        if self.focus == Focus::Sidebar && self.sidebar_index > 0 {
            self.sidebar_index -= 1;
            self.current_section = Section::all()[self.sidebar_index];
        } else if self.focus == Focus::Main && self.current_section == Section::Model {
            // Cycle through models
            if self.selected_model_index > 0 {
                self.selected_model_index -= 1;
            } else {
                self.selected_model_index = self.available_models.len() - 1;
            }
        } else if self.focus == Focus::Main && self.current_section == Section::Topics {
            // Navigate within topics list
            if self.topic_cursor > 0 {
                self.topic_cursor -= 1;
            }
        }
    }
    
    pub fn navigate_down(&mut self) {
        if self.focus == Focus::Sidebar && self.sidebar_index < 6 {
            self.sidebar_index += 1;
            self.current_section = Section::all()[self.sidebar_index];
        } else if self.focus == Focus::Main && self.current_section == Section::Model {
            // Cycle through models
            self.selected_model_index = (self.selected_model_index + 1) % self.available_models.len();
        } else if self.focus == Focus::Main && self.current_section == Section::Topics {
            // Navigate within topics list
            let max = self.generated_topics.len().saturating_sub(1);
            if self.topic_cursor < max {
                self.topic_cursor += 1;
            }
        }
    }
    
    pub fn navigate_right(&mut self) {
        self.focus = Focus::Main;
    }
    
    pub fn navigate_left(&mut self) {
        self.focus = Focus::Sidebar;
    }
    
    pub fn load_topics_from_file(&mut self) {
        match std::fs::read_to_string("topics.txt") {
            Ok(content) => {
                self.generated_topics = content
                    .lines()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                self.log(format!("Loaded {} topics from file", self.generated_topics.len()));
            }
            Err(e) => {
                self.log(format!("Error loading topics.txt: {}", e));
            }
        }
    }
    
    pub fn select_topic(&mut self) {
        if !self.generated_topics.is_empty() && self.topic_cursor < self.generated_topics.len() {
            let topic = self.generated_topics.remove(self.topic_cursor);
            self.selected_topics.push(topic);
            // Adjust cursor if needed
            if self.topic_cursor >= self.generated_topics.len() && self.topic_cursor > 0 {
                self.topic_cursor -= 1;
            }
        }
    }
    
    pub fn generate_companies(&mut self) {
        if self.selected_topics.is_empty() {
            self.log("No topics selected!");
            return;
        }
        
        let roles = ["CEO", "CFO", "CTO", "VP Sales", "VP Engineering", "Product Manager"];
        let names = ["Alice", "Bob", "Charlie", "David", "Eve", "Frank", "Grace", "Heidi"];
        
        self.companies = self.selected_topics
            .iter()
            .enumerate()
            .map(|(idx, topic)| {
                let company_name = format!("{}Corp", topic.replace(" ", ""));
                let employees = (0..self.company_size.min(6))
                    .map(|i| Employee {
                        name: names[(idx + i as usize) % names.len()].to_string(),
                        role: roles[i as usize % roles.len()].to_string(),
                    })
                    .collect();
                
                Company {
                    name: company_name,
                    employees,
                }
            })
            .collect();
        
        self.log(format!("Generated {} companies", self.companies.len()));
    }
    
    pub fn start_generation(&mut self) {
        if self.companies.is_empty() {
            self.log("No companies generated!");
            return;
        }
        
        self.log("Starting email generation...");
        
        // Serialize companies to JSON
        match serde_json::to_string(&self.companies) {
            Ok(json) => {
                self.log(format!("Companies JSON: {} bytes", json.len()));
                // In a real implementation, we'd spawn the Python process here
                self.log("Would execute: python generate_emails.py --companies-json ...");
            }
            Err(e) => {
                self.log(format!("Error serializing companies: {}", e));
            }
        }
    }
}
