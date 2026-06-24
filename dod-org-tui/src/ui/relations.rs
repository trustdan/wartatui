//! OPS-mode relations rail: the focused unit's operational edges as numbered
//! wormhole-jump targets.

use crate::app::App;
use crate::theme;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let mode_color = Color::Rgb(232, 140, 60);
    let rels = app.relations();

    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled(
            "OPERATIONAL CHAIN",
            Style::default().fg(mode_color).bold(),
        )),
        Line::from(Span::styled(
            "the second chain of command",
            Style::default().fg(Color::Rgb(120, 128, 140)).italic(),
        )),
        Line::from(""),
    ];

    if rels.is_empty() {
        lines.push(Line::from(Span::styled(
            "No operational edges for this unit.",
            Style::default().fg(Color::Rgb(120, 128, 140)),
        )));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Navigate to a force provider, service",
            Style::default().fg(Color::Rgb(100, 108, 120)),
        )));
        lines.push(Line::from(Span::styled(
            "component, or combat-support agency.",
            Style::default().fg(Color::Rgb(100, 108, 120)),
        )));
    } else {
        for (i, r) in rels.iter().enumerate() {
            let rel_color = theme::rgb_scale(theme::relation_rgb(&r.relation), 1.0);
            let other_label = app
                .tree
                .node(&r.other)
                .map(|n| n.data.label.clone())
                .unwrap_or_else(|| r.other.clone());
            let arrow = if r.outgoing { "─▶" } else { "◀─" };

            // "1 ─▶ INDOPACOM"
            lines.push(Line::from(vec![
                Span::styled(
                    format!(" {} ", i + 1),
                    Style::default().fg(Color::Black).bg(mode_color).bold(),
                ),
                Span::styled(format!(" {} ", arrow), Style::default().fg(rel_color)),
                Span::styled(
                    other_label,
                    Style::default().fg(Color::Rgb(235, 235, 245)).bold(),
                ),
            ]));
            // relation label, indented
            lines.push(Line::from(Span::styled(
                format!("     {}", theme::relation_label(&r.relation)),
                Style::default().fg(rel_color),
            )));
            lines.push(Line::from(""));
        }
        lines.push(Line::from(Span::styled(
            "1–9 jump · gd first · o → ADMIN",
            Style::default().fg(Color::Rgb(110, 118, 130)),
        )));
    }

    let block = Block::default()
        .title(" OPS · RELATIONS ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(mode_color));

    f.render_widget(
        Paragraph::new(lines).block(block).wrap(Wrap { trim: true }),
        area,
    );
}
