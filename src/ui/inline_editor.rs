//! Inline field editor overlay — opened by `e`, edits leaf fields in place.

use crate::app::App;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

const HDR: Color = Color::Rgb(120, 200, 255);
const DIM: Color = Color::Rgb(110, 118, 130);
const SEL_BG: Color = Color::Rgb(28, 48, 78);
const EDIT_FG: Color = Color::Rgb(232, 200, 90);
const DIRTY: Color = Color::Rgb(210, 100, 80);

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let ed = match &app.inline_editor {
        Some(e) => e,
        None => return,
    };

    // Right-panel sized popup (mirrors card panel width).
    let popup_w = ((area.width * 45) / 100).max(38).min(area.width);
    let rows_needed = (ed.fields.len() * 2 + 6) as u16;
    let popup_h = rows_needed.clamp(10, area.height);
    let x = area.x + area.width.saturating_sub(popup_w);
    let y = area.y + (area.height.saturating_sub(popup_h)) / 2;
    let rect = Rect { x, y, width: popup_w, height: popup_h };

    f.render_widget(Clear, rect);

    let mut lines: Vec<Line> = vec![Line::from("")];

    for (i, (name, value)) in ed.fields.iter().enumerate() {
        let is_sel = i == ed.selected;
        let is_editing = is_sel && ed.editing;

        let label_style = if is_sel {
            Style::default().fg(HDR).bold()
        } else {
            Style::default().fg(DIM)
        };
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(format!("{name}:"), label_style),
        ]));

        let display = if is_editing {
            ed.buffer.clone()
        } else {
            value.clone()
        };
        let display = if display.is_empty() {
            "(empty)".to_string()
        } else {
            display
        };

        if is_editing {
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(display, Style::default().fg(EDIT_FG).bg(SEL_BG)),
                Span::styled("▏", Style::default().fg(EDIT_FG)),
            ]));
        } else if is_sel {
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(display, Style::default().fg(Color::Rgb(210, 215, 225)).bg(SEL_BG)),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(display, Style::default().fg(Color::Rgb(155, 160, 170))),
            ]));
        }
    }

    lines.push(Line::from(""));

    if ed.dirty {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("* unsaved  ", Style::default().fg(DIRTY)),
            Span::styled("s", Style::default().fg(EDIT_FG).bold()),
            Span::styled(" to save", Style::default().fg(DIM)),
        ]));
    }

    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            "j/k select · Enter edit · s save · q/Esc close",
            Style::default().fg(DIM),
        ),
    ]));

    let title = if ed.dirty { " EDIT NODE * " } else { " EDIT NODE " };

    f.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(HDR)),
        ),
        rect,
    );
}
