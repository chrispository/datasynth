use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
    },
    Frame,
};

use crate::app::{App, Focus, Section};

pub struct Theme {
    pub accent: Color,
    pub highlight: Color,
    pub success: Color,
    pub secondary: Color,
    pub text: Color,
    pub text_dim: Color,
    pub border: Color,
}

pub struct NamedTheme {
    pub name: &'static str,
    pub theme: Theme,
}

pub const THEMES: &[NamedTheme] = &[
    NamedTheme {
        name: "Default",
        theme: Theme {
            accent: Color::Cyan,
            highlight: Color::Yellow,
            success: Color::Green,
            secondary: Color::Magenta,
            text: Color::White,
            text_dim: Color::Rgb(100, 100, 100),
            border: Color::Rgb(140, 140, 140),
        },
    },
    NamedTheme {
        name: "Nord",
        theme: Theme {
            accent: Color::Rgb(136, 192, 208),
            highlight: Color::Rgb(235, 203, 139),
            success: Color::Rgb(163, 190, 140),
            secondary: Color::Rgb(180, 142, 173),
            text: Color::Rgb(216, 222, 233),
            text_dim: Color::Rgb(76, 86, 106),
            border: Color::Rgb(67, 76, 94),
        },
    },
    NamedTheme {
        name: "Dracula",
        theme: Theme {
            accent: Color::Rgb(139, 233, 253),
            highlight: Color::Rgb(255, 121, 198),
            success: Color::Rgb(80, 250, 123),
            secondary: Color::Rgb(189, 147, 249),
            text: Color::Rgb(248, 248, 242),
            text_dim: Color::Rgb(98, 114, 164),
            border: Color::Rgb(68, 71, 90),
        },
    },
    NamedTheme {
        name: "Gruvbox",
        theme: Theme {
            accent: Color::Rgb(214, 93, 14),
            highlight: Color::Rgb(250, 189, 47),
            success: Color::Rgb(152, 151, 26),
            secondary: Color::Rgb(211, 134, 155),
            text: Color::Rgb(235, 219, 178),
            text_dim: Color::Rgb(146, 131, 116),
            border: Color::Rgb(80, 73, 69),
        },
    },
    NamedTheme {
        name: "Tokyo Night",
        theme: Theme {
            accent: Color::Rgb(125, 207, 255),
            highlight: Color::Rgb(224, 175, 104),
            success: Color::Rgb(158, 206, 106),
            secondary: Color::Rgb(187, 154, 247),
            text: Color::Rgb(192, 202, 245),
            text_dim: Color::Rgb(86, 95, 137),
            border: Color::Rgb(59, 66, 97),
        },
    },
];

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Body
            Constraint::Length(16), // Logs
            Constraint::Length(1),  // Help Footer
        ])
        .split(f.area());

    render_header(f, app, chunks[0]);
    render_body(f, app, chunks[1]);
    render_logs(f, app, chunks[2]);
    render_help(f, app, chunks[3]);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();
    let block = Block::default()
        .title(" datasynth ")
        .borders(Borders::ALL)
        .style(Style::default().fg(theme.accent));
    f.render_widget(block, area);
}

