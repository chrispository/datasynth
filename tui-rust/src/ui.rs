use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::{App, Focus, Section};

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Body
            Constraint::Length(8),  // Logs
            Constraint::Length(1),  // Help Footer
        ])
        .split(f.area());

    render_header(f, chunks[0]);
    render_body(f, app, chunks[1]);
    render_logs(f, app, chunks[2]);
    render_help(f, chunks[3]);
}

fn render_header(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" GSYN Email Generator ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(block, area);
}

fn render_help(f: &mut Frame, area: Rect) {
    let help_text = " Arrows: Nav | Enter/Space: Select/Load | G: Gen Companies | S: Start | Q: Quit ";
    let paragraph = Paragraph::new(Line::from(Span::styled(
        help_text,
        Style::default().fg(Color::DarkGray).add_modifier(Modifier::REVERSED),
    )));
    f.render_widget(paragraph, area);
}

fn render_body(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(20), Constraint::Min(0)])
        .split(area);

    render_sidebar(f, app, chunks[0]);
    render_main_content(f, app, chunks[1]);
}

fn render_sidebar(f: &mut Frame, app: &App, area: Rect) {
    let sections = Section::all();
    let items: Vec<ListItem> = sections
        .iter()
        .enumerate()
        .map(|(i, section)| {
            let prefix = if i == app.sidebar_index { "▸ " } else { "  " };
            let content = format!("{}{}", prefix, section.as_str());
            let style = if i == app.sidebar_index && app.focus == Focus::Sidebar {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Sections ")
                .borders(Borders::ALL)
                .border_style(if app.focus == Focus::Sidebar {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::Gray)
                }),
        );

    f.render_widget(list, area);
}

fn render_main_content(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(format!(" {} ", app.current_section.as_str()))
        .borders(Borders::ALL)
        .border_style(if app.focus == Focus::Main {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Gray)
        });

    let inner = block.inner(area);
    f.render_widget(block, area);

    match app.current_section {
        Section::Model => render_model_section(f, app, inner),
        Section::Quantity => render_quantity_section(f, app, inner),
        Section::Topics => render_topics_section(f, app, inner),
        Section::Companies => render_companies_section(f, app, inner),
        Section::Run => render_run_section(f, app, inner),
        _ => render_placeholder(f, app, inner),
    }
}

