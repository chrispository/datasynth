mod app;
mod ui;

use std::io;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::{App, Focus, Section};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env from project root
    let env_path = std::path::Path::new(".env");
    if env_path.exists() {
        dotenv::from_path(env_path).ok();
    }
    
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new();

    // Run app
    let res = run_app(&mut terminal, &mut app).await;
    
    // Save settings before exiting
    app.save_settings();

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err);
    }

    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    use std::time::Duration;

    loop {
        terminal.draw(|f| ui::render(f, app))?;

        // Check for keyboard events without blocking indefinitely
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('c') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                        app.should_quit = true;
                    }
                    KeyCode::Char('q') => {
                        app.should_quit = true;
                    }
                    KeyCode::Up => app.navigate_up(),
                    KeyCode::Down => app.navigate_down(),
                    KeyCode::Right => {
                         if app.focus == Focus::Sidebar {
                            app.navigate_right();
                        }
                    }
                    KeyCode::Enter => {
                        if app.focus == Focus::Main {
                             if app.current_section == Section::Topics {
                                app.select_topic();
                            } else if app.current_section == Section::Companies {
                                app.generate_companies();
                            } else if app.current_section == Section::Run {
                                app.start_generation();
                            } else if app.current_section == Section::Convert {
                                if app.convert_active_area == 2 {
                                    app.start_conversion();
                                }
                            } else if app.current_section == Section::Bates {
                                if app.bates_active_area == 5 {
                                    app.start_bates_stamp();
                                }
                            } else if app.current_section == Section::Settings {
                                app.theme_index = app.settings_cursor;
                                app.log(format!("Theme changed to: {}", ui::THEMES[app.theme_index].name));
                            }
                        }
                    }
                    KeyCode::Tab => {
                        if app.focus == Focus::Main && app.current_section == Section::Topics {
                            app.cycle_topic_panel();
                        } else if app.focus == Focus::Main && app.current_section == Section::Convert {
                            app.convert_active_area = (app.convert_active_area + 1) % 3;
                        } else if app.focus == Focus::Main && app.current_section == Section::Bates {
                            app.bates_active_area = (app.bates_active_area + 1) % 6;
                        }
                    }
                    KeyCode::Left => app.navigate_left(),
                    
                    // Section-specific actions
                    KeyCode::Char('l') if app.focus == Focus::Main && app.current_section == Section::Topics => {
                        app.load_topics_from_file();
                    }
                    KeyCode::Char(' ') if app.focus == Focus::Main && app.current_section == Section::Topics => {
                        app.select_topic();
                    }
                    KeyCode::Char(' ') if app.focus == Focus::Main && app.current_section == Section::Convert => {
                        if app.convert_active_area == 1 {
                            app.convert_combine = !app.convert_combine;
                        }
                    }
                    KeyCode::Char('s') if app.focus == Focus::Main && app.current_section == Section::Run => {
                        app.start_generation();
                    }
                    // Quantity adjustment with + and -
                    KeyCode::Char('+') | KeyCode::Char('=') if app.focus == Focus::Main && app.current_section == Section::Quantity => {
                        app.increment_quantity();
                    }
                    KeyCode::Char('-') | KeyCode::Char('_') if app.focus == Focus::Main && app.current_section == Section::Quantity => {
                        app.decrement_quantity();
                    }
                    // Bates field adjustment with + and -
                    KeyCode::Char('+') | KeyCode::Char('=') if app.focus == Focus::Main && app.current_section == Section::Bates => {
                        match app.bates_active_area {
                            2 => app.bates_separator_index = (app.bates_separator_index + 1) % app::BATES_SEPARATORS.len(),
                            3 => app.bates_start = app.bates_start.saturating_add(1),
                            4 => app.bates_padding = (app.bates_padding + 1).min(12),
                            _ => {}
                        }
                    }
                    KeyCode::Char('-') | KeyCode::Char('_') if app.focus == Focus::Main && app.current_section == Section::Bates => {
                        match app.bates_active_area {
                            2 => app.bates_separator_index = if app.bates_separator_index == 0 { app::BATES_SEPARATORS.len() - 1 } else { app.bates_separator_index - 1 },
                            3 => app.bates_start = app.bates_start.saturating_sub(1).max(1),
                            4 => app.bates_padding = app.bates_padding.saturating_sub(1).max(1),
                            _ => {}
                        }
                    }
                    KeyCode::Backspace | KeyCode::Delete if app.focus == Focus::Main && app.current_section == Section::Topics => {
                        app.remove_selected_topic();
                    }
                    KeyCode::Backspace | KeyCode::Delete if app.focus == Focus::Main && app.current_section == Section::Bates && app.bates_active_area == 1 => {
                        app.bates_prefix.pop();
                    }
                    KeyCode::Char(c) if app.focus == Focus::Main && app.current_section == Section::Bates && app.bates_active_area == 1 && app.bates_prefix.len() < 20 => {
                        app.bates_prefix.push(c);
                    }
                    // Log scrolling
                    KeyCode::PageUp => app.scroll_logs_up(),
                    KeyCode::PageDown => app.scroll_logs_down(),
                    _ => {}
                }
            }
        }

        // Handle background tasks (if any)
        app.update();

        if app.should_quit {
            return Ok(());
        }
    }
}
