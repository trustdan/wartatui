//! Application state and input handling.

use crate::anim::{self, Clock};
use crate::layout_radial::{self, Positions};
use crate::model::{OrgData, OrgEdge, OrgMeta};
use crate::tree::TreeState;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::cell::Cell;
use std::collections::HashMap;

/// A pending multi-key prefix awaiting its second key.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Pending {
    None,
    G,        // gg / gd
    Z,        // za / zM / zR
    SetMark,  // m{a-z}
    JumpMark, // '{a-z}
}

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

/// An operational edge as seen from the focused node.
pub struct Relation {
    pub other: String,
    pub relation: String,
    pub outgoing: bool,
}

/// An in-flight wormhole jump: (from id, to id, start time).
pub type Transition = (String, String, f32);

/// Wormhole transition duration in seconds.
pub const JUMP_SECS: f32 = 1.0;

pub struct App {
    pub tree: TreeState,
    pub meta: OrgMeta,
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
    /// Active wormhole jump animation, if any.
    pub transition: Option<Transition>,
    /// When true, all motion and animation is suppressed (--no-anim).
    pub no_anim: bool,
    /// When true, the help overlay is shown.
    pub show_help: bool,
    /// Brief confirmation after `m{x}`: (letter, clock-time-set). Shown for 1.5 s.
    pub mark_flash: Option<(char, f32)>,

    // Viewport sizes, written by the renderer each frame so paging matches
    // what's actually on screen.
    pub tree_viewport: Cell<u16>,
    pub card_viewport: Cell<u16>,
    pub card_lines: Cell<u16>,

    /// Pending multi-key prefix (g / z / m / ').
    pending: Pending,
    /// Named marks (m{a-z} → node id).
    marks: HashMap<char, String>,
}

