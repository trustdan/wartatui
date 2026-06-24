//! The constellation: a radial star-map of the whole org on a Canvas.
//! Faint skeleton of every link, a brightly lit breadcrumb path to the root,
//! a traveling spark, and a pulsing marker on the focused node.

use crate::anim;
use crate::app::{App, Mode, JUMP_SECS};
use crate::theme;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style, Stylize};
use ratatui::symbols::Marker;
use ratatui::text::{Line, Span};
use ratatui::widgets::canvas::{Canvas, Line as CanvasLine, Points};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
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
    let pulse = if app.no_anim { 0.5 } else { anim::pulse(elapsed, 3.2) };

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
    // Fill the whole panel as an ellipse so the outermost ring always lands
    // just inside the border — nothing is ever cut off.
    let rx = (w / 2.0 - 2.0).max(1.0);
    let ry = (h / 2.0 - 1.0).max(1.0);

    let map = |radius: f32, angle: f32| -> (f64, f64) {
        let rad = radius as f64;
        let ang = angle as f64;
        (cx + rad * ang.cos() * rx, cy + rad * ang.sin() * ry)
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

            // --- operational edges (OPS mode): the second chain of command ---
            if app.mode == Mode::Ops {
                let center = map(0.0, 0.0);
                for e in app.edges.iter() {
                    let (ps, pt) =
                        match (app.positions.get(&e.source), app.positions.get(&e.target)) {
                            (Some(a), Some(b)) => (a, b),
                            _ => continue,
                        };
                    let a = map(ps.0, ps.1);
                    let b = map(pt.0, pt.1);
                    let involved =
                        (e.source == focused_id || e.target == focused_id) && !app.no_anim;
                    let rgb = theme::relation_rgb(&e.relation);
                    draw_edge(ctx, a, b, center, rgb, involved, elapsed);
                }
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
            if path_ids.len() >= 2 && !app.no_anim {
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

            // --- wormhole comet: a streak racing along the jump arc ---
            if !app.no_anim {
            if let Some((from, to, start)) = &app.transition {
                let dt = (elapsed - start) / JUMP_SECS;
                if (0.0..1.0).contains(&dt) {
                    if let (Some(pf), Some(pt)) =
                        (app.positions.get(from), app.positions.get(to))
                    {
                        let center = map(0.0, 0.0);
                        let a = map(pf.0, pf.1);
                        let b = map(pt.0, pt.1);
                        let c = ctrl_point(a, b, center);
                        let bezier = |t: f64| -> (f64, f64) {
                            let u = 1.0 - t;
                            (
                                u * u * a.0 + 2.0 * u * t * c.0 + t * t * b.0,
                                u * u * a.1 + 2.0 * u * t * c.1 + t * t * b.1,
                            )
                        };
                        let head = anim::smoothstep(dt) as f64;

                        // Fading trail behind the head.
                        const TRAIL: usize = 10;
                        for k in 0..TRAIL {
                            let t = (head - k as f64 * 0.05).max(0.0);
                            let (x, y) = bezier(t);
                            let fade = 1.0 - k as f32 / TRAIL as f32;
                            ctx.draw(&Points {
                                coords: &[(x, y)],
                                color: Color::Rgb(
                                    (90.0 + 120.0 * fade) as u8,
                                    (170.0 + 70.0 * fade) as u8,
                                    255,
                                ),
                            });
                        }

                        // Bright head with a small halo.
                        let (hx, hy) = bezier(head);
                        let halo: Vec<(f64, f64)> = (0..10)
                            .map(|i| {
                                let ang = i as f64 / 10.0 * std::f64::consts::TAU;
                                (hx + ang.cos() * 2.6, hy + ang.sin() * 1.3)
                            })
                            .collect();
                        ctx.draw(&Points {
                            coords: &halo,
                            color: Color::Rgb(150, 220, 255),
                        });
                        ctx.draw(&Points {
                            coords: &[(hx, hy)],
                            color: Color::Rgb(255, 255, 255),
                        });

                        // Pulse the destination as the head nears it.
                        if dt > 0.6 {
                            let (dx, dy) = bezier(1.0);
                            let burst = ((dt - 0.6) / 0.4) as f64;
                            let rr = 1.0 + burst * 3.0;
                            let flash: Vec<(f64, f64)> = (0..14)
                                .map(|i| {
                                    let ang = i as f64 / 14.0 * std::f64::consts::TAU;
                                    (dx + ang.cos() * rr * 2.0, dy + ang.sin() * rr)
                                })
                                .collect();
                            ctx.draw(&Points {
                                coords: &flash,
                                color: Color::Rgb(255, 255, 255),
                            });
                        }
                    }
                }
            }
            } // end !no_anim
        });

    f.render_widget(canvas, area);

    render_orientation(f, app, area);
}

