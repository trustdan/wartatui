//! Top-level UI layout and rendering orchestration.

mod banner;
mod card;
mod constellation;
mod statusline;
mod tree_view;

use crate::app::App;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::Frame;

/// Border style for a panel — bright when it holds keyboard focus.
pub(crate) fn focus_border(focused: bool) -> Style {
    if focused {
        Style::default().fg(Color::Rgb(120, 200, 255))
    } else {
        Style::default().fg(Color::Rgb(60, 70, 90))
    }
}

pub fn render(f: &mut Frame, app: &App) {
    let size = f.size();

    // Only show the constellation when there's vertical room for it.
    let show_constellation = size.height >= 18;

    let rows = if show_constellation {
        let const_h = (size.height * 2 / 5).clamp(8, 18);
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),       // banner
                Constraint::Min(6),          // main (tree + card)
                Constraint::Length(const_h), // constellation
                Constraint::Length(1),       // status
            ])
            .split(size)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(size)
    };

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

    if show_constellation {
        constellation::render(f, app, rows[2]);
        statusline::render(f, app, rows[3]);
    } else {
        statusline::render(f, app, rows[2]);
    }
}
