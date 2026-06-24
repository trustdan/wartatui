//! File picker overlay — opened by `:e`, lists *.json in the data directory.

use crate::app::App;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

const HDR: Color = Color::Rgb(120, 200, 255);
const DIM: Color = Color::Rgb(110, 118, 130);
const SEL_BG: Color = Color::Rgb(28, 48, 78);

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let fp = match &app.file_picker {
        Some(fp) => fp,
        None => return,
    };

    let popup_w = area.width.min(62).max(40);
    let file_rows = fp.files.len().max(1) as u16;
    let popup_h = (file_rows + 5).clamp(7, area.height);
    let x = area.x + (area.width.saturating_sub(popup_w)) / 2;
    let y = area.y + (area.height.saturating_sub(popup_h)) / 2;
    let rect = Rect { x, y, width: popup_w, height: popup_h };

    f.render_widget(Clear, rect);

    let mut lines: Vec<Line> = vec![Line::from("")];

    if fp.files.is_empty() {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("no JSON files found", Style::default().fg(DIM)),
        ]));
    } else {
        for (i, path) in fp.files.iter().enumerate() {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.to_string_lossy().into_owned());

            if i == fp.selected {
                lines.push(Line::from(vec![
                    Span::raw("  "),
                    Span::styled(
                        format!("▶ {}", name),
                        Style::default()
                            .fg(Color::Rgb(232, 200, 90))
                            .bg(SEL_BG)
                            .bold(),
                    ),
                ]));
            } else {
                lines.push(Line::from(vec![
                    Span::raw("    "),
                    Span::styled(name, Style::default().fg(Color::Rgb(200, 205, 215))),
                ]));
            }
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled(
            "j/k navigate · Enter load · Esc/q cancel",
            Style::default().fg(DIM),
        ),
    ]));

    f.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .title(" OPEN FILE (:e) ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(HDR)),
        ),
        rect,
    );
}
