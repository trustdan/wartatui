//! dod-org-tui — a cinematic terminal navigator for the DoD org structure.

mod anim;
mod app;
mod model;
mod theme;
mod tree;
mod ui;

use app::App;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::error::Error;
use std::io;
use std::time::Duration;

/// Target frame budget (~30 fps) so animation stays smooth when idle.
const FRAME: Duration = Duration::from_millis(33);

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let json_path = args
        .get(1)
        .map(String::as_str)
        .unwrap_or("dod-org-data-research-ingest.json");

    let data = model::load(json_path)?;
    let app = App::new(data);

    // --- terminal setup ---
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run(&mut terminal, app);

    // --- restore terminal ---
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    res?;
    Ok(())
}

fn run<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> io::Result<()> {
    while !app.should_quit {
        terminal.draw(|f| ui::render(f, &app))?;

        // Poll for the frame budget; redraw on timeout to advance animation.
        if event::poll(FRAME)? {
            if let Event::Key(key) = event::read()? {
                // On Windows, only react to presses (avoids duplicate Release events).
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key);
                }
            }
        }
    }
    Ok(())
}
