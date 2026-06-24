//! Bottom status line: mode, breadcrumb, and key hints (or the search input).

use crate::app::{App, Mode, Panel};
use ratatui::layout::Rect;
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
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
                "   (Esc cancel · Enter keep)",
                Style::default().fg(Color::Rgb(110, 118, 130)),
            ),
        ]);
        f.render_widget(Paragraph::new(line), area);
        return;
    }

    let (mode_label, mode_color) = match app.mode {
        Mode::Admin => (" ADMIN ", Color::Rgb(91, 138, 192)),
        Mode::Ops => (" OPS ", Color::Rgb(232, 140, 60)),
    };

    let crumb = app
        .tree
        .focused_id()
        .map(|id| app.tree.breadcrumb(id).join(" › "))
        .unwrap_or_default();

    let hint = match app.focus {
        Panel::Tree => {
            "   hjkl · [ ] same-type · { } sib · m/' marks · / find · o ops · q quit"
        }
        Panel::Card => "   j/k scroll · D/U page · Tab→links · ↵ open · h→tree · q quit",
    };

    let line = Line::from(vec![
        Span::styled(
            mode_label,
            Style::default().fg(Color::Black).bg(mode_color).bold(),
        ),
        Span::raw(" "),
        Span::styled(truncate(&crumb, 50), Style::default().fg(Color::Rgb(170, 178, 190))),
        Span::styled(hint, Style::default().fg(Color::Rgb(95, 102, 115))),
    ]);
    f.render_widget(Paragraph::new(line), area);
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
