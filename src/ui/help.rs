//! Help overlay — a centered popup showing all keybindings.
//! Dismisses on any keypress (except q / Ctrl+C which quit).

use ratatui::layout::Rect;
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

const DIM: Color = Color::Rgb(110, 118, 130);
const KEY: Color = Color::Rgb(232, 200, 90);
const HDR: Color = Color::Rgb(120, 200, 255);

pub fn render(f: &mut Frame, area: Rect) {
    let popup_w = area.width.min(64).max(48);
    let popup_h = area.height.min(30).max(10);
    let x = area.x + (area.width.saturating_sub(popup_w)) / 2;
    let y = area.y + (area.height.saturating_sub(popup_h)) / 2;
    let rect = Rect { x, y, width: popup_w, height: popup_h };

    f.render_widget(Clear, rect);

    let lines = build_lines();

    f.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .title(" ? HELP  (any key to close) ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(HDR)),
        ),
        rect,
    );
}

fn k(text: &'static str) -> Span<'static> {
    Span::styled(text, Style::default().fg(KEY).bold())
}
fn sep() -> Span<'static> {
    Span::styled("  ", Style::default())
}
fn desc(text: &'static str) -> Span<'static> {
    Span::styled(text, Style::default().fg(Color::Rgb(200, 205, 215)))
}
fn dim(text: &'static str) -> Span<'static> {
    Span::styled(text, Style::default().fg(DIM))
}
fn hdr(text: &'static str) -> Line<'static> {
    Line::from(Span::styled(text, Style::default().fg(HDR).bold()))
}
fn blank() -> Line<'static> {
    Line::from("")
}
fn row(key: &'static str, d: &'static str) -> Line<'static> {
    Line::from(vec![
        Span::raw("  "),
        k(key),
        sep(),
        desc(d),
    ])
}
fn row2(k1: &'static str, d1: &'static str, k2: &'static str, d2: &'static str) -> Line<'static> {
    Line::from(vec![
        Span::raw("  "),
        k(k1),
        sep(),
        desc(d1),
        Span::raw("   "),
        k(k2),
        sep(),
        desc(d2),
    ])
}

fn build_lines() -> Vec<Line<'static>> {
    vec![
        blank(),
        hdr(" NAVIGATE"),
        row2("j / k  ↑↓", "move focus",         "gg / Home", "top"),
        row2("h / l  ←→", "collapse / expand",   "G  / End ", "bottom"),
        row2("^D  /  D",   "half-page ↓",         "^U  /  U",  "half-page ↑"),
        row( "PageDn / PageUp", "full page"),
        blank(),
        hdr(" CROSS-CUTTING"),
        row2("[ / ]", "prev/next same type",  "{ / }", "prev/next sibling"),
        row( "m{a}",  "set mark at current node"),
        row( "'{a}  or  `{a}", "jump to mark  (use any letter a–z)"),
        blank(),
        hdr(" OPS / WORMHOLE"),
        row2("o",    "toggle ADMIN ⇄ OPS",  "gd",  "wormhole-jump (relation 1)"),
        row( "1–9",  "wormhole-jump to relation N"),
        blank(),
        hdr(" FOLDING & SEARCH"),
        row2("Space / ↵", "toggle fold",  "za",  "toggle (vim alias)"),
        row2("zM",  "collapse all",        "zR",  "expand all"),
        row2("/",   "search",              "n / N", "next / prev match"),
        blank(),
        hdr(" PANELS & MISC"),
        row2("d",    "unit-data card",    "Tab",    "cycle focus → card → links"),
        row2("↵",    "open link",         "Esc",    "close / cancel"),
        row( "? / i", "this help"),
        row2("q",    "quit",              "^C",     "quit"),
        blank(),
        Line::from(vec![
            Span::raw("  "),
            dim("--no-anim flag: disables all motion (SSH / battery)"),
        ]),
        blank(),
    ]
}
