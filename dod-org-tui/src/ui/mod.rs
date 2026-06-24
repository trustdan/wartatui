//! Top-level UI layout and rendering orchestration.

mod banner;
mod card;
mod statusline;
mod tree_view;

use crate::app::App;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Frame;

pub fn render(f: &mut Frame, app: &App) {
    let size = f.size();

    // banner / main / status
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(size);

    banner::render(f, app, rows[0]);

    // main: tree (+ optional card)
    let cols = if app.show_card {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(rows[1])
    } else {
        Layout::default()
            .constraints([Constraint::Percentage(100)])
            .split(rows[1])
    };

    tree_view::render(f, app, cols[0]);
    if app.show_card {
        card::render(f, app, cols[1]);
    }

    statusline::render(f, app, rows[2]);
}
