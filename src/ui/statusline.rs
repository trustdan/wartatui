//! Bottom status line: mode, breadcrumb, key hints (or search / cmd / mark flash).

use crate::app::{App, Mode, Panel};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

/// How long the mark-set confirmation stays visible (seconds).
const MARK_FLASH_SECS: f32 = 1.5;
/// How long a command error stays visible (seconds).
const CMD_ERROR_SECS: f32 = 1.5;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    // Search bar takes priority.
    if app.tree.search_mode {
        let line = Line::from(vec![
            Span::styled(
                " SEARCH ",
                Style::default().fg(Color::Black).bg(Color::Rgb(232, 200, 90)).bold(),
            ),
            Span::raw(" "),
            Span::styled(
                format!("/{}", app.tree.search_query),
                Style::default().fg(Color::Rgb(232, 200, 90)),
            ),
            Span::styled("▏", Style::default().fg(Color::Rgb(232, 200, 90))),
            Span::styled(
                "   Esc cancel · Enter keep · n/N step matches",
                Style::default().fg(Color::Rgb(110, 118, 130)),
            ),
        ]);
        f.render_widget(Paragraph::new(line), area);
        return;
    }

    // Command bar.
    if let Some(ref buf) = app.cmd_bar {
        let line = Line::from(vec![
            Span::styled(
                " CMD ",
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Rgb(205, 135, 40))
                    .bold(),
            ),
            Span::raw(" "),
            Span::styled(
                format!(":{}", buf),
                Style::default().fg(Color::Rgb(232, 200, 90)),
            ),
            Span::styled("▏", Style::default().fg(Color::Rgb(232, 200, 90))),
            Span::styled(
                "   Esc cancel · Enter run",
                Style::default().fg(Color::Rgb(110, 118, 130)),
            ),
        ]);
        f.render_widget(Paragraph::new(line), area);
        return;
    }

    // Command error flash.
    if let Some((ref msg, set_at)) = app.cmd_error {
        if app.clock.elapsed() - set_at < CMD_ERROR_SECS {
            let line = Line::from(vec![
                Span::styled(
                    " ERR ",
                    Style::default().fg(Color::Black).bg(Color::Rgb(190, 55, 55)).bold(),
                ),
                Span::raw("  "),
                Span::styled(msg.clone(), Style::default().fg(Color::Rgb(220, 100, 100))),
            ]);
            f.render_widget(Paragraph::new(line), area);
            return;
        }
    }

    // Brief mark-set confirmation overrides the hint for 1.5 s.
    if let Some((ch, set_at)) = app.mark_flash {
        if app.clock.elapsed() - set_at < MARK_FLASH_SECS {
            let (mode_label, mode_color) = mode_badge(app);
            let line = Line::from(vec![
                Span::styled(mode_label, Style::default().fg(Color::Black).bg(mode_color).bold()),
                Span::raw("  "),
                Span::styled(
                    format!("mark '{ch}' set  —  press '{ch} to jump back here"),
                    Style::default().fg(Color::Rgb(120, 200, 255)),
                ),
            ]);
            f.render_widget(Paragraph::new(line), area);
            return;
        }
    }

    let (mode_label, mode_color) = mode_badge(app);

    let crumb = app
        .tree
        .focused_id()
        .map(|id| app.tree.breadcrumb(id).join(" › "))
        .unwrap_or_default();

    let hint = match app.focus {
        Panel::Tree => {
            "   hjkl · ^D/^U page · d card · e edit · E editor · : cmd · ? help · q quit"
        }
        Panel::Card => {
            "   j/k scroll · ^D/^U page · Tab→links · ↵ open · h→tree · q quit"
        }
    };

    let line = Line::from(vec![
        Span::styled(mode_label, Style::default().fg(Color::Black).bg(mode_color).bold()),
        Span::raw(" "),
        Span::styled(truncate(&crumb, 44), Style::default().fg(Color::Rgb(170, 178, 190))),
        Span::styled(hint, Style::default().fg(Color::Rgb(95, 102, 115))),
    ]);
    f.render_widget(Paragraph::new(line), area);
}

fn mode_badge(app: &App) -> (&'static str, Color) {
    match app.mode {
        Mode::Admin => (" ADMIN ", Color::Rgb(91, 138, 192)),
        Mode::Ops => (" OPS ", Color::Rgb(232, 140, 60)),
    }
}

fn truncate(s: &str, max: usize) -> String {
    let count = s.chars().count();
    if count <= max {
        s.to_string()
    } else {
        let tail: String = s.chars().skip(count - max + 1).collect();
        format!("…{}", tail)
    }
}
