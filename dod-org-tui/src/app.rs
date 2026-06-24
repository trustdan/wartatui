//! Application state and input handling.

use crate::anim::{self, Clock};
use crate::model::{OrgData, OrgEdge, OrgMeta};
use crate::tree::TreeState;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Which chain of command we're viewing. (OPS lands in Phase 3.)
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Admin,
    Ops,
}

pub struct App {
    pub tree: TreeState,
    pub meta: OrgMeta,
    /// Operational edges — surfaced in Phase 3.
    #[allow(dead_code)]
    pub edges: Vec<OrgEdge>,
    pub mode: Mode,
    pub show_card: bool,
    pub clock: Clock,
    pub should_quit: bool,
}

impl App {
    pub fn new(data: OrgData) -> Self {
        let tree = TreeState::new(&data);
        App {
            tree,
            meta: data.meta,
            edges: data.edges,
            mode: Mode::Admin,
            show_card: false,
            clock: Clock::new(),
            should_quit: false,
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
        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => self.should_quit = true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true
            }
            KeyCode::Up | KeyCode::Char('k') => self.tree.move_focus(-1),
            KeyCode::Down | KeyCode::Char('j') => self.tree.move_focus(1),
            KeyCode::Left | KeyCode::Char('h') => self.tree.collapse_or_parent(),
            KeyCode::Right | KeyCode::Char('l') => self.tree.expand_or_child(),
            KeyCode::Char(' ') | KeyCode::Enter => self.tree.toggle_expanded(),
            KeyCode::Char('d') => self.show_card = !self.show_card,
            KeyCode::Char('/') => {
                self.tree.search_mode = true;
                self.tree.search_query.clear();
            }
            KeyCode::Esc => self.show_card = false,
            _ => {}
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
