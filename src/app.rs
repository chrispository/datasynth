use serde::{Deserialize, Serialize};

use crate::ui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Model,
    Quantity,
    Topics,
    Companies,
    Preview,
    Run,
    Convert,
    Bates,
    Settings,
}

impl Section {
    pub fn as_str(&self) -> &str {
        match self {
            Section::Model => "Model",
            Section::Quantity => "Quantity",
            Section::Topics => "Topics",
            Section::Companies => "Companies",
            Section::Preview => "Preview",
            Section::Run => "Run",
            Section::Convert => "Convert",
            Section::Bates => "Bates",
            Section::Settings => "Settings",
        }
    }

    pub fn all() -> [Section; 9] {
        [
            Section::Model,
            Section::Quantity,
            Section::Topics,
            Section::Companies,
            Section::Preview,
            Section::Run,
            Section::Convert,
            Section::Bates,
            Section::Settings,
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

const SETTINGS_FILE: &str = "settings.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub selected_model_index: usize,
    #[serde(default)]
    pub provider_index: usize,
    #[serde(default)]
    pub openrouter_model_index: usize,
    pub total_files: u32,
    pub percent_attachments: u32,
    pub selected_topics: Vec<String>,
    pub company_size: u32,
    pub companies: Vec<Company>,
    pub convert_combine: bool,
    #[serde(default = "default_bates_prefix")]
    pub bates_prefix: String,
    #[serde(default)]
    pub bates_separator_index: usize,
    #[serde(default = "default_bates_start")]
    pub bates_start: u32,
    #[serde(default = "default_bates_padding")]
    pub bates_padding: u32,
    #[serde(default = "default_reply_pct")]
    pub reply_percent: u32,
    #[serde(default = "default_forward_pct")]
    pub forward_percent: u32,
    #[serde(default)]
    pub theme_index: usize,
}

fn default_reply_pct() -> u32 { 80 }
fn default_forward_pct() -> u32 { 10 }

fn default_bates_prefix() -> String { "BATES".to_string() }
fn default_bates_start() -> u32 { 1 }
fn default_bates_padding() -> u32 { 7 }

impl Settings {
    pub fn load() -> Option<Self> {
        match std::fs::read_to_string(SETTINGS_FILE) {
            Ok(content) => serde_json::from_str(&content).ok(),
            Err(_) => None,
        }
    }
    
    pub fn save(&self) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(SETTINGS_FILE, json)
    }
}

pub struct App {
    // Navigation
    pub current_section: Section,
    pub sidebar_index: usize,
    pub focus: Focus,

    // Provider & Model
    pub provider_index: usize,
    pub api_key: String,
    pub openrouter_api_key: String,
    pub available_models: Vec<String>,
    pub selected_model_index: usize,
    pub gemini_models: Vec<String>,
    pub openrouter_models: Vec<String>,
    pub gemini_model_index: usize,
    pub openrouter_model_index: usize,
    
    // Quantity
    pub total_files: u32,
    pub percent_attachments: u32,
    pub reply_percent: u32,
    pub forward_percent: u32,
    pub quantity_field_index: usize, // 0=total_files, 1=attachments, 2=reply%, 3=forward%
    
    // Topics
    pub generated_topics: Vec<String>,
    pub selected_topics: Vec<String>,
    pub topic_cursor: usize,
    pub selected_topic_cursor: usize,  // Cursor for the selected topics panel
    pub topic_panel: usize,  // 0 = load button, 1 = generated, 2 = selected
    
    // Companies
    pub company_size: u32,
    pub companies: Vec<Company>,
    
    // Logs
    pub logs: Vec<String>,
    pub log_rx: tokio::sync::mpsc::UnboundedReceiver<String>,
    pub log_tx: tokio::sync::mpsc::UnboundedSender<String>,
    pub log_scroll_offset: Option<usize>, // None = auto-scroll to bottom
    
    // Control
    pub should_quit: bool,
    pub is_generating: bool,
    