fn render_help(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();
    let help_text = " Arrows: Nav | Enter/Space: Select/Load | G: Gen Companies | S: Start | PgUp/PgDn: Scroll Logs | Q: Quit ";
    let paragraph = Paragraph::new(Line::from(Span::styled(
        help_text,
        Style::default()
            .fg(theme.text_dim)
            .add_modifier(Modifier::REVERSED),
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
    let theme = app.theme();
    let sections = Section::all();

    let section_line = |i: usize| -> Line {
        let prefix = if i == app.sidebar_index { "▸ " } else { "  " };
        let content = format!("{}{}", prefix, sections[i].as_str());
        let style = if i == app.sidebar_index && app.focus == Focus::Sidebar {
            Style::default()
                .fg(theme.highlight)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.text)
        };
        Line::from(Span::styled(content, style))
    };

    let header_style = Style::default()
        .fg(theme.text_dim)
        .add_modifier(Modifier::BOLD);
    let divider_style = Style::default().fg(theme.border);

    let mut lines: Vec<Line> = Vec::new();

    // CONFIG group
    lines.push(Line::from(Span::styled(" CONFIG", header_style)));
    for i in 0..4 {
        lines.push(section_line(i));
    }

    // Divider
    lines.push(Line::from(Span::styled("──────────────────", divider_style)));

    // ACTIONS group
    lines.push(Line::from(Span::styled(" ACTIONS", header_style)));
    for i in 4..7 {
        lines.push(section_line(i));
    }

    // Divider
    lines.push(Line::from(Span::styled("──────────────────", divider_style)));

    // Settings
    lines.push(section_line(7));

    let paragraph = Paragraph::new(lines).block(
        Block::default()
            .title(" Sections ")
            .borders(Borders::ALL)
            .border_style(if app.focus == Focus::Sidebar {
                Style::default().fg(theme.highlight)
            } else {
                Style::default().fg(theme.border)
            }),
    );

    f.render_widget(paragraph, area);
}

fn render_main_content(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();
    let block = Block::default()
        .title(format!(" {} ", app.current_section.as_str()))
        .borders(Borders::ALL)
        .border_style(if app.focus == Focus::Main {
            Style::default().fg(theme.highlight)
        } else {
            Style::default().fg(theme.border)
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
        Section::Bates => render_bates_section(f, app, inner),
        Section::Settings => render_settings_section(f, app, inner),
    }
}

fn render_quantity_section(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();
    let is_focused = app.focus == Focus::Main;

    let fields = [
        ("Total Files", app.total_files, 0),
        ("Attachments %", app.percent_attachments, 1),
    ];

    let mut lines = vec![
        Line::from(Span::styled(
            "Generation Parameters",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "Each file = one unique email thread",
            Style::default().fg(theme.text_dim),
        )),
        Line::from(""),
    ];

    for (name, value, idx) in fields {
        let is_selected = app.quantity_field_index == idx && is_focused;
        let prefix = if is_selected { "▸ " } else { "  " };

        let style = if is_selected {
            Style::default()
                .fg(theme.highlight)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        lines.push(Line::from(vec![
            Span::styled(format!("{}{}: ", prefix, name), style),
            Span::styled(
                format!("{}", value),
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(if is_selected {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
            ),
        ]));
    }

    lines.push(Line::from(""));
    if is_focused {
        lines.push(Line::from(Span::styled(
            "↑/↓: Select | +/-: Adjust",
            Style::default().fg(theme.border),
        )));
    } else {
        lines.push(Line::from(Span::styled(
            "→ to edit values",
            Style::default().fg(theme.border),
        )));
    }

    let paragraph = Paragraph::new(lines);
    f.render_widget(paragraph, area);
}

fn render_model_section(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    // Hint
    let hint = Paragraph::new(Span::styled(
        "LLM used to generate synthetic email content",
        Style::default().fg(theme.text_dim),
    ));
    f.render_widget(hint, chunks[0]);

    // API Key status
    let key_text = if app.api_key.is_empty() {
        vec![Line::from(Span::styled(
            "API Key: (not set - check .env file)",
            Style::default().fg(theme.highlight),
        ))]
    } else {
        vec![Line::from(vec![
            Span::raw("API Key: "),
            Span::styled(
                "*".repeat(app.api_key.len()),
                Style::default().fg(theme.success),
            ),
            Span::styled(" ✓", Style::default().fg(theme.success)),
        ])]
    };
    let key_para = Paragraph::new(key_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border)),
    );
    f.render_widget(key_para, chunks[1]);

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
                    .fg(theme.highlight)
                    .add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default().fg(theme.accent)
            } else {
                Style::default().fg(theme.text)
            };
            ListItem::new(format!("{}{}", prefix, model)).style(style)
        })
        .collect();

    let model_list = List::new(model_items).block(
        Block::default()
            .title(" Select Model (↑/↓) ")
            .borders(Borders::ALL)
            .border_style(if app.focus == Focus::Main {
                Style::default().fg(theme.highlight)
            } else {
                Style::default().fg(theme.border)
            }),
    );

    f.render_widget(model_list, chunks[2]);
}

fn render_topics_section(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(2),
        ])
        .split(area);

    // Hint
    let hint = Paragraph::new(Span::styled(
        "Subjects for generated email threads (one topic per thread)",
        Style::default().fg(theme.text_dim),
    ));
    f.render_widget(hint, chunks[0]);

    let is_focused = app.focus == Focus::Main;

    // Load button (panel 0)
    let load_highlight = app.topic_panel == 0 && is_focused;
    let (load_style, load_border) = if load_highlight {
        (
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED),
            Style::default().fg(theme.accent),
        )
    } else {
        (
            Style::default().fg(theme.text),
            Style::default().fg(theme.border),
        )
    };
    let load_btn = Paragraph::new(Span::styled("Load from topics.txt", load_style))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(load_border),
        )
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(load_btn, chunks[1]);

    let list_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[2]);

    // Generated topics (panel 1)
    let gen_items: Vec<ListItem> = if app.generated_topics.is_empty() {
        vec![ListItem::new("  (empty)").style(Style::default().fg(theme.text_dim))]
    } else {
        app.generated_topics
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let is_selected = app.topic_panel == 1 && i == app.topic_cursor && is_focused;
                let prefix = if is_selected { "▸ " } else { "  " };
                let style = if is_selected {
                    Style::default()
                        .fg(theme.highlight)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.text)
                };
                ListItem::new(format!("{}{}", prefix, t)).style(style)
            })
            .collect()
    };

    let gen_border_style = if app.topic_panel == 1 && is_focused {
        Style::default().fg(theme.highlight)
    } else {
        Style::default().fg(theme.accent)
    };

    let gen_list = List::new(gen_items).block(
        Block::default()
            .title(" Generated ")
            .borders(Borders::ALL)
            .border_style(gen_border_style),
    );

    // Selected topics (panel 2)
    let sel_items: Vec<ListItem> = if app.selected_topics.is_empty() {
        vec![ListItem::new("  (empty)").style(Style::default().fg(theme.text_dim))]
    } else {
        app.selected_topics
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let is_selected =
                    app.topic_panel == 2 && i == app.selected_topic_cursor && is_focused;
                let prefix = if is_selected { "▸ " } else { "  " };
                let style = if is_selected {
                    Style::default()
                        .fg(theme.highlight)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.text)
                };
                ListItem::new(format!("{}{}", prefix, t)).style(style)
            })
            .collect()
    };

    let sel_border_style = if app.topic_panel == 2 && is_focused {
        Style::default().fg(theme.highlight)
    } else {
        Style::default().fg(theme.success)
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
        Span::styled("Tab", Style::default().fg(theme.accent)),
        Span::raw(": Switch panels  "),
        Span::styled("Enter/Space", Style::default().fg(theme.accent)),
        Span::raw(": Move topic  "),
        Span::styled("Backspace", Style::default().fg(theme.accent)),
        Span::raw(": Remove selected"),
    ]));
    f.render_widget(help, chunks[3]);
}

