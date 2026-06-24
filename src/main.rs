//! dod-org-tui — a cinematic terminal navigator for the DoD org structure.

mod anim;
mod app;
mod layout_radial;
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

/// Frame budget when animations are running (~30 fps).
const FRAME_ANIM: Duration = Duration::from_millis(33);
/// Frame budget when animations are off — input-only, no idle redraws needed.
const FRAME_STATIC: Duration = Duration::from_millis(100);

const DATA_FILE: &str = "dod-org-data-research-ingest.json";

fn main() -> Result<(), Box<dyn Error>> {
    let (json_path, no_anim) = resolve_args()?;
    let data = model::load(&json_path)?;
    let app = App::new(data, no_anim);

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

/// Parse CLI args. Returns `(data_path, no_anim)`.
fn resolve_args() -> Result<(String, bool), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let mut explicit: Option<String> = None;
    let mut no_anim = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--data" | "-d" => {
                explicit = args.get(i + 1).cloned();
                i += 2;
            }
            "--no-anim" => {
                no_anim = true;
                i += 1;
            }
            "-h" | "--help" => {
                println!(
                    "dod-org-tui — DoD org navigator\n\n\
                     USAGE:\n  dod-org-tui [--data <path>] [--no-anim]\n\n\
                     --data <path>   Data file (default: {DATA_FILE} auto-discovered)\n\
                     --no-anim       Disable all motion; reduces CPU on SSH/battery"
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
            return Ok((path, no_anim));
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
            return Ok((c.to_string_lossy().into_owned(), no_anim));
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
    let frame = if app.no_anim { FRAME_STATIC } else { FRAME_ANIM };
    while !app.should_quit {
        terminal.draw(|f| ui::render(f, &app))?;

        // Poll for the frame budget; redraw on timeout to advance animation.
        if event::poll(frame)? {
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