    // Convert
    pub convert_subfolders: Vec<String>,
    pub convert_selected_index: usize,
    pub convert_combine: bool,
    pub is_converting: bool,
    pub convert_active_area: usize, // 0: Folder List, 1: Combine Toggle, 2: Convert Button

    // Bates
    pub bates_prefix: String,
    pub bates_separator_index: usize,
    pub bates_start: u32,
    pub bates_padding: u32,
    pub bates_active_area: usize, // 0=file list, 1=prefix, 2=separator, 3=start, 4=padding, 5=stamp button
    pub bates_pdf_files: Vec<String>,
    pub bates_file_index: usize,
    pub is_stamping: bool,

    // Preview
    pub preview_scroll: u16,

    // Settings / Theme
    pub theme_index: usize,
    pub settings_cursor: usize,
}

pub const BATES_SEPARATORS: &[&str] = &["-", "_", "."];
pub const PROVIDERS: &[&str] = &["Gemini", "OpenRouter"];

impl App {
    pub fn new() -> Self {
        // Load API keys from .env
        let api_key = dotenv::var("GEMINI_API_KEY").unwrap_or_default();
        let openrouter_api_key = dotenv::var("OPENROUTER_API_KEY").unwrap_or_default();

        let gemini_models = vec![
            "gemini-2.5-flash".to_string(),
            "gemini-2.5-pro".to_string(),
            "gemini-3-pro-preview".to_string(),
            "gemini-3-flash-preview".to_string(),
        ];

        let openrouter_models = vec![
            "moonshotai/kimi-k2".to_string(),
        ];

        let (log_tx, log_rx) = tokio::sync::mpsc::unbounded_channel();

        // Try to load saved settings
        let saved = Settings::load();

        let provider_index = saved.as_ref().map(|s| s.provider_index).unwrap_or(0);
        let gemini_model_index = saved.as_ref().map(|s| s.selected_model_index).unwrap_or(0);
        let openrouter_model_index = saved.as_ref().map(|s| s.openrouter_model_index).unwrap_or(0);

        let available_models = match provider_index {
            0 => gemini_models.clone(),
            _ => openrouter_models.clone(),
        };
        let selected_model_index = match provider_index {
            0 => gemini_model_index,
            _ => openrouter_model_index,
        };

        let mut app = Self {
            current_section: Section::Model,
            sidebar_index: 0,
            focus: Focus::Sidebar,
            provider_index,
            api_key,
            openrouter_api_key,
            available_models,
            selected_model_index,
            gemini_models,
            openrouter_models,
            gemini_model_index,
            openrouter_model_index,
            total_files: saved.as_ref().map(|s| s.total_files).unwrap_or(25),
            percent_attachments: saved.as_ref().map(|s| s.percent_attachments).unwrap_or(30),
            reply_percent: saved.as_ref().map(|s| s.reply_percent).unwrap_or(80),
            forward_percent: saved.as_ref().map(|s| s.forward_percent).unwrap_or(10),
            quantity_field_index: 0,
            generated_topics: Vec::new(),
            selected_topics: saved.as_ref().map(|s| s.selected_topics.clone()).unwrap_or_default(),
            topic_cursor: 0,
            selected_topic_cursor: 0,
            topic_panel: 0,
            company_size: saved.as_ref().map(|s| s.company_size).unwrap_or(10),
            companies: saved.as_ref().map(|s| s.companies.clone()).unwrap_or_default(),
            logs: vec!["Application initialized".to_string()],
            log_rx,
            log_tx,
            log_scroll_offset: None,
            should_quit: false,
            is_generating: false,
            convert_subfolders: Vec::new(),
            convert_selected_index: 0,
            convert_combine: saved.as_ref().map(|s| s.convert_combine).unwrap_or(false),
            is_converting: false,
            convert_active_area: 0,
            bates_prefix: saved.as_ref().map(|s| s.bates_prefix.clone()).unwrap_or_else(|| "BATES".to_string()),
            bates_separator_index: saved.as_ref().map(|s| s.bates_separator_index).unwrap_or(0),
            bates_start: saved.as_ref().map(|s| s.bates_start).unwrap_or(1),
            bates_padding: saved.as_ref().map(|s| s.bates_padding).unwrap_or(7),
            bates_active_area: 0,
            bates_pdf_files: Vec::new(),
            bates_file_index: 0,
            is_stamping: false,
            preview_scroll: 0,
            theme_index: saved.as_ref().map(|s| s.theme_index).unwrap_or(0),
            settings_cursor: saved.as_ref().map(|s| s.theme_index).unwrap_or(0),
        };

        // Initial scan of output folders
        app.scan_output_folders();
        app.scan_bates_pdfs();
        
        if saved.is_some() {
            app.log("Loaded saved settings");
        } else {
            app.log("No saved settings found, using defaults");
        }
        app
    }
    
