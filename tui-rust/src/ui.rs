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
        Section::Convert => render_convert_section(f, app, inner),
        _ => render_placeholder(f, app, inner),
    }
}

fn render_quantity_section(f: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus == Focus::Main;
    
    let fields = [
        ("Total Files", app.total_files, 0),
        ("Attachments %", app.percent_attachments, 1),
    ];
    
    let mut lines = vec![
        Line::from(Span::styled("Generation Parameters", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
    ];
    
    for (name, value, idx) in fields {
        let is_selected = app.quantity_field_index == idx && is_focused;
        let prefix = if is_selected { "▸ " } else { "  " };
        
        let style = if is_selected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        
        lines.push(Line::from(vec![
            Span::styled(format!("{}{}: ", prefix, name), style),
            Span::styled(format!("{}", value), Style::default().fg(Color::Cyan).add_modifier(if is_selected { Modifier::BOLD } else { Modifier::empty() })),
        ]));
    }
    
    lines.push(Line::from(""));
    if is_focused {
        lines.push(Line::from(Span::styled("↑/↓: Select | +/-: Adjust", Style::default().fg(Color::DarkGray))));
    } else {
        lines.push(Line::from(Span::styled("→ to edit values", Style::default().fg(Color::DarkGray))));
    }

    let paragraph = Paragraph::new(lines);
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
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(2)])
        .split(area);

    let is_focused = app.focus == Focus::Main;

    // Load button (panel 0)
    let load_highlight = app.topic_panel == 0 && is_focused;
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

    // Generated topics (panel 1)
    let gen_items: Vec<ListItem> = if app.generated_topics.is_empty() {
        vec![ListItem::new("  (empty)").style(Style::default().fg(Color::DarkGray))]
    } else {
        app.generated_topics
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let is_selected = app.topic_panel == 1 && i == app.topic_cursor && is_focused;
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

    let gen_border_style = if app.topic_panel == 1 && is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Cyan)
    };

    let gen_list = List::new(gen_items).block(
        Block::default()
            .title(" Generated ")
            .borders(Borders::ALL)
            .border_style(gen_border_style),
    );

    // Selected topics (panel 2)
    let sel_items: Vec<ListItem> = if app.selected_topics.is_empty() {
        vec![ListItem::new("  (empty)").style(Style::default().fg(Color::DarkGray))]
    } else {
        app.selected_topics
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let is_selected = app.topic_panel == 2 && i == app.selected_topic_cursor && is_focused;
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

    let sel_border_style = if app.topic_panel == 2 && is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Green)
    };

    let sel_list = List::new(sel_items).block(
        Block::default()
            .title(" Selected ")
            .borders(Borders::ALL)
            .border_style(sel_border_style),
    );

    f.render_widget(gen_list, list_chunks[0]);
    f.render_widget(sel_list, list_chunks[1]);
    
    // Help text
    let help = Paragraph::new(Line::from(vec![
        Span::styled("Tab", Style::default().fg(Color::Cyan)),
        Span::raw(": Switch panels  "),
        Span::styled("Enter/Space", Style::default().fg(Color::Cyan)),
        Span::raw(": Move topic  "),
        Span::styled("Backspace", Style::default().fg(Color::Cyan)),
        Span::raw(": Remove selected"),
    ]));
    f.render_widget(help, chunks[2]);
}

fn render_companies_section(f: &mut Frame, app: &App, area: Rect) {
    if app.companies.is_empty() {
        let text = vec![
            Line::from(Span::styled("No companies generated yet.", Style::default().fg(Color::Yellow))),
            Line::from(""),
            Line::from(Span::styled("[Enter] Generate 2 Random Companies", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(Span::styled("(Press Enter to generate companies with randomized employees)", Style::default().fg(Color::DarkGray))),
        ];
        let paragraph = Paragraph::new(text);
        f.render_widget(paragraph, area);
        return;
    }

    // Split area horizontally for 2 companies
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);
    
    for (idx, company) in app.companies.iter().take(2).enumerate() {
        let mut lines = vec![
            Line::from(Span::styled(&company.company_name, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
            Line::from(Span::styled(format!("Domain: {}", company.domain), Style::default().fg(Color::DarkGray))),
            Line::from(""),
            Line::from(Span::styled("Employees:", Style::default().add_modifier(Modifier::UNDERLINED))),
        ];
        
        for emp in company.employees.iter().take(8) {
            lines.push(Line::from(vec![
                Span::styled(format!("  {} ", emp.name), Style::default().fg(Color::White)),
                Span::styled(format!("({})", emp.title), Style::default().fg(Color::DarkGray)),
            ]));
        }
        
        if company.employees.len() > 8 {
            lines.push(Line::from(Span::styled(
                format!("  ... and {} more", company.employees.len() - 8),
                Style::default().fg(Color::DarkGray)
            )));
        }
        
        let block = Block::default()
            .title(format!(" Company {} ", idx + 1))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(if idx == 0 { Color::Cyan } else { Color::Magenta }));
        
        let paragraph = Paragraph::new(lines).block(block);
        f.render_widget(paragraph, chunks[idx]);
    }
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

fn render_convert_section(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(area);

    let is_focused = app.focus == Focus::Main;

    // Top: Folder List
    let folders: Vec<ListItem> = if app.convert_subfolders.is_empty() {
        vec![ListItem::new(" (No output folders found) ").style(Style::default().fg(Color::DarkGray))]
    } else {
        app.convert_subfolders
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let is_selected = i == app.convert_selected_index;
                let prefix = if is_selected { "▸ " } else { "  " };
                let style = if is_selected && is_focused {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else if is_selected {
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(format!("{}{}", prefix, name)).style(style)
            })
            .collect()
    };

    let list = List::new(folders).block(
        Block::default()
            .title(" Select Output Folder (↑/↓) ")
            .borders(Borders::ALL)
            .border_style(if is_focused && app.convert_active_area == 0 {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Gray)
            }),
    );
    f.render_widget(list, chunks[0]);

    // Bottom: Combined Options Pane
    let check_mark = if app.convert_combine { "[x]" } else { "[ ]" };
    
    let btn_text = if app.is_converting {
        "Converting... (Please wait)"
    } else {
        "[ Enter ] Convert to PDF"
    };
    
    let option_focused = is_focused && app.convert_active_area == 1;
    let button_focused = is_focused && app.convert_active_area == 2;

    let button_style = if button_focused {
        if app.is_converting {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        }
    } else if is_focused && app.convert_active_area == 0 {
         Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let options_text = vec![
        Line::from(vec![
            Span::styled(format!("{} Combine into 1 PDF", check_mark), 
                if option_focused { Style::default().fg(Color::White).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::DarkGray) }
            ),
            Span::styled("  (Space to toggle)", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(""),
        Line::from(Span::styled(btn_text, button_style)),
        Line::from(""),
        Line::from(Span::styled(
            "Converts .eml files and attachments to PDF. Merges them in chronological order.",
            Style::default().fg(Color::DarkGray)
        )),
    ];
    
    let options_para = Paragraph::new(options_text).block(
        Block::default()
            .title(" Options ")
            .borders(Borders::ALL)
            .border_style(if option_focused || button_focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Gray)
            }),
    );
    f.render_widget(options_para, chunks[1]);
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
