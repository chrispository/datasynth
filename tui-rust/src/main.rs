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
    // Load .env from parent directory
    let env_path = std::path::Path::new("../.env");
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
                    KeyCode::Right | KeyCode::Enter => {
                        if app.focus == Focus::Sidebar {
                            app.navigate_right();
                        } else if app.focus == Focus::Main && app.current_section == Section::Topics {
                            app.select_topic();
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
                    KeyCode::Char('g') if app.focus == Focus::Main && app.current_section == Section::Companies => {
                        app.generate_companies();
                    }
                    KeyCode::Char('s') if app.focus == Focus::Main && app.current_section == Section::Run => {
                        app.start_generation();
                    }
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