/// A small "you are here" label — major category › sub-category — parked in
/// whichever blank corner matches the focused node's direction on the circle.
fn render_orientation(f: &mut Frame, app: &App, area: Rect) {
    let focused_id = match app.tree.focused_id() {
        Some(id) => id.clone(),
        None => return,
    };
    let chain = app.tree.ancestry(&focused_id); // [root, SecDef, major, sub, ...]

    // Major category = the depth-2 ancestor; sub = depth-3.
    let major_id = chain.get(2).or_else(|| chain.last());
    let sub_id = chain.get(3);
    let major_id = match major_id {
        Some(id) => id,
        None => return,
    };

    let major_node = match app.tree.node(major_id) {
        Some(n) => n,
        None => return,
    };
    let major_label = major_node.data.label.clone();
    let major_color = theme::node_color(&major_node.data.org_type, major_node.data.echelon);
    let sub_label = sub_id
        .and_then(|id| app.tree.node(id))
        .map(|n| n.data.label.clone());

    // Build the label lines.
    let mut lines: Vec<Line> = vec![Line::from(vec![
        Span::styled("◆ ", Style::default().fg(major_color)),
        Span::styled(major_label.clone(), Style::default().fg(major_color).bold()),
    ])];
    if let Some(sub) = &sub_label {
        lines.push(Line::from(Span::styled(
            format!("  {}", sub),
            Style::default().fg(Color::Rgb(150, 160, 175)),
        )));
    }

    // Box size.
    let label_w = major_label.chars().count() + 2;
    let sub_w = sub_label.as_ref().map(|s| s.chars().count() + 2).unwrap_or(0);
    let boxw = (label_w.max(sub_w) as u16 + 2).min(area.width.saturating_sub(2));
    let boxh = lines.len() as u16;
    if boxw == 0 || boxh == 0 || area.width < boxw + 4 || area.height < boxh + 2 {
        return;
    }

    // Pick the corner matching the node's direction (canvas y is up).
    let angle = app.positions.get(&focused_id).map(|p| p.1).unwrap_or(0.0);
    let right = angle.cos() >= 0.0;
    let top = angle.sin() >= 0.0;

    let x = if right {
        area.x + area.width - boxw - 1
    } else {
        area.x + 1
    };
    let y = if top {
        area.y + 1
    } else {
        area.y + area.height - boxh - 1
    };

    let rect = Rect {
        x,
        y,
        width: boxw,
        height: boxh,
    };
    f.render_widget(Clear, rect);
    f.render_widget(Paragraph::new(lines), rect);
}

/// Quadratic-Bezier control point that bows an edge toward the center.
fn ctrl_point(a: (f64, f64), b: (f64, f64), center: (f64, f64)) -> (f64, f64) {
    let mid = ((a.0 + b.0) * 0.5, (a.1 + b.1) * 0.5);
    (
        mid.0 * 0.55 + center.0 * 0.45,
        mid.1 * 0.55 + center.1 * 0.45,
    )
}

/// Draw an operational edge as a bowed arc; flowing dots when `flowing`.
fn draw_edge(
    ctx: &mut ratatui::widgets::canvas::Context,
    a: (f64, f64),
    b: (f64, f64),
    center: (f64, f64),
    rgb: (u8, u8, u8),
    flowing: bool,
    elapsed: f32,
) {
    let c = ctrl_point(a, b, center);
    const N: usize = 28;
    let pts: Vec<(f64, f64)> = (0..=N)
        .map(|i| {
            let t = i as f64 / N as f64;
            let u = 1.0 - t;
            (
                u * u * a.0 + 2.0 * u * t * c.0 + t * t * b.0,
                u * u * a.1 + 2.0 * u * t * c.1 + t * t * b.1,
            )
        })
        .collect();

    if !flowing {
        let col = theme::rgb_scale(rgb, 0.22);
        for w in pts.windows(2) {
            ctx.draw(&CanvasLine {
                x1: w[0].0,
                y1: w[0].1,
                x2: w[1].0,
                y2: w[1].1,
                color: col,
            });
        }
        return;
    }

    // Base glow line plus bright dots flowing source -> target.
    let base = theme::rgb_scale(rgb, 0.45);
    for w in pts.windows(2) {
        ctx.draw(&CanvasLine {
            x1: w[0].0,
            y1: w[0].1,
            x2: w[1].0,
            y2: w[1].1,
            color: base,
        });
    }
    for (i, p) in pts.iter().enumerate() {
        let t = i as f32 / N as f32;
        let flow = ((t * 5.0 - elapsed * 2.5) * std::f32::consts::TAU).sin() * 0.5 + 0.5;
        if flow > 0.6 {
            ctx.draw(&Points {
                coords: &[*p],
                color: theme::rgb_scale(rgb, 0.7 + 0.3 * flow),
            });
        }
    }
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