impl App {
    pub fn new(data: OrgData, no_anim: bool) -> Self {
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
            transition: None,
            no_anim,
            show_help: false,
            mark_flash: None,
            tree_viewport: Cell::new(10),
            card_viewport: Cell::new(10),
            card_lines: Cell::new(0),
            pending: Pending::None,
            marks: HashMap::new(),
        }
    }

    /// Boot cascade progress 0..1, always 1.0 when animations are off.
    pub fn boot_progress(&self) -> f32 {
        if self.no_anim {
            1.0
        } else {
            anim::ease_out_cubic(self.clock.elapsed() / anim::BOOT_SECS)
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        // Help overlay intercepts all keys — only quit passes through.
        if self.show_help {
            let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
            match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => self.should_quit = true,
                KeyCode::Char('c') if ctrl => self.should_quit = true,
                _ => self.show_help = false,
            }
            return;
        }

        if self.tree.search_mode {
            self.handle_search_key(key);
            return;
        }

        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

        // Resolve a pending multi-key prefix; consume the key if it completes one.
        if self.pending != Pending::None {
            let pending = self.pending;
            self.pending = Pending::None;
            if self.resolve_pending(pending, key) {
                return;
            }
            // Not consumed — fall through so the key still acts normally.
        }

        // Arm a new prefix.
        if !ctrl {
            match key.code {
                KeyCode::Char('g') => {
                    self.pending = Pending::G;
                    return;
                }
                KeyCode::Char('z') => {
                    self.pending = Pending::Z;
                    return;
                }
                KeyCode::Char('m') => {
                    self.pending = Pending::SetMark;
                    return;
                }
                KeyCode::Char('\'') | KeyCode::Char('`') => {
                    self.pending = Pending::JumpMark;
                    return;
                }
                _ => {}
            }
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => self.should_quit = true,
            KeyCode::Char('c') if ctrl => self.should_quit = true,

            // Open help overlay.
            KeyCode::Char('?') | KeyCode::Char('i') => self.show_help = true,

            // Panel focus
            KeyCode::Tab => self.cycle_focus(),
            KeyCode::BackTab => {
                self.focus = Panel::Tree;
                self.link_focus = None;
            }

            // Vertical movement (line / half-page / full-page)
            KeyCode::Up | KeyCode::Char('k') => self.move_vert(-1),
            KeyCode::Down | KeyCode::Char('j') => self.move_vert(1),
            // Ctrl+D/U (vim standard) and Shift+D/U (both work).
            KeyCode::Char('u') if ctrl => self.move_vert(-self.half_page()),
            KeyCode::Char('d') if ctrl => self.move_vert(self.half_page()),
            KeyCode::Char('U') => self.move_vert(-self.half_page()),
            KeyCode::Char('D') => self.move_vert(self.half_page()),
            KeyCode::PageUp => self.move_vert(-self.full_page()),
            KeyCode::PageDown => self.move_vert(self.full_page()),
            KeyCode::Char('G') | KeyCode::End => self.goto_bottom(),
            KeyCode::Home => self.goto_top(),

            // Cross-cutting navigation
            KeyCode::Char('[') => self.same_type(-1),
            KeyCode::Char(']') => self.same_type(1),
            KeyCode::Char('{') => self.sibling(-1),
            KeyCode::Char('}') => self.sibling(1),
            KeyCode::Char('n') => self.search_next(1),
            KeyCode::Char('N') => self.search_next(-1),

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
            KeyCode::Char('o') | KeyCode::Char('O') => self.toggle_mode(),
            KeyCode::Char(c @ '1'..='9') => {
                self.jump_to_relation((c as u8 - b'1') as usize)
            }
            KeyCode::Char('/') => {
                self.tree.search_mode = true;
                self.tree.search_query.clear();
            }
            KeyCode::Esc => {
                self.show_help = false;
                self.show_card = false;
                self.focus = Panel::Tree;
                self.link_focus = None;
            }
            _ => {}
        }
    }

    /// Handle the second key of a multi-key prefix. Returns true if consumed.
    fn resolve_pending(&mut self, pending: Pending, key: KeyEvent) -> bool {
        match pending {
            Pending::G => match key.code {
                KeyCode::Char('g') => {
                    self.goto_top();
                    true
                }
                KeyCode::Char('d') => {
                    self.jump_to_relation(0);
                    true
                }
                _ => false,
            },
            Pending::Z => match key.code {
                KeyCode::Char('a') => {
                    self.tree.toggle_expanded();
                    self.reset_card_view();
                    true
                }
                KeyCode::Char('M') => {
                    self.tree.collapse_all();
                    self.reset_card_view();
                    true
                }
                KeyCode::Char('R') => {
                    self.tree.expand_all();
                    true
                }
                _ => false,
            },
            Pending::SetMark => {
                if let KeyCode::Char(c) = key.code {
                    if c.is_ascii_alphabetic() {
                        let letter = c.to_ascii_lowercase();
                        if let Some(id) = self.tree.focused_id().cloned() {
                            self.marks.insert(letter, id);
                            self.mark_flash = Some((letter, self.clock.elapsed()));
                        }
                        return true;
                    }
                }
                false
            }
            Pending::JumpMark => {
                if let KeyCode::Char(c) = key.code {
                    if c.is_ascii_alphabetic() {
                        if let Some(id) = self.marks.get(&c.to_ascii_lowercase()).cloned() {
                            self.tree.focus_on(&id);
                            self.reset_card_view();
                        }
                        return true;
                    }
                }
                false
            }
            Pending::None => false,
        }
    }

    /// Hop to the previous/next node of the same type (cross-cutting).
    fn same_type(&mut self, dir: isize) {
        let fid = match self.tree.focused_id().cloned() {
            Some(id) => id,
            None => return,
        };
        let ftype = match self.tree.node(&fid) {
            Some(n) => n.data.org_type.clone(),
            None => return,
        };
        let same: Vec<String> = self
            .tree
            .dfs_all()
            .into_iter()
            .filter(|id| {
                self.tree
                    .node(id)
                    .map_or(false, |n| n.data.org_type == ftype)
            })
            .collect();
        if same.len() <= 1 {
            return;
        }
        if let Some(pos) = same.iter().position(|x| x == &fid) {
            let n = same.len() as isize;
            let ni = (pos as isize + dir).rem_euclid(n) as usize;
            let target = same[ni].clone();
            self.tree.focus_on(&target);
            self.reset_card_view();
        }
    }

    /// Move to the previous/next sibling under the same parent.
    fn sibling(&mut self, dir: isize) {
        let fid = match self.tree.focused_id().cloned() {
            Some(id) => id,
            None => return,
        };
        let parent = self.tree.node(&fid).and_then(|n| n.data.parent.clone());
        let pid = match parent {
            Some(p) => p,
            None => return,
        };
        let sibs = match self.tree.node(&pid) {
            Some(n) => n.children.clone(),
            None => return,
        };
        if let Some(pos) = sibs.iter().position(|x| x == &fid) {
            let n = sibs.len() as isize;
            let ni = (pos as isize + dir).rem_euclid(n) as usize;
            let target = sibs[ni].clone();
            self.tree.focus_on(&target);
            self.reset_card_view();
        }
    }

    /// Jump to the next/previous visible search match.
    fn search_next(&mut self, dir: isize) {
        if self.tree.search_query.is_empty() {
            return;
        }
        let n = self.tree.flat_list.len();
        if n == 0 {
            return;
        }
        let start = self.tree.focused_idx as isize;
        for step in 1..=n as isize {
            let idx = (start + dir * step).rem_euclid(n as isize) as usize;
            let id = self.tree.flat_list[idx].clone();
            if self.tree.is_match(&id) {
                self.tree.focused_idx = idx;
                self.reset_card_view();
                break;
            }
        }
    }

    fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            Mode::Admin => Mode::Ops,
            Mode::Ops => Mode::Admin,
        };
        // The relations rail takes the right panel in OPS; reset link focus.
        self.focus = Panel::Tree;
        self.link_focus = None;
    }

    /// Operational edges touching the focused node (in edge order).
    pub fn relations(&self) -> Vec<Relation> {
        let fid = match self.tree.focused_id() {
            Some(id) => id.clone(),
            None => return Vec::new(),
        };
        let mut out = Vec::new();
        for e in &self.edges {
            if e.source == fid {
                out.push(Relation {
                    other: e.target.clone(),
                    relation: e.relation.clone(),
                    outgoing: true,
                });
            } else if e.target == fid {
                out.push(Relation {
                    other: e.source.clone(),
                    relation: e.relation.clone(),
                    outgoing: false,
                });
            }
        }
        out
    }

    /// Wormhole-jump to the Nth operational relation of the focused node.
    fn jump_to_relation(&mut self, index: usize) {
        let target = match self.relations().get(index) {
            Some(r) => r.other.clone(),
            None => return,
        };
        let from = self.tree.focused_id().cloned();
        self.tree.focus_on(&target);
        self.reset_card_view();
        if let Some(f) = from {
            self.transition = Some((f, target, self.clock.elapsed()));
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
