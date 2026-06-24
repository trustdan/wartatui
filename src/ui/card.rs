//! Unit-data card: statutory authority, echelon, reporting line, and notes.

use crate::app::{App, Panel};
use crate::theme;
use ratatui::layout::{Margin, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
};
use ratatui::Frame;

use super::focus_border;

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

    // Hyperlinks — Tab into the card cycles them; Enter opens the selected one.
    let links = app.current_links();
    if !links.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "LINKS  (Tab to select · ↵ open)",
            Style::default().fg(Color::Rgb(110, 118, 130)).bold(),
        )));
        for (i, url) in links.iter().enumerate() {
            let selected = app.focus == Panel::Card && app.link_focus == Some(i);
            if selected {
                lines.push(Line::from(vec![
                    Span::styled(
                        format!(" {} ", url),
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Rgb(120, 200, 255))
                            .bold(),
                    ),
                    Span::styled(
                        "  ↵ open",
                        Style::default().fg(Color::Rgb(120, 200, 255)).bold(),
                    ),
                ]));
            } else {
                lines.push(Line::from(Span::styled(
                    url.clone(),
                    Style::default().fg(Color::Rgb(90, 140, 190)).underlined(),
                )));
            }
        }
    }

    let focused = app.focus == Panel::Card;
    let title = if focused {
        format!(" {} ▲▼ ", node.label)
    } else {
        format!(" {} ", node.label)
    };
    let border_style = if focused {
        focus_border(true)
    } else {
        Style::default().fg(color)
    };
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(border_style);

    // Estimate wrapped height so paging/scroll bounds match the rendering.
    let inner_w = area.width.saturating_sub(2).max(1) as usize;
    let inner_h = area.height.saturating_sub(2);
    let total: u16 = lines
        .iter()
        .map(|l| ((l.width().max(1) + inner_w - 1) / inner_w) as u16)
        .sum();
    app.card_viewport.set(inner_h);
    app.card_lines.set(total);

    let scroll = app.card_scroll.min(total.saturating_sub(inner_h));

    f.render_widget(
        Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: true })
            .scroll((scroll, 0)),
        area,
    );

    // Scrollbar on the right border when content overflows.
    if total > inner_h {
        let mut state = ScrollbarState::new((total - inner_h) as usize).position(scroll as usize);
        f.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None)
                .thumb_style(Style::default().fg(Color::Rgb(120, 200, 255)))
                .track_style(Style::default().fg(Color::Rgb(50, 58, 72))),
            area.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }),
            &mut state,
        );
    }
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
