//! The constellation: a radial star-map of the whole org on a Canvas.
//! Faint skeleton of every link, a brightly lit breadcrumb path to the root,
//! a traveling spark, and a pulsing marker on the focused node.

use crate::anim;
use crate::app::App;
use crate::theme;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::symbols::Marker;
use ratatui::widgets::canvas::{Canvas, Line as CanvasLine, Points};
use ratatui::widgets::{Block, Borders};
use ratatui::Frame;
use std::collections::HashSet;

/// Samples per curved link (more = smoother spiral).
const CURVE_SAMPLES: usize = 14;

pub fn render(f: &mut Frame, app: &App, area: Rect) {
    let border = Style::default().fg(Color::Rgb(60, 70, 90));

    // Too small to be meaningful — show an empty labeled box.
    if area.width < 16 || area.height < 5 {
        f.render_widget(
            Block::default()
                .title(" CONSTELLATION ")
                .borders(Borders::ALL)
                .border_style(border),
            area,
        );
        return;
    }

    let elapsed = app.clock.elapsed();
    let pulse = anim::pulse(elapsed, 3.2);

    let focused = app.tree.focused_id().cloned();
    let path_ids: Vec<String> = focused
        .as_ref()
        .map(|id| app.tree.ancestry(id))
        .unwrap_or_default();
    let path_set: HashSet<String> = path_ids.iter().cloned().collect();
    let focused_id = focused.clone().unwrap_or_default();

    let focused_label = app
        .tree
        .focused_node()
        .map(|n| n.label.clone())
        .unwrap_or_default();
    let title = format!(" CONSTELLATION · {} ", focused_label);

    // Inner canvas dimensions (inside the border).
    let w = area.width.saturating_sub(2) as f64;
    let h = area.height.saturating_sub(2) as f64;
    let cx = w / 2.0;
    let cy = h / 2.0;
    // Vertical radius; horizontal scaled ×2 to offset terminal cell aspect.
    let r = (h / 2.0 - 1.0).min(w / 4.0 - 1.0).max(1.0);

    let map = |radius: f32, angle: f32| -> (f64, f64) {
        let rad = radius as f64;
        let ang = angle as f64;
        (cx + rad * ang.cos() * 2.0 * r, cy + rad * ang.sin() * r)
    };

    // Boot: grow the constellation outward in step with the tree cascade.
    let boot = app.boot_progress();
    let reveal = boot * (app.positions.max_depth as f32 + 0.5);

    let canvas = Canvas::default()
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(border),
        )
        .marker(Marker::Braille)
        .x_bounds([0.0, w])
        .y_bounds([0.0, h])
        .paint(move |ctx| {
            // --- faint skeleton: every parent→child link ---
            for (id, node) in app.tree.nodes.iter() {
                let parent = match &node.data.parent {
                    Some(p) => p,
                    None => continue,
                };
                let depth = app.tree.depth_of(id) as f32;
                if depth > reveal {
                    continue;
                }
                let (cp, pp) = match (app.positions.get(id), app.positions.get(parent)) {
                    (Some(a), Some(b)) => (a, b),
                    _ => continue,
                };
                if path_set.contains(id.as_str()) {
                    continue; // drawn brightly below
                }
                let col = theme::node_color_factor(&node.data.org_type, node.data.echelon, 0.28);
                draw_curve(ctx, pp, cp, col, map);
            }

            // --- nodes ---
            for (id, node) in app.tree.nodes.iter() {
                let depth = app.tree.depth_of(id) as f32;
                if depth > reveal {
                    continue;
                }
                let pos = match app.positions.get(id) {
                    Some(p) => p,
                    None => continue,
                };
                let (x, y) = map(pos.0, pos.1);
                let lit = path_set.contains(id.as_str());
                let factor = if lit { 1.15 } else { 0.5 };
                let col = theme::node_color_factor(&node.data.org_type, node.data.echelon, factor);
                ctx.draw(&Points {
                    coords: &[(x, y)],
                    color: col,
                });
            }

            // --- bright breadcrumb path to root ---
            for pair in path_ids.windows(2) {
                if let (Some(a), Some(b)) =
                    (app.positions.get(&pair[0]), app.positions.get(&pair[1]))
                {
                    draw_curve(ctx, a, b, Color::Rgb(120, 200, 255), map);
                }
            }

            // --- traveling spark along the path ---
            if path_ids.len() >= 2 {
                let segs = (path_ids.len() - 1) as f32;
                let t = (elapsed * 0.6).fract() * segs;
                let seg = t.floor() as usize;
                let local = t.fract();
                if let (Some(a), Some(b)) = (
                    app.positions.get(&path_ids[seg]),
                    app.positions.get(&path_ids[seg + 1]),
                ) {
                    let radius = anim::lerp(a.0, b.0, local);
                    let angle = anim::lerp(a.1, b.1, local);
                    let (x, y) = map(radius, angle);
                    ctx.draw(&Points {
                        coords: &[(x, y)],
                        color: Color::Rgb(220, 240, 255),
                    });
                }
            }

            // --- pulsing focused node ---
            if let Some(pos) = app.positions.get(&focused_id) {
                let (x, y) = map(pos.0, pos.1);
                let ring_r = 0.8 + pulse as f64 * 1.4;
                let ring: Vec<(f64, f64)> = (0..12)
                    .map(|i| {
                        let a = i as f64 / 12.0 * std::f64::consts::TAU;
                        (x + a.cos() * ring_r * 2.0, y + a.sin() * ring_r)
                    })
                    .collect();
                let bright = 180 + (pulse * 75.0) as u8;
                ctx.draw(&Points {
                    coords: &ring,
                    color: Color::Rgb(120, bright, 255),
                });
                ctx.draw(&Points {
                    coords: &[(x, y)],
                    color: Color::Rgb(255, 255, 255),
                });
            }
        });

    f.render_widget(canvas, area);
}

/// Draw a curved link by interpolating in polar space (yields a spiral arc).
fn draw_curve<F>(
    ctx: &mut ratatui::widgets::canvas::Context,
    from: (f32, f32),
    to: (f32, f32),
    color: Color,
    map: F,
) where
    F: Fn(f32, f32) -> (f64, f64),
{
    let mut prev = map(from.0, from.1);
    for i in 1..=CURVE_SAMPLES {
        let t = i as f32 / CURVE_SAMPLES as f32;
        let radius = anim::lerp(from.0, to.0, t);
        let angle = anim::lerp(from.1, to.1, t);
        let cur = map(radius, angle);
        ctx.draw(&CanvasLine {
            x1: prev.0,
            y1: prev.1,
            x2: cur.0,
            y2: cur.1,
            color,
        });
        prev = cur;
    }
}