    pub fn save_settings(&self) {
        // Save current provider's model index to the right slot
        let (gmi, ormi) = match self.provider_index {
            0 => (self.selected_model_index, self.openrouter_model_index),
            _ => (self.gemini_model_index, self.selected_model_index),
        };
        let settings = Settings {
            selected_model_index: gmi,
            provider_index: self.provider_index,
            openrouter_model_index: ormi,
            total_files: self.total_files,
            percent_attachments: self.percent_attachments,
            selected_topics: self.selected_topics.clone(),
            company_size: self.company_size,
            companies: self.companies.clone(),
            convert_combine: self.convert_combine,
            bates_prefix: self.bates_prefix.clone(),
            bates_separator_index: self.bates_separator_index,
            bates_start: self.bates_start,
            bates_padding: self.bates_padding,
            reply_percent: self.reply_percent,
            forward_percent: self.forward_percent,
            theme_index: self.theme_index,
        };

        if let Err(e) = settings.save() {
            eprintln!("Failed to save settings: {}", e);
        }
    }

    pub fn theme(&self) -> &'static ui::Theme {
        &ui::THEMES[self.theme_index.min(ui::THEMES.len() - 1)].theme
    }

    pub fn toggle_provider(&mut self) {
        // Save current selection
        match self.provider_index {
            0 => self.gemini_model_index = self.selected_model_index,
            _ => self.openrouter_model_index = self.selected_model_index,
        }
        // Toggle
        self.provider_index = (self.provider_index + 1) % PROVIDERS.len();
        // Load new provider's models
        match self.provider_index {
            0 => {
                self.available_models = self.gemini_models.clone();
                self.selected_model_index = self.gemini_model_index;
            }
            _ => {
                self.available_models = self.openrouter_models.clone();
                self.selected_model_index = self.openrouter_model_index;
            }
        }
        self.log(format!("Provider: {}", PROVIDERS[self.provider_index]));
    }

    pub fn current_api_key(&self) -> &str {
        match self.provider_index {
            0 => &self.api_key,
            _ => &self.openrouter_api_key,
        }
    }

    pub fn provider_name(&self) -> &str {
        match self.provider_index {
            0 => "gemini",
            _ => "openrouter",
        }
    }