fn render_companies_section(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();
    let is_focused = app.focus == Focus::Main;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    // Hint
    let hint = Paragraph::new(Span::styled(
        "Fictitious companies whose employees exchange emails",
        Style::default().fg(theme.text_dim),
    ));
    f.render_widget(hint, chunks[0]);

    // Generate button
    let (gen_style, gen_border) = if is_focused {
        (
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED),
            Style::default().fg(theme.accent),
        )
    } else {
        (
            Style::default().fg(theme.text),
            Style::default().fg(theme.border),
        )
    };
    let generate_btn = Paragraph::new(Span::styled("Generate New Companies", gen_style))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(gen_border),
        )
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(generate_btn, chunks[1]);

    // Companies display area
    let companies_area = chunks[2];

    if app.companies.is_empty() {
        let text = vec![
            Line::from(Span::styled(
                "No companies generated yet.",
                Style::default().fg(theme.highlight),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press Enter to generate 2 random companies",
                Style::default().fg(theme.border),
            )),
        ];
        let paragraph = Paragraph::new(text);
        f.render_widget(paragraph, companies_area);
        return;
    }

    // Split area horizontally for 2 companies
    let company_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(companies_area);

    for (idx, company) in app.companies.iter().take(2).enumerate() {
        let mut lines = vec![
            Line::from(Span::styled(
                &company.company_name,
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                format!("Domain: {}", company.domain),
                Style::default().fg(theme.text_dim),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Employees:",
                Style::default().add_modifier(Modifier::UNDERLINED),
            )),
        ];

        for emp in &company.employees {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {} ", emp.name),
                    Style::default().fg(theme.text),
                ),
                Span::styled(
                    format!("({})", emp.title),
                    Style::default().fg(theme.text_dim),
                ),
            ]));
        }

        let block = Block::default()
            .title(format!(" Company {} ", idx + 1))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(if idx == 0 {
                theme.accent
            } else {
                theme.secondary
            }));

        let paragraph = Paragraph::new(lines).block(block);
        f.render_widget(paragraph, company_chunks[idx]);
    }
}

