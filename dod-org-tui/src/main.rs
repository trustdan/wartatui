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
use std::path::PathBuf;
use std::time::Duration;

/// Target frame budget (~30 fps) so animation stays smooth when idle.
const FRAME: Duration = Duration::from_millis(33);

const DATA_FILE: &str = "dod-org-data-research-ingest.json";

fn main() -> Result<(), Box<dyn Error>> {
    let json_path = resolve_data_path()?;
    let data = model::load(&json_path)?;
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

/// Resolve the data file: honor `--data/-d <path>` or a positional path,
/// otherwise search the cwd, next to the executable, and the project root.
fn resolve_data_path() -> Result<String, Box<dyn Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // Explicit path via flag or positional argument.
    let mut explicit: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--data" | "-d" => {
                explicit = args.get(i + 1).cloned();
                i += 2;
            }
            "-h" | "--help" => {
                println!(
                    "dod-org-tui — DoD org navigator\n\n\
                     USAGE:\n  dod-org-tui [--data <path>]\n\n\
                     If --data is omitted, looks for {DATA_FILE} in the current\n\
                     directory, beside the executable, and the project root."
                );
                std::process::exit(0);
            }
            other if !other.starts_with('-') => {
                explicit = Some(other.to_string());
                i += 1;
            }
            _ => i += 1,
        }
    }

    if let Some(path) = explicit {
        if PathBuf::from(&path).is_file() {
            return Ok(path);
        }
        return Err(format!("Data file not found: {}", path).into());
    }

    // Candidate locations for the default file.
    let mut candidates: Vec<PathBuf> = vec![PathBuf::from(DATA_FILE)];
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join(DATA_FILE));
            // target/release/<exe> -> project root is two levels up.
            candidates.push(dir.join("..").join("..").join(DATA_FILE));
        }
    }

    for c in &candidates {
        if c.is_file() {
            return Ok(c.to_string_lossy().into_owned());
        }
    }

    Err(format!(
        "Could not find {DATA_FILE}. Pass --data <path>, or run from a directory \
         that contains it.\nLooked in:\n{}",
        candidates
            .iter()
            .map(|c| format!("  {}", c.display()))
            .collect::<Vec<_>>()
            .join("\n")
    )
    .into())
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
