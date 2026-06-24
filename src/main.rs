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
    let data_path = PathBuf::from(&json_path);
    let data = model::load(&json_path)?;
    let app = App::new(data, no_anim, data_path);

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

fn run<B: ratatui::backend::Backend + io::Write>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> io::Result<()> {
    let frame = if app.no_anim { FRAME_STATIC } else { FRAME_ANIM };
    while !app.should_quit {
        terminal.draw(|f| ui::render(f, &app))?;

        // External editor: suspend TUI, spawn editor, resume and reload.
        if app.should_edit_external {
            app.should_edit_external = false;
            if let Some(node_id) = app.tree.focused_id().cloned() {
                let line = find_node_line(&app.data_path, &node_id).unwrap_or(1);
                disable_raw_mode()?;
                execute!(
                    terminal.backend_mut(),
                    LeaveAlternateScreen,
                    DisableMouseCapture
                )?;
                terminal.show_cursor()?;
                let _ = spawn_editor(&app.data_path, line)
                    .and_then(|mut child| child.wait().map(|_| ()));
                enable_raw_mode()?;
                execute!(
                    terminal.backend_mut(),
                    EnterAlternateScreen,
                    EnableMouseCapture
                )?;
                terminal.hide_cursor()?;
                terminal.clear()?;
                if let Ok(data) = model::load(&app.data_path.to_string_lossy()) {
                    let no_anim = app.no_anim;
                    let path = app.data_path.clone();
                    app = App::new(data, no_anim, path);
                }
            }
        }

        // Reload signal: switch to a different data file.
        if let Some(path) = app.should_reload.take() {
            match model::load(&path.to_string_lossy()) {
                Ok(data) => {
                    let no_anim = app.no_anim;
                    app = App::new(data, no_anim, path);
                }
                Err(e) => {
                    app.cmd_error =
                        Some((e.to_string(), app.clock.elapsed()));
                }
            }
        }

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

/// Find the 1-based line number of a node's `"id"` entry in the JSON file.
fn find_node_line(path: &std::path::Path, node_id: &str) -> Option<usize> {
    let content = std::fs::read_to_string(path).ok()?;
    for (i, line) in content.lines().enumerate() {
        if line.contains(r#""id""#) && line.contains(node_id) {
            return Some(i + 1);
        }
    }
    None
}

/// Spawn the user's preferred editor at the given file and line.
fn spawn_editor(
    path: &std::path::Path,
    line: usize,
) -> io::Result<std::process::Child> {
    use std::process::Command;

    if let Ok(editor) = std::env::var("EDITOR") {
        // VS Code uses --goto file:line instead of +N.
        let bin = std::path::Path::new(&editor)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if bin == "code" || bin == "code-insiders" {
            return Command::new(&editor)
                .arg("--goto")
                .arg(format!("{}:{}", path.display(), line))
                .spawn();
        }
        return Command::new(&editor)
            .arg(format!("+{}", line))
            .arg(path)
            .spawn();
    }

    // Platform fallbacks when EDITOR is unset.
    #[cfg(target_os = "windows")]
    {
        Command::new("notepad").arg(path).spawn()
    }
    #[cfg(not(target_os = "windows"))]
    {
        Command::new("vi")
            .arg(format!("+{}", line))
            .arg(path)
            .spawn()
    }
}