fn render_run_section(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();
    let ready = !app.companies.is_empty() && !app.api_key.is_empty();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    // Hint
    let hint = Paragraph::new(Span::styled(
        "Produces .eml files with realistic email threads in output/",
        Style::default().fg(theme.text_dim),
    ));
    f.render_widget(hint, chunks[0]);

    // Status info
    let key_style = if app.api_key.is_empty() {
        Style::default().fg(theme.highlight)
    } else {
        Style::default().fg(theme.success)
    };
    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Files:     ", Style::default().fg(theme.text)),
            Span::styled(format!("{}", app.total_files), Style::default().fg(theme.accent)),
        ]),
        Line::from(vec![
            Span::styled("  Companies: ", Style::default().fg(theme.text)),
            Span::styled(format!("{}", app.companies.len()), Style::default().fg(theme.accent)),
        ]),
        Line::from(vec![
            Span::styled("  API Key:   ", Style::default().fg(theme.text)),
            Span::styled(
                if app.api_key.is_empty() { "Not Set" } else { "Set" },
                key_style,
            ),
        ]),
    ];
    let paragraph = Paragraph::new(text);
    f.render_widget(paragraph, chunks[1]);

    // Button
    let (btn_text, btn_style, border_color) = if app.is_generating {
        (
            "Generating...",
            Style::default()
                .fg(theme.highlight)
                .add_modifier(Modifier::BOLD | Modifier::ITALIC),
            theme.highlight,
        )
    } else if ready {
        (
            "Start Generation",
            Style::default()
                .fg(theme.success)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED),
            theme.success,
        )
    } else {
        (
            "Start (Not Ready)",
            Style::default().fg(theme.text_dim),
            theme.border,
        )
    };

    let btn = Paragraph::new(Line::from(Span::styled(btn_text, btn_style)))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)),
        )
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(btn, chunks[2]);
}


