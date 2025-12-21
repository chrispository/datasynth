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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuantityField {
    TotalFiles,
    Chains,
    PercentAttachments,
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
    pub quantity_field_index: usize, // 0 = total_files, 1 = chains, 2 = percent_attachments
    
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
            quantity_field_index: 0,
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
        } else if self.focus == Focus::Main && self.current_section == Section::Quantity {
            // Cycle through quantity fields
            if self.quantity_field_index > 0 {
                self.quantity_field_index -= 1;
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
        } else if self.focus == Focus::Main && self.current_section == Section::Quantity {
            // Cycle through quantity fields (3 fields: 0, 1, 2)
            if self.quantity_field_index < 2 {
                self.quantity_field_index += 1;
            }
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
    
    pub fn increment_quantity(&mut self) {
        match self.quantity_field_index {
            0 => self.total_files = self.total_files.saturating_add(5).min(500),
            1 => self.chains = self.chains.saturating_add(1).min(50),
            2 => self.percent_attachments = self.percent_attachments.saturating_add(5).min(100),
            _ => {}
        }
    }
    
    pub fn decrement_quantity(&mut self) {
        match self.quantity_field_index {
            0 => self.total_files = self.total_files.saturating_sub(5).max(1),
            1 => self.chains = self.chains.saturating_sub(1).max(1),
            2 => self.percent_attachments = self.percent_attachments.saturating_sub(5),
            _ => {}
        }
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
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        // Load company names from companies.txt
        let company_names: Vec<String> = match std::fs::read_to_string("companies.txt") {
            Ok(content) => content
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            Err(_) => {
                self.log("Warning: companies.txt not found, using fallback");
                vec![
                    "Acme Innovations".to_string(),
                    "Global Solutions Inc.".to_string(),
                    "Apex Corporation".to_string(),
                    "Quantum Innovations".to_string(),
                ]
            }
        };
        
        // Load names from people.txt
        let mut names: Vec<String> = match std::fs::read_to_string("people.txt") {
            Ok(content) => content
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            Err(_) => {
                self.log("Warning: people.txt not found, using fallback names");
                vec![
                    "Maria Rodriguez".to_string(), "James Williams".to_string(),
                    "Fatima Al-Farsi".to_string(), "Carlos Mendoza".to_string(),
                    "Chinwe Okoro".to_string(), "Robert Brown".to_string(),
                    "Amina Diallo".to_string(), "Michael Davis".to_string(),
                    "Isabella Garcia".to_string(), "Mohammed Ali".to_string(),
                    "Yaa Asantewaa".to_string(), "David Miller".to_string(),
                    "William Wilson".to_string(), "Richard Moore".to_string(),
                    "Joseph Taylor".to_string(), "Diego Fernandez".to_string(),
                    "Nia Johnson".to_string(), "Thomas Anderson".to_string(),
                    "Charles Jackson".to_string(), "Olivia Martinez".to_string(),
                ]
            }
        };
        
        // Shuffle names based on current time for randomization
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        
        // Simple Fisher-Yates shuffle using hash-based randomness
        for i in (1..names.len()).rev() {
            let mut hasher = DefaultHasher::new();
            (seed + i as u64).hash(&mut hasher);
            let j = (hasher.finish() as usize) % (i + 1);
            names.swap(i, j);
        }
        
        let departments = ["Engineering", "Marketing", "Sales", "HR", "Finance", "Legal", "Product"];
        let titles = ["Manager", "Specialist", "Director", "Lead", "Associate", "Senior Engineer", "VP"];
        
        // Pick 2 random company names
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);
        let idx1 = (hasher.finish() as usize) % company_names.len();
        seed.wrapping_add(1).hash(&mut hasher);
        let mut idx2 = (hasher.finish() as usize) % company_names.len();
        if idx2 == idx1 { idx2 = (idx1 + 1) % company_names.len(); }
        
        let selected_companies = vec![
            company_names[idx1].clone(),
            company_names[idx2].clone(),
        ];
        
        let employees_per_company = self.company_size.min(12) as usize;
        
        self.companies = selected_companies
            .iter()
            .enumerate()
            .map(|(company_idx, company_name)| {
                // Create domain from company name
                let clean_name: String = company_name
                    .chars()
                    .filter(|c| c.is_alphanumeric() || *c == ' ')
                    .collect::<String>()
                    .split_whitespace()
                    .take(2)
                    .collect::<Vec<_>>()
                    .join("")
                    .to_lowercase();
                let domain = format!("{}.com", clean_name);
                
                // Each company gets a different, non-overlapping slice of shuffled names
                let start_idx = company_idx * employees_per_company;
                let employees = (0..employees_per_company)
                    .map(|i| {
                        let name_idx = (start_idx + i) % names.len();
                        let full_name = names[name_idx].clone();
                        
                        // Create email from name parts
                        let name_parts: Vec<&str> = full_name.split_whitespace().collect();
                        let email_local = if name_parts.len() >= 2 {
                            format!("{}.{}", 
                                name_parts[0].to_lowercase(), 
                                name_parts[1].to_lowercase())
                        } else {
                            full_name.to_lowercase().replace(" ", ".")
                        };
                        
                        Employee {
                            name: full_name,
                            email: format!("{}@{}", email_local, domain),
                            department: departments[i % departments.len()].to_string(),
                            title: titles[i % titles.len()].to_string(),
                        }
                    })
                    .collect();
                
                Company {
                    company_name: company_name.clone(),
                    domain,
                    employees,
                }
            })
            .collect();
        
        self.log(format!("Generated {} companies: {} & {}", 
            self.companies.len(),
            self.companies.get(0).map(|c| c.company_name.as_str()).unwrap_or("?"),
            self.companies.get(1).map(|c| c.company_name.as_str()).unwrap_or("?")
        ));
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
        let attachments = self.percent_attachments.to_string();
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
            import_process_logic(tx, model, api_key, roots, steps, attachments, topic).await;
        });
    }
}

async fn import_process_logic(
    tx: tokio::sync::mpsc::UnboundedSender<String>,
    model: String,
    api_key: String,
    roots: String,
    steps: String,
    attachments: String,
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
        .arg("--attachments").arg(attachments)
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
