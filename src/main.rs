//! Termiban — Terminal Kanban
//! A self-hosted personal Kanban TUI for Linux, macOS and Windows.
//!
//! Architecture (highly modular):
//! * `board`   — pure data model (Task/Column/Board)
//! * `storage` — cross-platform persistence (data_path, load/save)
//! * `app`     — state & business logic (navigation, CRUD, key handling)
//! * `ui`      — rendering only (reads &App, never mutates)
//! * `main`    — terminal lifecycle + event loop (slim orchestrator)

mod app;
mod board;
mod storage;
mod ui;

use app::App;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{error::Error, io, io::Stdout, time::Duration};

fn main() -> Result<(), Box<dyn Error>> {
    // Load persisted state before taking over the terminal
    // so errors go to normal stderr, not the alternate screen.
    let mut app = App::new();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {err:?}");
    }
    Ok(())
}

/// Main loop — draw, poll input, delegate to `App::handle_key`.
///
/// Kept tiny on purpose: all domain logic lives in `app.rs`,
/// all drawing in `ui.rs`. This makes `main.rs` trivial to audit.
fn run_app(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::ui(f, app))?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                // Ignore release/repeat on Windows
                if key.kind != event::KeyEventKind::Press {
                    continue;
                }
                let should_quit = app.handle_key(key);
                if should_quit {
                    return Ok(());
                }
            }
        }
    }
}