fn render_quantity_section(f: &mut Frame, app: &App, area: Rect) {
    let text = vec![
        Line::from(Span::styled("Generation Parameters", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![
            Span::raw("Root Threads: "),
            Span::styled(format!("{}", app.chains), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::raw("Total Files: "),
            Span::styled(format!("{}", app.total_files), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(Span::styled("(Editing not yet implemented)", Style::default().fg(Color::DarkGray))),
    ];

    let paragraph = Paragraph::new(text);
    f.render_widget(paragraph, area);
}

fn render_model_section(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    // API Key status
    let key_text = if app.api_key.is_empty() {
        vec![Line::from(Span::styled(
            "API Key: (not set - check .env file)",
            Style::default().fg(Color::Yellow),
        ))]
    } else {
        vec![Line::from(vec![
            Span::raw("API Key: "),
            Span::styled("*".repeat(app.api_key.len()), Style::default().fg(Color::Green)),
            Span::styled(" ✓", Style::default().fg(Color::Green)),
        ])]
    };
    let key_para = Paragraph::new(key_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Gray)),
    );
    f.render_widget(key_para, chunks[0]);

    // Model selection
    let model_items: Vec<ListItem> = app
        .available_models
        .iter()
        .enumerate()
        .map(|(i, model)| {
            let is_selected = i == app.selected_model_index;
            let prefix = if is_selected { "▸ " } else { "  " };
            let style = if is_selected && app.focus == Focus::Main {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(format!("{}{}", prefix, model)).style(style)
        })
        .collect();

    let model_list = List::new(model_items).block(
        Block::default()
            .title(" Select Model (↑/↓) ")
            .borders(Borders::ALL)
            .border_style(if app.focus == Focus::Main {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Gray)
            }),
    );

    f.render_widget(model_list, chunks[1]);
}

fn render_topics_section(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    // Load button
    let load_highlight = app.topic_cursor == 0 && app.focus == Focus::Main;
    let load_btn = Paragraph::new(" [ Load from topics.txt ] ")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(if load_highlight {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Gray)
                }),
        )
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(load_btn, chunks[0]);

    let list_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // Generated topics
    let gen_items: Vec<ListItem> = if app.generated_topics.is_empty() {
        vec![ListItem::new("  (empty)").style(Style::default().fg(Color::DarkGray))]
    } else {
        app.generated_topics
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let actual_idx = i + 1;
                let is_selected = actual_idx == app.topic_cursor && app.focus == Focus::Main;
                let prefix = if is_selected { "▸ " } else { "  " };
                let style = if is_selected {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(format!("{}{}", prefix, t)).style(style)
            })
            .collect()
    };

    let gen_list = List::new(gen_items).block(
        Block::default()
            .title(" Generated ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    // Selected topics
    let sel_items: Vec<ListItem> = if app.selected_topics.is_empty() {
        vec![ListItem::new("  (empty)").style(Style::default().fg(Color::DarkGray))]
    } else {
        app.selected_topics
            .iter()
            .map(|t| ListItem::new(format!("  {}", t)))
            .collect()
    };

    let sel_list = List::new(sel_items).block(
        Block::default()
            .title(" Selected ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)),
    );

    f.render_widget(gen_list, list_chunks[0]);
    f.render_widget(sel_list, list_chunks[1]);
}

fn render_companies_section(f: &mut Frame, app: &App, area: Rect) {
    let text = if app.companies.is_empty() {
        vec![
            Line::from("No companies generated yet."),
            Line::from(""),
            Line::from("Go to Topics → Select Topics → Press 'G' to generate"),
        ]
    } else {
        let mut lines = vec![Line::from(format!("Companies: {}", app.companies.len()))];
        lines.push(Line::from(""));
        
        for company in app.companies.iter().take(5) {
            lines.push(Line::from(format!(
                "• {} ({} employees)",
                company.company_name,
                company.employees.len()
            )));
        }
        
        lines
    };

    let paragraph = Paragraph::new(text);
    f.render_widget(paragraph, area);
}

fn render_run_section(f: &mut Frame, app: &App, area: Rect) {
    let ready = !app.companies.is_empty() && !app.api_key.is_empty();
    
    let mut text = vec![
        Line::from("Ready to Generate"),
        Line::from(""),
        Line::from(format!("Files: {}", app.total_files)),
        Line::from(format!("Companies: {}", app.companies.len())),
        Line::from(format!("API Key: {}", if app.api_key.is_empty() { "Not Set" } else { "Set" })),
        Line::from(""),
    ];

    if app.is_generating {
        text.push(Line::from(Span::styled(
            "Generating... See logs below.",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD | Modifier::ITALIC)
        )));
    } else {
        text.push(Line::from(Span::styled(
            if ready { "[S] Start Generation" } else { "[S] Start (Not Ready)" },
            if ready {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            },
        )));
    }

    let paragraph = Paragraph::new(text);
    f.render_widget(paragraph, area);
}

fn render_placeholder(f: &mut Frame, app: &App, area: Rect) {
    let text = vec![
        Line::from("Feature not implemented yet."),
        Line::from(""),
        Line::from(format!("Section: {}", app.current_section.as_str())),
    ];

    let paragraph = Paragraph::new(text);
    f.render_widget(paragraph, area);
}

fn render_logs(f: &mut Frame, app: &App, area: Rect) {
    let logs: Vec<Line> = app
        .logs
        .iter()
        .rev()
        .take(6)
        .rev()
        .map(|log| Line::from(log.as_str()))
        .collect();

    let paragraph = Paragraph::new(logs).block(
        Block::default()
            .title(" Logs ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Gray)),
    );

    f.render_widget(paragraph, area);
}
