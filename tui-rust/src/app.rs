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
    pub email: String,
    pub department: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Company {
    pub company_name: String,
    pub domain: String,
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
    pub log_rx: tokio::sync::mpsc::UnboundedReceiver<String>,
    pub log_tx: tokio::sync::mpsc::UnboundedSender<String>,
    
    // Control
    pub should_quit: bool,
    pub is_generating: bool,
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
        
        let (log_tx, log_rx) = tokio::sync::mpsc::unbounded_channel();
        
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
            log_rx,
            log_tx,
            should_quit: false,
            is_generating: false,
        };
        
        app.log("Loaded environment");
        app
    }
    
    pub fn log(&mut self, msg: impl Into<String>) {
        let msg = msg.into();
        self.logs.push(msg);
        if self.logs.len() > 100 {
            self.logs.remove(0);
        }
    }
    
    pub fn update(&mut self) {
        // Drain logs from background tasks
        while let Ok(msg) = self.log_rx.try_recv() {
            if msg == "__GENERATION_COMPLETE__" {
                self.is_generating = false;
                self.log("Generation process finished.");
            } else {
                self.log(msg);
            }
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
            // Navigate within topics list (0 is the Load button)
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
            // Navigate within topics list (0 is the Load button)
            let max = self.generated_topics.len();
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
        if self.topic_cursor == 0 {
            self.load_topics_from_file();
        } else {
            let actual_idx = self.topic_cursor - 1;
            if actual_idx < self.generated_topics.len() {
                let topic = self.generated_topics.remove(actual_idx);
                self.selected_topics.push(topic);
                // Adjust cursor if needed
                if self.topic_cursor > self.generated_topics.len() && self.topic_cursor > 0 {
                    self.topic_cursor -= 1;
                }
            }
        }
    }
    
    pub fn generate_companies(&mut self) {
        if self.selected_topics.is_empty() {
            self.log("No topics selected!");
            return;
        }
        
        let departments = ["Engineering", "Marketing", "Sales", "HR", "Finance", "Legal", "Product"];
        let titles = ["Manager", "Specialist", "Director", "Lead", "Associate"];
        let names = ["Alice", "Bob", "Charlie", "David", "Eve", "Frank", "Grace", "Heidi"];
        
        self.companies = self.selected_topics
            .iter()
            .enumerate()
            .map(|(idx, topic)| {
                let clean_topic = topic.replace(" ", "");
                let company_name = format!("{}Corp", clean_topic);
                let domain = format!("{}.com", clean_topic.to_lowercase());
                
                let employees = (0..self.company_size.min(10))
                    .map(|i| {
                        let name_idx = (idx + i as usize) % names.len();
                        let name = names[name_idx].to_string();
                        let dept = departments[i as usize % departments.len()];
                        Employee {
                            name: format!("{} {}", name, i),
                            email: format!("{}.{}@{}", name.to_lowercase(), i, domain),
                            department: dept.to_string(),
                            title: titles[i as usize % titles.len()].to_string(),
                        }
                    })
                    .collect();
                
                Company {
                    company_name,
                    domain,
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
        
        if self.is_generating {
            self.log("Already generating!");
            return;
        }
        
        self.is_generating = true;
        self.log("Starting email generation...");
        
        let tx = self.log_tx.clone();
        let roster_path = "../roster.json".to_string(); // Save to root
        let model = self.available_models[self.selected_model_index].clone();
        let api_key = self.api_key.clone();
        let roots = self.chains.to_string();
        let steps = self.total_files.to_string();
        let topic = self.selected_topics.get(0).cloned().unwrap_or_default();
        
        // Save the first company to roster.json (matching Python's format)
        if let Some(company) = self.companies.get(0) {
            match serde_json::to_string_pretty(company) {
                Ok(json) => {
                    if let Err(e) = std::fs::write(&roster_path, json) {
                        self.log(format!("Error saving roster.json: {}", e));
                        self.is_generating = false;
                        return;
                    }
                    self.log(format!("Roster saved for {}", company.company_name));
                }
                Err(e) => {
                    self.log(format!("Error serializing company: {}", e));
                    self.is_generating = false;
                    return;
                }
            }
        } else {
            self.log("Error: No company data to save.");
            self.is_generating = false;
            return;
        }
        
        // Spawn background task
        tokio::spawn(async move {
            import_process_logic(tx, model, api_key, roots, steps, topic).await;
        });
    }
}

async fn import_process_logic(
    tx: tokio::sync::mpsc::UnboundedSender<String>,
    model: String,
    api_key: String,
    roots: String,
    steps: String,
    topic: String,
) {
    use tokio::process::Command;
    use tokio::io::{AsyncBufReadExt, BufReader};
    use std::process::Stdio;

    let mut cmd = Command::new("python3");
    cmd.current_dir(".."); // Run from project root
    cmd.arg("generate_emails.py")
        .arg("--roots").arg(roots)
        .arg("--steps").arg(steps)
        .arg("--roster").arg("roster.json") // It's in the root now
        .arg("--gemini")
        .arg("--model").arg(model);
    
    if !topic.is_empty() {
        cmd.arg("--topic").arg(topic);
    }
    
    // Pass API Key
    cmd.env("GEMINI_API_KEY", api_key);
    
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    match cmd.spawn() {
        Ok(mut child) => {
            let stdout = child.stdout.take().unwrap();
            let stderr = child.stderr.take().unwrap();
            
            let mut stdout_reader = BufReader::new(stdout).lines();
            let mut stderr_reader = BufReader::new(stderr).lines();
            
            let tx_clone = tx.clone();
            tokio::spawn(async move {
                while let Ok(Some(line)) = stdout_reader.next_line().await {
                    let _ = tx_clone.send(line);
                }
            });
            
            let tx_clone2 = tx.clone();
            tokio::spawn(async move {
                while let Ok(Some(line)) = stderr_reader.next_line().await {
                    if !line.trim().is_empty() {
                        let _ = tx_clone2.send(format!("ERROR: {}", line));
                    }
                }
            });
            
            let _ = child.wait().await;
            let _ = tx.send("__GENERATION_COMPLETE__".to_string());
        }
        Err(e) => {
            let _ = tx.send(format!("Failed to start process: {}", e));
            let _ = tx.send("__GENERATION_COMPLETE__".to_string());
        }
    }
}