    pub fn build_preview_prompt(&self) -> Vec<String> {
        let mut lines = Vec::new();
        let terminate = 100u32.saturating_sub(self.reply_percent + self.forward_percent);

        // Thread logic explainer
        lines.push("── Thread Generation Logic ──".into());
        lines.push(String::new());
        lines.push("Each file = one inclusive (leaf) email thread.".into());
        lines.push("The generator creates a root email, then iterates:".into());
        lines.push(format!("  Reply:     {}%  (continue thread with reply)", self.reply_percent));
        lines.push(format!("  Forward:   {}%  (forward to new recipients)", self.forward_percent));
        lines.push(format!("  Terminate: {}%  (end the chain)", terminate));
        lines.push(String::new());
        lines.push("Replies keep the same thread; forwards start a new".into());
        lines.push("thread with new recipients. Each inclusive email".into());
        lines.push("contains the full quoted history of its branch.".into());
        lines.push(String::new());

        // Sample prompt
        lines.push("── Sample Email Prompt ──".into());
        lines.push(String::new());

        let topic = self.selected_topics.get(0)
            .map(|s| s.as_str())
            .unwrap_or("General check-in");

        let (sender_name, sender_title, sender_dept) = if let Some(company) = self.companies.get(0) {
            if let Some(emp) = company.employees.get(0) {
                (emp.name.as_str(), emp.title.as_str(), emp.department.as_str())
            } else {
                ("John Smith", "Manager", "Engineering")
            }
        } else {
            ("John Smith", "Manager", "Engineering")
        };

        let recipients: Vec<&str> = if let Some(company) = self.companies.get(0) {
            company.employees.iter().skip(1).take(2).map(|e| e.name.as_str()).collect()
        } else {
            vec!["Jane Doe", "Bob Wilson"]
        };

        lines.push("Generate a professional business email.".into());
        lines.push(format!("Sender: {} ({} in {})", sender_name, sender_title, sender_dept));
        lines.push(format!("Recipients: {}", recipients.join(", ")));
        lines.push(format!("Topic: {}", topic));
        lines.push("Style/Tone: [randomly selected at runtime]".into());
        lines.push(String::new());
        lines.push("INSTRUCTIONS:".into());
        lines.push("1. This is the start of a new email thread.".into());
        lines.push("2. Create a specific, interesting Subject line".into());
        lines.push("   relevant to the topic (avoid generic titles).".into());
        lines.push("3. Write the body of the email initiating the".into());
        lines.push("   discussion.".into());
        lines.push(String::new());
        lines.push("Format: Subject: [Subject]\\n\\n[Body]".into());
        lines.push(String::new());

        // Reply variant
        lines.push("── Reply Variant (added for replies) ──".into());
        lines.push(String::new());
        lines.push("CONTEXT (Previous Email Thread):".into());
        lines.push("  [previous email body inserted here]".into());
        lines.push(String::new());
        lines.push("+ Address specific points raised in context".into());
        lines.push("+ Do NOT repeat the full history".into());
        lines.push("+ Keep subject consistent (Re: ...)".into());
        lines.push(String::new());

        // Provider info
        lines.push("── Provider ──".into());
        lines.push(String::new());
        lines.push(format!("Provider: {}", PROVIDERS[self.provider_index]));
        lines.push(format!("Model: {}", self.available_models.get(self.selected_model_index).map(|s| s.as_str()).unwrap_or("none")));
        let key = self.current_api_key();
        lines.push(format!("API Key: {}", if key.is_empty() { "NOT SET" } else { "configured" }));

        lines
    }

    pub fn log(&mut self, msg: impl Into<String>) {
        let msg = msg.into();
        self.logs.push(msg);
        if self.logs.len() > 100 {
            self.logs.remove(0);
            // Adjust scroll offset when oldest log is removed
            if let Some(offset) = self.log_scroll_offset.as_mut() {
                *offset = offset.saturating_sub(1);
            }
        }
        // If user hasn't manually scrolled, stay at bottom (None = auto-scroll)
    }

    pub fn scroll_logs_up(&mut self) {
        let visible_height = 14; // 16 - 2 for borders
        if self.logs.len() <= visible_height {
            return;
        }
        let max_offset = self.logs.len() - visible_height;
        let current = self.log_scroll_offset.unwrap_or(max_offset);
        self.log_scroll_offset = Some(current.saturating_sub(1));
    }

