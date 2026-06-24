//! Classification banner with a boot flicker and typewriter title.

use crate::anim;
use crate::app::App;
use crate::theme;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let elapsed = app.clock.elapsed();
    let classification = app
        .meta
        .classification
        .clone()
        .unwrap_or_else(|| "UNCLASSIFIED".to_string());
    let class_color = theme::classification_color(&classification);

    // Flicker the classification label during the first instants of boot.
    let class_visible = if elapsed < anim::BANNER_FLICKER_SECS {
        ((elapsed * 28.0) as i32) % 2 == 0
    } else {
        true
    };

    // Typewriter the title in as boot progresses.
    let full_title = if app.meta.as_of.is_empty() {
        app.meta.title.clone()
    } else {
        format!("{}  ·  as of {}", app.meta.title, app.meta.as_of)
    };
    let reveal = anim::clamp01(app.boot_progress() * 1.3);
    let shown = (full_title.chars().count() as f32 * reveal).round() as usize;
    let title: String = full_title.chars().take(shown).collect();

    let mut spans = vec![
        Span::styled(
            if class_visible {
                format!(" {} ", classification)
            } else {
                format!(" {} ", " ".repeat(classification.len()))
            },
            Style::default().fg(Color::Black).bg(class_color).bold(),
        ),
        Span::raw("   "),
        Span::styled(title, Style::default().fg(Color::Rgb(235, 235, 245)).bold()),
    ];
    // Blinking cursor while the title is still typing.
    if reveal < 1.0 && anim::pulse(elapsed, 10.0) > 0.5 {
        spans.push(Span::styled("▏", Style::default().fg(Color::Rgb(120, 200, 255))));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(60, 70, 90)));

    f.render_widget(
        Paragraph::new(Line::from(spans))
            .block(block)
            .alignment(Alignment::Center),
        area,
    );
}
