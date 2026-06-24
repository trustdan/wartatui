//! The administrative tree, with boot cascade and breathing cursor glow.

use crate::anim;
use crate::app::{App, Panel};
use crate::theme;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use super::focus_border;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let tree = &app.tree;
    let elapsed = app.clock.elapsed();
    let boot = app.boot_progress();

    // During boot, reveal the tree outward from the root, depth by depth.
    let max_depth = tree.max_depth() as f32;
    let reveal_front = boot * (max_depth + 1.5);

    // Breathing glow factor for the focused node.
    let glow = 1.0 + 0.18 * anim::pulse(elapsed, 3.5);

    let mut lines: Vec<Line> = Vec::new();
    for (i, id) in tree.flat_list.iter().enumerate() {
        let node = match tree.node(id) {
            Some(n) => n,
            None => continue,
        };
        let depth = tree.depth_of(id);

        // Boot gate: skip nodes past the reveal front; fade in the leading edge.
        let fade = if boot >= 1.0 {
            1.0
        } else if (depth as f32) > reveal_front {
            continue;
        } else {
            anim::clamp01(reveal_front - depth as f32) * 0.7 + 0.3
        };

        let indent = "  ".repeat(depth);
        let has_children = !node.children.is_empty();
        let marker = if has_children {
            if node.expanded {
                "▼ "
            } else {
                "▶ "
            }
        } else {
            "· "
        };

        let focused = i == tree.focused_idx;
        let is_match = tree.is_match(id);

        let factor = fade * if focused { glow } else { 1.0 };
        let color = theme::node_color_factor(&node.data.org_type, node.data.echelon, factor);

        let mut style = Style::default().fg(color);
        if focused {
            style = style
                .bg(Color::Rgb(40, 44, 56))
                .add_modifier(Modifier::BOLD);
        }
        if is_match {
            style = style.add_modifier(Modifier::UNDERLINED);
        }

        let mut spans = vec![
            Span::raw(indent),
            Span::styled(marker, Style::default().fg(color)),
            Span::styled(node.data.label.clone(), style),
        ];
        if focused {
            spans.push(Span::styled(
                "  ◄",
                Style::default().fg(Color::Rgb(120, 200, 255)),
            ));
        }
        lines.push(Line::from(spans));
    }

    let title = format!(" {} ", app.meta.title);
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(focus_border(app.focus == Panel::Tree));

    // Keep the focused line in view by scrolling the paragraph.
    let inner_height = area.height.saturating_sub(2) as usize;
    app.tree_viewport.set(inner_height as u16);
    let scroll = focus_scroll(tree.focused_idx, lines.len(), inner_height);

    f.render_widget(
        Paragraph::new(lines).block(block).scroll((scroll, 0)),
        area,
    );
}

/// Compute a vertical scroll offset that keeps the focused row visible.
fn focus_scroll(focused: usize, total: usize, height: usize) -> u16 {
    if height == 0 || total <= height {
        return 0;
    }
    let max_scroll = total - height;
    // Center the focus when possible.
    let desired = focused.saturating_sub(height / 2);
    desired.min(max_scroll) as u16
}