    pub fn scroll_logs_down(&mut self) {
        let visible_height = 14; // 16 - 2 for borders
        if self.logs.len() <= visible_height {
            return;
        }
        let max_offset = self.logs.len() - visible_height;
        if let Some(offset) = self.log_scroll_offset {
            let new_offset = (offset + 1).min(max_offset);
            if new_offset >= max_offset {
                // Back at bottom, resume auto-scroll
                self.log_scroll_offset = None;
            } else {
                self.log_scroll_offset = Some(new_offset);
            }
        }
        // If None (auto-scroll), already at bottom, do nothing
    }
    
    pub fn update(&mut self) {
        // Drain logs from background tasks
        while let Ok(msg) = self.log_rx.try_recv() {
            if msg == "__GENERATION_COMPLETE__" {
                self.is_generating = false;
                self.log("Generation process finished.");
            } else if msg == "__CONVERSION_COMPLETE__" {
                self.is_converting = false;
                self.log("Conversion process finished.");
                self.scan_output_folders();
                self.scan_bates_pdfs();
            } else if msg == "__STAMPING_COMPLETE__" {
                self.is_stamping = false;
                self.log("Bates stamping process finished.");
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
            if self.quantity_field_index > 0 {
                self.quantity_field_index -= 1;
            }
        } else if self.focus == Focus::Main && self.current_section == Section::Preview {
            self.preview_scroll = self.preview_scroll.saturating_sub(1);
        } else if self.focus == Focus::Main && self.current_section == Section::Topics {
            // Navigate within topics based on current panel
            match self.topic_panel {
                0 => {}, // Load button, can't go up
                1 => {
                    if self.topic_cursor > 0 {
                        self.topic_cursor -= 1;
                    } else if self.topic_cursor == 0 {
                        // At top of list, go back to load button
                        self.topic_panel = 0;
                    }
                }
                2 => {
                    if self.selected_topic_cursor > 0 {
                        self.selected_topic_cursor -= 1;
                    } else if self.selected_topic_cursor == 0 {
                        // At top of list, go back to generated panel
                        self.topic_panel = 1;
                    }
                }
                _ => {}
            }
        } else if self.focus == Focus::Main && self.current_section == Section::Convert {
            match self.convert_active_area {
                0 => {
                    if self.convert_selected_index > 0 {
                        self.convert_selected_index -= 1;
                    }
                }
                1 => self.convert_active_area = 0,
                2 => self.convert_active_area = 1,
                _ => {}
            }
        } else if self.focus == Focus::Main && self.current_section == Section::Bates {
            match self.bates_active_area {
                0 => {
                    if self.bates_file_index > 0 {
                        self.bates_file_index -= 1;
                    }
                }
                1..=5 => self.bates_active_area -= 1,
                _ => {}
            }
        } else if self.focus == Focus::Main && self.current_section == Section::Settings {
            if self.settings_cursor > 0 {
                self.settings_cursor -= 1;
            }
        }
    }
    
    pub fn navigate_down(&mut self) {
        if self.focus == Focus::Sidebar && self.sidebar_index < Section::all().len() - 1 {
            self.sidebar_index += 1;
            self.current_section = Section::all()[self.sidebar_index];
        } else if self.focus == Focus::Main && self.current_section == Section::Model {
            // Cycle through models
            self.selected_model_index = (self.selected_model_index + 1) % self.available_models.len();
        } else if self.focus == Focus::Main && self.current_section == Section::Quantity {
            if self.quantity_field_index < 3 {
                self.quantity_field_index += 1;
            }
        } else if self.focus == Focus::Main && self.current_section == Section::Preview {
            self.preview_scroll = self.preview_scroll.saturating_add(1);
        } else if self.focus == Focus::Main && self.current_section == Section::Topics {
            // Navigate within topics based on current panel
            match self.topic_panel {
                0 => {
                    // From load button, go to generated list if not empty
                    if !self.generated_topics.is_empty() {
                        self.topic_panel = 1;
                        self.topic_cursor = 0;
                    }
                }
                1 => {
                    if self.topic_cursor < self.generated_topics.len().saturating_sub(1) {
                        self.topic_cursor += 1;
                    } else if self.generated_topics.len() > 0 && self.topic_cursor == self.generated_topics.len() - 1 {
                        // At bottom of list, go to selected panel if it has topics
                        if !self.selected_topics.is_empty() {
                            self.topic_panel = 2;
                            self.selected_topic_cursor = 0;
                        }
                    }
                }
                2 => {
                    if self.selected_topic_cursor < self.selected_topics.len().saturating_sub(1) {
                        self.selected_topic_cursor += 1;
                    }
                    // At bottom of selected list - stay there (it's the last panel)
                }
                _ => {}
            }
        } else if self.focus == Focus::Main && self.current_section == Section::Convert {
            match self.convert_active_area {
                0 => {
                    if !self.convert_subfolders.is_empty() && self.convert_selected_index < self.convert_subfolders.len() - 1 {
                        self.convert_selected_index += 1;
                    } else if !self.convert_subfolders.is_empty() {
                        self.convert_active_area = 1;
                    }
                }
                1 => self.convert_active_area = 2,
                2 => {}
                _ => {}
            }
        } else if self.focus == Focus::Main && self.current_section == Section::Bates {
            match self.bates_active_area {
                0 => {
                    if !self.bates_pdf_files.is_empty() && self.bates_file_index < self.bates_pdf_files.len() - 1 {
                        self.bates_file_index += 1;
                    } else {
                        self.bates_active_area = 1;
                    }
                }
                1..=4 => self.bates_active_area += 1,
                5 => {}
                _ => {}
            }
        } else if self.focus == Focus::Main && self.current_section == Section::Settings {
            if self.settings_cursor < ui::THEMES.len() - 1 {
                self.settings_cursor += 1;
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
            1 => self.percent_attachments = self.percent_attachments.saturating_add(5).min(100),
            2 => {
                let max = 100u32.saturating_sub(self.forward_percent);
                self.reply_percent = self.reply_percent.saturating_add(5).min(max);
            }
            3 => {
                let max = 100u32.saturating_sub(self.reply_percent);
                self.forward_percent = self.forward_percent.saturating_add(5).min(max);
            }
            _ => {}
        }
    }

    pub fn decrement_quantity(&mut self) {
        match self.quantity_field_index {
            0 => self.total_files = self.total_files.saturating_sub(5).max(1),
            1 => self.percent_attachments = self.percent_attachments.saturating_sub(5),
            2 => self.reply_percent = self.reply_percent.saturating_sub(5),
            3 => self.forward_percent = self.forward_percent.saturating_sub(5),
            _ => {}
        }
    }
    
    pub fn load_topics_from_file(&mut self) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        match std::fs::read_to_string("topics.txt") {
            Ok(content) => {
                let mut all_topics: Vec<String> = content
                    .lines()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                // Shuffle topics based on current time for randomization
                let seed = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64;

                // Fisher-Yates shuffle
                for i in (1..all_topics.len()).rev() {
                    let mut hasher = DefaultHasher::new();
                    (seed + i as u64).hash(&mut hasher);
                    let j = (hasher.finish() as usize) % (i + 1);
                    all_topics.swap(i, j);
                }

                // Take first 25 topics
                self.generated_topics = all_topics.into_iter().take(25).collect();
                self.log(format!("Randomly selected {} topics from file", self.generated_topics.len()));
            }
            Err(e) => {
                self.log(format!("Error loading topics.txt: {}", e));
            }
        }
    }
    
    pub fn select_topic(&mut self) {
        match self.topic_panel {
            0 => {
                // Load button - stay on load so user can re-load
                self.load_topics_from_file();
            }
            1 => {
                // Move from generated to selected
                if self.topic_cursor < self.generated_topics.len() {
                    let topic = self.generated_topics.remove(self.topic_cursor);
                    self.selected_topics.push(topic);
                    // Adjust cursor if needed
                    if self.topic_cursor >= self.generated_topics.len() && self.topic_cursor > 0 {
                        self.topic_cursor -= 1;
                    }
                }
            }
            2 => {
                // Move from selected back to generated
                if self.selected_topic_cursor < self.selected_topics.len() {
                    let topic = self.selected_topics.remove(self.selected_topic_cursor);
                    self.generated_topics.push(topic);
                    // Adjust cursor if needed
                    if self.selected_topic_cursor >= self.selected_topics.len() && self.selected_topic_cursor > 0 {
                        self.selected_topic_cursor -= 1;
                    }
                }
            }
            _ => {}
        }
    }
    
    pub fn cycle_topic_panel(&mut self) {
        // Cycle: 0 (load) -> 1 (generated) -> 2 (selected) -> 0
        self.topic_panel = match self.topic_panel {
            0 => if !self.generated_topics.is_empty() { 1 } else if !self.selected_topics.is_empty() { 2 } else { 0 },
            1 => if !self.selected_topics.is_empty() { 2 } else { 0 },
            2 => 0,
            _ => 0,
        };
        // Reset cursors when switching
        self.topic_cursor = 0;
        self.selected_topic_cursor = 0;
    }
    
    pub fn remove_selected_topic(&mut self) {
        if self.topic_panel == 2 && self.selected_topic_cursor < self.selected_topics.len() {
            self.selected_topics.remove(self.selected_topic_cursor);
            // Adjust cursor
            if self.selected_topic_cursor >= self.selected_topics.len() && self.selected_topic_cursor > 0 {
                self.selected_topic_cursor -= 1;
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
        let roster_path = "roster.json".to_string();
        let model = self.available_models[self.selected_model_index].clone();
        let api_key = self.current_api_key().to_string();
        let provider = self.provider_name().to_string();
        let steps = self.total_files.to_string();
        let attachments = self.percent_attachments.to_string();
        let reply_pct = self.reply_percent.to_string();
        let forward_pct = self.forward_percent.to_string();
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
            import_process_logic(tx, provider, model, api_key, steps, attachments, reply_pct, forward_pct, topic).await;
        });
    }

    pub fn scan_output_folders(&mut self) {
        let output_path = std::path::Path::new("output");
        self.convert_subfolders.clear();
        
        if output_path.exists() && output_path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(output_path) {
                for entry in entries.flatten() {
                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_dir() {
                            if let Ok(name) = entry.file_name().into_string() {
                                self.convert_subfolders.push(name);
                            }
                        }
                    }
                }
            }
        }
        self.convert_subfolders.sort();
        // Reset index if out of bounds
        if self.convert_selected_index >= self.convert_subfolders.len() {
            self.convert_selected_index = 0;
        }
    }
    
    pub fn scan_bates_pdfs(&mut self) {
        let output_path = std::path::Path::new("output");
        self.bates_pdf_files.clear();

        if output_path.exists() && output_path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(output_path) {
                for entry in entries.flatten() {
                    if let Ok(name) = entry.file_name().into_string() {
                        if name.ends_with("_combined.pdf") {
                            self.bates_pdf_files.push(name);
                        }
                    }
                }
            }
        }
        self.bates_pdf_files.sort();
        if self.bates_file_index >= self.bates_pdf_files.len() {
            self.bates_file_index = 0;
        }
    }

    pub fn start_bates_stamp(&mut self) {
        if self.bates_pdf_files.is_empty() {
            self.log("No combined PDF files found for Bates stamping!");
            return;
        }

        if self.is_stamping {
            self.log("Already stamping!");
            return;
        }

        if self.bates_file_index >= self.bates_pdf_files.len() {
            return;
        }

        let file_name = self.bates_pdf_files[self.bates_file_index].clone();
        let file_path = format!("output/{}", file_name);
        let prefix = self.bates_prefix.clone();
        let separator = BATES_SEPARATORS[self.bates_separator_index].to_string();
        let start = self.bates_start;
        let padding = self.bates_padding;

        self.is_stamping = true;
        self.log(format!("Starting Bates stamping on {}...", file_name));

        let tx = self.log_tx.clone();

        tokio::spawn(async move {
            bates_process_logic(tx, file_path, prefix, separator, start, padding).await;
        });
    }

    pub fn start_conversion(&mut self) {
        if self.convert_subfolders.is_empty() {
            self.log("No subfolders to convert!");
            return;
        }
        
        if self.is_converting {
            self.log("Already converting!");
            return;
        }
        
        if self.convert_selected_index >= self.convert_subfolders.len() {
            return;
        }
        
        let folder_name = self.convert_subfolders[self.convert_selected_index].clone();
        let combine = self.convert_combine;
        
        self.is_converting = true;
        self.log(format!("Starting PDF conversion for {}...", folder_name));
        
        let tx = self.log_tx.clone();
        // Construct absolute path or relative from root where script is run
        let folder_path = format!("output/{}", folder_name); 
        
        tokio::spawn(async move {
            convert_process_logic(tx, folder_path, combine).await;
        });
    }
}