fn render_convert_section(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    // Hint
    let hint = Paragraph::new(Span::styled(
        "Convert generated .eml files and attachments to PDF",
        Style::default().fg(theme.text_dim),
    ));
    f.render_widget(hint, chunks[0]);

    let is_focused = app.focus == Focus::Main;

    // Top: Folder List
    let folders: Vec<ListItem> = if app.convert_subfolders.is_empty() {
        vec![ListItem::new(" (No output folders found) ")
            .style(Style::default().fg(theme.text_dim))]
    } else {
        app.convert_subfolders
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let is_selected = i == app.convert_selected_index;
                let prefix = if is_selected { "▸ " } else { "  " };
                let style = if is_selected && is_focused {
                    Style::default()
                        .fg(theme.highlight)
                        .add_modifier(Modifier::BOLD)
                } else if is_selected {
                    Style::default()
                        .fg(theme.text)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.text)
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
                Style::default().fg(theme.highlight)
            } else {
                Style::default().fg(theme.border)
            }),
    );
    f.render_widget(list, chunks[1]);

    // Bottom: Options pane
    let option_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Toggle
            Constraint::Length(1), // Spacer
            Constraint::Length(3), // Button
            Constraint::Min(0),   // Description
        ])
        .split(chunks[2]);

    let option_focused = is_focused && app.convert_active_area == 1;
    let button_focused = is_focused && app.convert_active_area == 2;

    // Checkbox toggle
    let toggle = if app.convert_combine { "●" } else { "○" };
    let toggle_line = Line::from(vec![
        Span::styled(
            format!(" {} Combine into 1 PDF ", toggle),
            if option_focused {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            },
        ),
        Span::styled("  Space to toggle", Style::default().fg(theme.border)),
    ]);
    f.render_widget(Paragraph::new(toggle_line), option_chunks[0]);

    // Convert button
    let (btn_text, btn_style, border_color) = if app.is_converting {
        (
            "Converting...",
            Style::default()
                .fg(theme.highlight)
                .add_modifier(Modifier::BOLD | Modifier::ITALIC),
            theme.highlight,
        )
    } else if button_focused {
        (
            "Convert to PDF",
            Style::default()
                .fg(theme.success)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED),
            theme.success,
        )
    } else {
        (
            "Convert to PDF",
            Style::default().fg(theme.text),
            theme.border,
        )
    };

    let btn = Paragraph::new(Line::from(Span::styled(btn_text, btn_style)))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)),
        )
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(btn, option_chunks[2]);

    // Description
    let desc = Paragraph::new(Span::styled(
        " Merges .eml and attachments in chronological order.",
        Style::default().fg(theme.text_dim),
    ));
    f.render_widget(desc, option_chunks[3]);
}

