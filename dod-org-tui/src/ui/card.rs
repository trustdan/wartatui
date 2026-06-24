//! Unit-data card: statutory authority, echelon, reporting line, and notes.

use crate::app::App;
use crate::theme;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let node = match app.tree.focused_node() {
        Some(n) => n,
        None => return,
    };
    let color = theme::node_color(&node.org_type, node.echelon);

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        node.full_name.clone(),
        Style::default().fg(Color::Rgb(235, 235, 245)).bold(),
    )));
    if let Some(alias) = node.meta_str("displayAlias") {
        lines.push(Line::from(Span::styled(
            format!("“{}”", alias),
            Style::default().fg(Color::Rgb(150, 160, 175)).italic(),
        )));
    }
    lines.push(Line::from(""));

    lines.push(field(
        "Type",
        Span::styled(theme::type_label(&node.org_type), Style::default().fg(color)),
    ));
    lines.push(field(
        "Echelon",
        Span::raw(format!("E{}", node.echelon)),
    ));

    // Reporting line (administrative parent).
    let reports = match &node.parent {
        Some(pid) => app
            .tree
            .node(pid)
            .map(|p| p.data.label.clone())
            .unwrap_or_else(|| "—".into()),
        None => "— (apex)".into(),
    };
    lines.push(field("Reports to", Span::raw(reports)));

    if let Some(conf) = node.meta_str("confidence") {
        lines.push(field("Confidence", confidence_span(conf)));
    }

    // Statutory authority.
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "STATUTORY AUTHORITY",
        Style::default().fg(Color::Rgb(232, 200, 90)).bold(),
    )));
    match &node.source {
        Some(src) => lines.push(Line::from(Span::raw(src.clone()))),
        None => lines.push(Line::from(Span::styled(
            "Unverified — reporting line to confirm",
            Style::default().fg(Color::Rgb(120, 120, 120)).italic(),
        ))),
    }

    // Notes (often HQ + commander).
    if let Some(notes) = node.meta_str("notes") {
        if !notes.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "NOTES",
                Style::default().fg(Color::Rgb(140, 185, 230)).bold(),
            )));
            lines.push(Line::from(Span::raw(notes.to_string())));
        }
    }

    if let Some(url) = node.meta_str("sourceUrl") {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            url.to_string(),
            Style::default().fg(Color::Rgb(90, 140, 190)).underlined(),
        )));
    }

    let block = Block::default()
        .title(format!(" {} ", node.label))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color));

    f.render_widget(
        Paragraph::new(lines).block(block).wrap(Wrap { trim: true }),
        area,
    );
}

fn field<'a>(label: &'a str, value: Span<'a>) -> Line<'a> {
    Line::from(vec![
        Span::styled(
            format!("{:<11}", format!("{}:", label)),
            Style::default().fg(Color::Rgb(150, 160, 175)),
        ),
        value,
    ])
}

fn confidence_span(conf: &str) -> Span<'static> {
    let color = match conf.to_ascii_lowercase().as_str() {
        "high" => Color::Rgb(120, 200, 140),
        "medium" => Color::Rgb(232, 200, 90),
        "low" => Color::Rgb(220, 120, 90),
        _ => Color::Rgb(150, 160, 175),
    };
    Span::styled(conf.to_string(), Style::default().fg(color))
}