async fn convert_process_logic(
    tx: tokio::sync::mpsc::UnboundedSender<String>,
    folder_path: String,
    combine: bool,
) {
    use tokio::process::Command;
    use tokio::io::{AsyncBufReadExt, BufReader};
    use std::process::Stdio;

    let mut cmd = Command::new("python3");
    cmd.arg("convert_to_pdf.py")
        .arg("--folder").arg(folder_path);
        
    if combine {
        cmd.arg("--combine");
    }
    
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
            let _ = tx.send("__CONVERSION_COMPLETE__".to_string());
        }
        Err(e) => {
            let _ = tx.send(format!("Failed to start conversion process: {}", e));
            let _ = tx.send("__CONVERSION_COMPLETE__".to_string());
        }
    }
}

async fn bates_process_logic(
    tx: tokio::sync::mpsc::UnboundedSender<String>,
    file_path: String,
    prefix: String,
    separator: String,
    start: u32,
    padding: u32,
) {
    use tokio::process::Command;
    use tokio::io::{AsyncBufReadExt, BufReader};
    use std::process::Stdio;

    let mut cmd = Command::new("python3");
    cmd.arg("bates_stamp.py")
        .arg("--file").arg(file_path)
        .arg("--prefix").arg(prefix)
        .arg("--separator").arg(separator)
        .arg("--start").arg(start.to_string())
        .arg("--padding").arg(padding.to_string());

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
            let _ = tx.send("__STAMPING_COMPLETE__".to_string());
        }
        Err(e) => {
            let _ = tx.send(format!("Failed to start bates stamping process: {}", e));
            let _ = tx.send("__STAMPING_COMPLETE__".to_string());
        }
    }
}

async fn import_process_logic(
    tx: tokio::sync::mpsc::UnboundedSender<String>,
    provider: String,
    model: String,
    api_key: String,
    steps: String,
    attachments: String,
    reply_pct: String,
    forward_pct: String,
    topic: String,
) {
    use tokio::process::Command;
    use tokio::io::{AsyncBufReadExt, BufReader};
    use std::process::Stdio;

    let mut cmd = Command::new("python3");
    cmd.arg("generate_emails.py")
        .arg("--files").arg(steps)
        .arg("--attachments").arg(attachments)
        .arg("--roster").arg("roster.json")
        .arg("--provider").arg(&provider)
        .arg("--model").arg(model)
        .arg("--reply-pct").arg(reply_pct)
        .arg("--forward-pct").arg(forward_pct);

    if !topic.is_empty() {
        cmd.arg("--topic").arg(topic);
    }

    // Pass the appropriate API key
    match provider.as_str() {
        "gemini" => { cmd.env("GEMINI_API_KEY", api_key); }
        "openrouter" => { cmd.env("OPENROUTER_API_KEY", api_key); }
        _ => { cmd.env("GEMINI_API_KEY", api_key); }
    }
    
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
