//! Application state and input handling.

use crate::anim::{self, Clock};
use crate::layout_radial::{self, Positions};
use crate::model::{OrgData, OrgEdge, OrgMeta};
use crate::tree::TreeState;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::cell::Cell;

/// Which chain of command we're viewing. (OPS lands in Phase 3.)
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Admin,
    Ops,
}

/// Which panel keyboard movement currently drives.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Tree,
    Card,
}

pub struct App {
    pub tree: TreeState,
    pub meta: OrgMeta,
    /// Operational edges — surfaced in Phase 3.
    #[allow(dead_code)]
    pub edges: Vec<OrgEdge>,
    pub mode: Mode,
    pub show_card: bool,
    pub focus: Panel,
    pub card_scroll: u16,
    /// When the card holds focus, which hyperlink (if any) is selected.
    pub link_focus: Option<usize>,
    pub clock: Clock,
    pub should_quit: bool,
    /// Radial positions for the constellation (computed once).
    pub positions: Positions,

    // Viewport sizes, written by the renderer each frame so paging matches
    // what's actually on screen.
    pub tree_viewport: Cell<u16>,
    pub card_viewport: Cell<u16>,
    pub card_lines: Cell<u16>,

    /// Pending `g` for the `gg` sequence.
    pending_g: bool,
}

impl App {
    pub fn new(data: OrgData) -> Self {
        let tree = TreeState::new(&data);
        let positions = layout_radial::compute(&tree);
        App {
            tree,
            meta: data.meta,
            edges: data.edges,
            mode: Mode::Admin,
            show_card: false,
            focus: Panel::Tree,
            card_scroll: 0,
            link_focus: None,
            clock: Clock::new(),
            should_quit: false,
            positions,
            tree_viewport: Cell::new(10),
            card_viewport: Cell::new(10),
            card_lines: Cell::new(0),
            pending_g: false,
        }
    }

