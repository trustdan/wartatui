//! Marks popup overlay — shows all currently-set named marks.

use crate::app::App;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

const HDR: Color = Color::Rgb(120, 200, 255);
const KEY: Color = Color::Rgb(232, 200, 90);
const DIM: Color = Color::Rgb(110, 118, 130);

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let marks = app.marks_snapshot();
    let popup_w = area.width.min(48).max(36);
    let inner_h = if marks.is_empty() { 1 } else { marks.len() as u16 };
    let popup_h = (inner_h + 4).clamp(6, area.height);
    let x = area.x + (area.width.saturating_sub(popup_w)) / 2;
    let y = area.y + (area.height.saturating_sub(popup_h)) / 2;
    let rect = Rect { x, y, width: popup_w, height: popup_h };

    f.render_widget(Clear, rect);

    let mut lines: Vec<Line> = vec![Line::from("")];

    if marks.is_empty() {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("no marks set", Style::default().fg(DIM)),
        ]));
    } else {
        for (ch, label) in &marks {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(ch.to_string(), Style::default().fg(KEY).bold()),
                Span::styled("  →  ", Style::default().fg(DIM)),
                Span::styled(label.clone(), Style::default().fg(Color::Rgb(200, 205, 215))),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled("any key to close", Style::default().fg(DIM)),
    ]));

    f.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .title(" MARKS ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(HDR)),
        ),
        rect,
    );
}