fn render_bates_section(f: &mut Frame, app: &App, area: Rect) {
    use crate::app::BATES_SEPARATORS;

    let theme = app.theme();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),      // Hint
            Constraint::Percentage(35), // File list
            Constraint::Percentage(40), // Config fields
            Constraint::Percentage(25), // Button + Help
        ])
        .split(area);

    // Hint
    let hint = Paragraph::new(Span::styled(
        "Add sequential page numbers to combined PDFs for legal reference",
        Style::default().fg(theme.text_dim),
    ));
    f.render_widget(hint, chunks[0]);

    let is_focused = app.focus == Focus::Main;

    // Top: PDF file selector
    let files: Vec<ListItem> = if app.bates_pdf_files.is_empty() {
        vec![ListItem::new(" (No combined PDFs found) ").style(Style::default().fg(theme.text_dim))]
    } else {
        app.bates_pdf_files
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let is_selected = i == app.bates_file_index;
                let prefix = if is_selected { "▸ " } else { "  " };
                let style = if is_selected && is_focused && app.bates_active_area == 0 {
                    Style::default()
                        .fg(theme.highlight)
                        .add_modifier(Modifier::BOLD)
                } else if is_selected {
                    Style::default()
                        .fg(theme.text)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.text)
                };
                ListItem::new(format!("{}{}", prefix, name)).style(style)
            })
            .collect()
    };

    let file_list = List::new(files).block(
        Block::default()
            .title(" Combined PDFs (↑/↓) ")
            .borders(Borders::ALL)
            .border_style(if is_focused && app.bates_active_area == 0 {
                Style::default().fg(theme.highlight)
            } else {
                Style::default().fg(theme.border)
            }),
    );
    f.render_widget(file_list, chunks[1]);

    // Middle: Config fields
    let current_prefix = &app.bates_prefix;
    let current_sep = BATES_SEPARATORS[app.bates_separator_index];
    let preview = format!(
        "{}{}{}",
        current_prefix,
        current_sep,
        format!(
            "{:0>width$}",
            app.bates_start,
            width = app.bates_padding as usize
        )
    );

    let field_style = |area_idx: usize| -> Style {
        if is_focused && app.bates_active_area == area_idx {
            Style::default()
                .fg(theme.highlight)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        }
    };

    let value_style = |area_idx: usize| -> Style {
        if is_focused && app.bates_active_area == area_idx {
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.accent)
        }
    };

    let btn_focused = is_focused && app.bates_active_area == 5;

    let config_lines = vec![
        Line::from(Span::styled(
            "Bates Configuration",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                if is_focused && app.bates_active_area == 1 {
                    "▸ "
                } else {
                    "  "
                },
                field_style(1),
            ),
            Span::styled("Prefix:    ", field_style(1)),
            Span::styled(current_prefix, value_style(1)),
            Span::styled("  (type to edit)", Style::default().fg(theme.border)),
        ]),
        Line::from(vec![
            Span::styled(
                if is_focused && app.bates_active_area == 2 {
                    "▸ "
                } else {
                    "  "
                },
                field_style(2),
            ),
            Span::styled("Separator: ", field_style(2)),
            Span::styled(format!("\"{}\"", current_sep), value_style(2)),
            Span::styled("  (+/- to cycle)", Style::default().fg(theme.border)),
        ]),
        Line::from(vec![
            Span::styled(
                if is_focused && app.bates_active_area == 3 {
                    "▸ "
                } else {
                    "  "
                },
                field_style(3),
            ),
            Span::styled("Start:     ", field_style(3)),
            Span::styled(format!("{}", app.bates_start), value_style(3)),
            Span::styled("  (+/- to adjust)", Style::default().fg(theme.border)),
        ]),
        Line::from(vec![
            Span::styled(
                if is_focused && app.bates_active_area == 4 {
                    "▸ "
                } else {
                    "  "
                },
                field_style(4),
            ),
            Span::styled("Padding:   ", field_style(4)),
            Span::styled(format!("{}", app.bates_padding), value_style(4)),
            Span::styled("  (+/- to adjust)", Style::default().fg(theme.border)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Preview: ", Style::default().fg(theme.text_dim)),
            Span::styled(
                preview,
                Style::default()
                    .fg(theme.secondary)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    let config_para = Paragraph::new(config_lines).block(
        Block::default()
            .title(" Settings ")
            .borders(Borders::ALL)
            .border_style(if is_focused && app.bates_active_area >= 1 {
                Style::default().fg(theme.highlight)
            } else {
                Style::default().fg(theme.border)
            }),
    );
    f.render_widget(config_para, chunks[2]);

    // Bottom: Button + help
    let bottom_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(chunks[3]);

    let (btn_text, btn_style, border_color) = if app.is_stamping {
        (
            "Stamping...",
            Style::default()
                .fg(theme.highlight)
                .add_modifier(Modifier::BOLD | Modifier::ITALIC),
            theme.highlight,
        )
    } else if btn_focused {
        (
            "Stamp Bates Numbers",
            Style::default()
                .fg(theme.success)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED),
            theme.success,
        )
    } else {
        (
            "Stamp Bates Numbers",
            Style::default().fg(theme.text),
            theme.border,
        )
    };

    let btn = Paragraph::new(Line::from(Span::styled(btn_text, btn_style)))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)),
        )
        .alignment(ratatui::layout::Alignment::Center);
    f.render_widget(btn, bottom_chunks[0]);

    let help = Paragraph::new(Line::from(vec![
        Span::styled("Tab", Style::default().fg(theme.accent)),
        Span::raw(": Cycle fields  "),
        Span::styled("+/-", Style::default().fg(theme.accent)),
        Span::raw(": Adjust  "),
        Span::styled("Enter", Style::default().fg(theme.accent)),
        Span::raw(": Stamp"),
    ]));
    f.render_widget(help, bottom_chunks[1]);
}