    /// Boot cascade progress 0..1 (eased).
    pub fn boot_progress(&self) -> f32 {
        anim::ease_out_cubic(self.clock.elapsed() / anim::BOOT_SECS)
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if self.tree.search_mode {
            self.handle_search_key(key);
            return;
        }

        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        // `gg` sequence: a bare 'g' arms it; the next 'g' jumps to top.
        if !ctrl && key.code == KeyCode::Char('g') {
            if self.pending_g {
                self.goto_top();
                self.pending_g = false;
            } else {
                self.pending_g = true;
            }
            return;
        }
        self.pending_g = false;

        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => self.should_quit = true,
            KeyCode::Char('c') if ctrl => self.should_quit = true,

            // Panel focus
            KeyCode::Tab => self.cycle_focus(),
            KeyCode::BackTab => {
                self.focus = Panel::Tree;
                self.link_focus = None;
            }

            // Vertical movement (line / half-page / full-page)
            KeyCode::Up | KeyCode::Char('k') => self.move_vert(-1),
            KeyCode::Down | KeyCode::Char('j') => self.move_vert(1),
            KeyCode::Char('U') => self.move_vert(-self.half_page()),
            KeyCode::Char('D') => self.move_vert(self.half_page()),
            KeyCode::PageUp => self.move_vert(-self.full_page()),
            KeyCode::PageDown => self.move_vert(self.full_page()),
            KeyCode::Char('G') | KeyCode::End => self.goto_bottom(),
            KeyCode::Home => self.goto_top(),

            // Tree-only horizontal: collapse/expand, or leave the card.
            KeyCode::Left | KeyCode::Char('h') => match self.focus {
                Panel::Card => self.focus = Panel::Tree,
                Panel::Tree => self.tree.collapse_or_parent(),
            },
            KeyCode::Right | KeyCode::Char('l') => {
                if self.focus == Panel::Tree {
                    self.tree.expand_or_child();
                }
            }

            KeyCode::Char(' ') => {
                if self.focus == Panel::Tree {
                    self.tree.toggle_expanded();
                    self.reset_card_view();
                }
            }
            KeyCode::Enter => match self.focus {
                Panel::Tree => {
                    self.tree.toggle_expanded();
                    self.reset_card_view();
                }
                Panel::Card => self.open_focused_link(),
            },
            KeyCode::Char('d') => self.toggle_card(),
            KeyCode::Char('/') => {
                self.tree.search_mode = true;
                self.tree.search_query.clear();
            }
            KeyCode::Esc => {
                self.show_card = false;
                self.focus = Panel::Tree;
                self.link_focus = None;
            }
            _ => {}
        }
    }

    /// URLs available on the focused unit (source page + any links in notes).
    pub fn current_links(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        if let Some(node) = self.tree.focused_node() {
            if let Some(u) = node.meta_str("sourceUrl") {
                if u.starts_with("http") {
                    out.push(u.to_string());
                }
            }
            if let Some(notes) = node.meta_str("notes") {
                for tok in notes.split_whitespace() {
                    let t = tok.trim_matches(|c: char| {
                        !(c.is_ascii_alphanumeric()
                            || matches!(c, ':' | '/' | '.' | '-' | '_' | '?' | '=' | '&' | '#' | '%' | '+' | '~'))
                    });
                    if t.starts_with("http") && !out.iter().any(|e| e == t) {
                        out.push(t.to_string());
                    }
                }
            }
        }
        out
    }

    fn open_focused_link(&self) {
        if let Some(i) = self.link_focus {
            if let Some(url) = self.current_links().get(i) {
                open_url(url);
            }
        }
    }

    fn reset_card_view(&mut self) {
        self.card_scroll = 0;
        self.link_focus = None;
    }

    fn half_page(&self) -> isize {
        let vp = match self.focus {
            Panel::Tree => self.tree_viewport.get(),
            Panel::Card => self.card_viewport.get(),
        };
        ((vp / 2) as isize).max(1)
    }

    fn full_page(&self) -> isize {
        let vp = match self.focus {
            Panel::Tree => self.tree_viewport.get(),
            Panel::Card => self.card_viewport.get(),
        };
        ((vp.saturating_sub(1)) as isize).max(1)
    }

    fn move_vert(&mut self, delta: isize) {
        match self.focus {
            Panel::Tree => {
                self.tree.move_focus(delta);
                self.reset_card_view();
            }
            Panel::Card => self.scroll_card(delta),
        }
    }

    fn scroll_card(&mut self, delta: isize) {
        let max = self
            .card_lines
            .get()
            .saturating_sub(self.card_viewport.get());
        let next = (self.card_scroll as isize + delta).clamp(0, max as isize);
        self.card_scroll = next as u16;
    }

    fn goto_top(&mut self) {
        match self.focus {
            Panel::Tree => {
                self.tree.focused_idx = 0;
                self.reset_card_view();
            }
            Panel::Card => self.card_scroll = 0,
        }
    }

    fn goto_bottom(&mut self) {
        match self.focus {
            Panel::Tree => {
                self.tree.focused_idx = self.tree.flat_list.len().saturating_sub(1);
                self.reset_card_view();
            }
            Panel::Card => {
                let max = self
                    .card_lines
                    .get()
                    .saturating_sub(self.card_viewport.get());
                self.card_scroll = max;
            }
        }
    }

    fn toggle_card(&mut self) {
        self.show_card = !self.show_card;
        if self.show_card {
            self.reset_card_view();
        } else {
            self.focus = Panel::Tree;
            self.link_focus = None;
        }
    }

    /// Tab: Tree → Card → cycle each link → back to Tree.
    fn cycle_focus(&mut self) {
        match self.focus {
            Panel::Tree => {
                if !self.show_card {
                    self.show_card = true;
                    self.card_scroll = 0;
                }
                self.focus = Panel::Card;
                self.link_focus = None;
            }
            Panel::Card => {
                let n_links = self.current_links().len();
                match self.link_focus {
                    None if n_links > 0 => self.link_focus = Some(0),
                    Some(i) if i + 1 < n_links => self.link_focus = Some(i + 1),
                    _ => {
                        self.focus = Panel::Tree;
                        self.link_focus = None;
                    }
                }
            }
        }
    }

    fn handle_search_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.tree.search_mode = false;
                self.tree.search_query.clear();
                self.tree.rebuild_flat_list();
            }
            KeyCode::Enter => self.tree.search_mode = false,
            KeyCode::Char(c) => {
                let mut q = self.tree.search_query.clone();
                q.push(c);
                self.tree.update_search(&q);
            }
            KeyCode::Backspace => {
                let mut q = self.tree.search_query.clone();
                q.pop();
                self.tree.update_search(&q);
            }
            _ => {}
        }
    }
}

/// Open a URL in the user's default browser (best-effort, non-blocking).
fn open_url(url: &str) {
    use std::process::Command;
    let _ = {
        #[cfg(target_os = "windows")]
        {
            Command::new("cmd").args(["/C", "start", "", url]).spawn()
        }
        #[cfg(target_os = "macos")]
        {
            Command::new("open").arg(url).spawn()
        }
        #[cfg(all(unix, not(target_os = "macos")))]
        {
            Command::new("xdg-open").arg(url).spawn()
        }
    };
}
