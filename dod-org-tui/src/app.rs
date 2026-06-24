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
            KeyCode::Tab | KeyCode::BackTab => self.cycle_focus(),

            // Vertical movement (line / half-page / full-page)
            KeyCode::Up | KeyCode::Char('k') => self.move_vert(-1),
            KeyCode::Down | KeyCode::Char('j') => self.move_vert(1),
            KeyCode::Char('u') if ctrl => self.move_vert(-self.half_page()),
            KeyCode::Char('d') if ctrl => self.move_vert(self.half_page()),
            KeyCode::Char('b') if ctrl => self.move_vert(-self.full_page()),
            KeyCode::Char('f') if ctrl => self.move_vert(self.full_page()),
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

            KeyCode::Char(' ') | KeyCode::Enter => {
                if self.focus == Panel::Tree {
                    self.tree.toggle_expanded();
                    self.card_scroll = 0;
                }
            }
            KeyCode::Char('d') => self.toggle_card(),
            KeyCode::Char('/') => {
                self.tree.search_mode = true;
                self.tree.search_query.clear();
            }
            KeyCode::Esc => {
                self.show_card = false;
                self.focus = Panel::Tree;
            }
            _ => {}
        }
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
                self.card_scroll = 0;
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
                self.card_scroll = 0;
            }
            Panel::Card => self.card_scroll = 0,
        }
    }

    fn goto_bottom(&mut self) {
        match self.focus {
            Panel::Tree => {
                self.tree.focused_idx = self.tree.flat_list.len().saturating_sub(1);
                self.card_scroll = 0;
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
            self.card_scroll = 0;
        } else {
            self.focus = Panel::Tree;
        }
    }

    fn cycle_focus(&mut self) {
        match self.focus {
            Panel::Tree => {
                if !self.show_card {
                    self.show_card = true;
                    self.card_scroll = 0;
                }
                self.focus = Panel::Card;
            }
            Panel::Card => self.focus = Panel::Tree,
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