fn render_settings_section(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();
    let is_focused = app.focus == Focus::Main;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(8)])
        .split(area);

    // Top: Theme list
    let items: Vec<ListItem> = THEMES
        .iter()
        .enumerate()
        .map(|(i, named)| {
            let is_current = i == app.theme_index;
            let is_cursor = i == app.settings_cursor && is_focused;
            let prefix = if is_cursor { "▸ " } else { "  " };
            let suffix = if is_current { " (active)" } else { "" };
            let style = if is_cursor {
                Style::default()
                    .fg(theme.highlight)
                    .add_modifier(Modifier::BOLD)
            } else if is_current {
                Style::default().fg(theme.accent)
            } else {
                Style::default().fg(theme.text)
            };
            ListItem::new(format!("{}{}{}", prefix, named.name, suffix)).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(" Theme (↑/↓ Enter to apply) ")
            .borders(Borders::ALL)
            .border_style(if is_focused {
                Style::default().fg(theme.highlight)
            } else {
                Style::default().fg(theme.border)
            }),
    );
    f.render_widget(list, chunks[0]);

    // Bottom: Color swatch preview for the highlighted theme
    let preview_theme = &THEMES[app.settings_cursor.min(THEMES.len() - 1)].theme;
    let preview_name = THEMES[app.settings_cursor.min(THEMES.len() - 1)].name;

    let preview_lines = vec![
        Line::from(Span::styled(
            format!("Preview: {}", preview_name),
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled("  accent    ", Style::default().fg(preview_theme.accent))),
        Line::from(Span::styled("  highlight ", Style::default().fg(preview_theme.highlight))),
        Line::from(Span::styled("  success   ", Style::default().fg(preview_theme.success))),
        Line::from(Span::styled("  secondary ", Style::default().fg(preview_theme.secondary))),
        Line::from(Span::styled("  text      ", Style::default().fg(preview_theme.text))),
        Line::from(Span::styled("  dim       ", Style::default().fg(preview_theme.text_dim))),
    ];

    let preview_para = Paragraph::new(preview_lines).block(
        Block::default()
            .title(" Color Preview ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border)),
    );
    f.render_widget(preview_para, chunks[1]);
}

fn render_logs(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.theme();
    let logs: Vec<Line> = app
        .logs
        .iter()
        .map(|log| Line::from(log.as_str()))
        .collect();

    let log_block = Block::default()
        .title(" Logs ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border));

    // Visible height = area height minus 2 for borders
    let visible_height = area.height.saturating_sub(2) as usize;
    let total_logs = logs.len();

    // Calculate scroll offset: auto-scroll to bottom unless user has scrolled up
    let scroll_offset = if total_logs > visible_height {
        let max_offset = total_logs - visible_height;
        // Use app.log_scroll_offset if set, otherwise auto-scroll to bottom
        app.log_scroll_offset.unwrap_or(max_offset)
    } else {
        0
    };

    let paragraph = Paragraph::new(logs)
        .block(log_block)
        .scroll((scroll_offset as u16, 0));

    f.render_widget(paragraph, area);

    // Render scrollbar if content overflows
    if total_logs > visible_height {
        let mut scrollbar_state =
            ScrollbarState::new(total_logs.saturating_sub(visible_height)).position(scroll_offset);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"));
        f.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
    }
}
