//! Tree state: the administrative (parent-link) hierarchy, its flattened
//! view, and vim-style navigation over it.

use crate::model::{OrgData, OrgNode};
use std::collections::{HashMap, HashSet};

pub struct TreeNode {
    pub data: OrgNode,
    pub children: Vec<String>,
    pub expanded: bool,
}

pub struct TreeState {
    pub nodes: HashMap<String, TreeNode>,
    pub root_id: String,
    /// Flattened, currently-visible node ids (respects expansion / search).
    pub flat_list: Vec<String>,
    pub focused_idx: usize,
    pub search_mode: bool,
    pub search_query: String,
}

impl TreeState {
    pub fn new(data: &OrgData) -> Self {
        let mut nodes = HashMap::new();
        for node in &data.nodes {
            nodes.insert(
                node.id.clone(),
                TreeNode {
                    data: node.clone(),
                    children: Vec::new(),
                    expanded: node.echelon <= 2, // auto-expand top echelons
                },
            );
        }
        // Assign children in source order (deterministic tree).
        for node in &data.nodes {
            if let Some(parent_id) = &node.parent {
                if let Some(parent) = nodes.get_mut(parent_id) {
                    parent.children.push(node.id.clone());
                }
            }
        }

        let root_id = data
            .nodes
            .iter()
            .find(|n| n.parent.is_none())
            .map(|n| n.id.clone())
            .expect("dataset has no root node");

        let mut state = TreeState {
            nodes,
            root_id,
            flat_list: Vec::new(),
            focused_idx: 0,
            search_mode: false,
            search_query: String::new(),
        };
        state.rebuild_flat_list();
        state
    }

    pub fn rebuild_flat_list(&mut self) {
        self.flat_list.clear();
        let root = self.root_id.clone();
        self.flatten(&root);
        self.clamp_focus();
    }

    fn flatten(&mut self, id: &str) {
        self.flat_list.push(id.to_string());
        let (expanded, children) = match self.nodes.get(id) {
            Some(n) => (n.expanded, n.children.clone()),
            None => return,
        };
        if expanded {
            for child in children.iter() {
                self.flatten(child);
            }
        }
    }

    fn clamp_focus(&mut self) {
        self.focused_idx = self
            .focused_idx
            .min(self.flat_list.len().saturating_sub(1));
    }

    pub fn focused_id(&self) -> Option<&String> {
        self.flat_list.get(self.focused_idx)
    }

    pub fn focused_node(&self) -> Option<&OrgNode> {
        self.focused_id()
            .and_then(|id| self.nodes.get(id))
            .map(|n| &n.data)
    }

    pub fn node(&self, id: &str) -> Option<&TreeNode> {
        self.nodes.get(id)
    }

    pub fn move_focus(&mut self, delta: isize) {
        let new = (self.focused_idx as isize + delta).max(0) as usize;
        self.focused_idx = new.min(self.flat_list.len().saturating_sub(1));
    }

    pub fn toggle_expanded(&mut self) {
        if let Some(id) = self.flat_list.get(self.focused_idx).cloned() {
            if let Some(node) = self.nodes.get_mut(&id) {
                node.expanded = !node.expanded;
            }
            self.rebuild_flat_list();
        }
    }

    /// Collapse if expanded, else jump to parent.
    pub fn collapse_or_parent(&mut self) {
        let id = match self.focused_id().cloned() {
            Some(id) => id,
            None => return,
        };
        let (expanded, parent) = match self.nodes.get(&id) {
            Some(n) => (n.expanded, n.data.parent.clone()),
            None => return,
        };
        if expanded {
            self.toggle_expanded();
        } else if let Some(parent_id) = parent {
            if let Some(pos) = self.flat_list.iter().position(|i| i == &parent_id) {
                self.focused_idx = pos;
            }
        }
    }

    /// Expand if collapsed, else jump to first child.
    pub fn expand_or_child(&mut self) {
        let id = match self.focused_id().cloned() {
            Some(id) => id,
            None => return,
        };
        let (expanded, first_child) = match self.nodes.get(&id) {
            Some(n) => (n.expanded, n.children.first().cloned()),
            None => return,
        };
        match (expanded, first_child) {
            (false, Some(_)) => self.toggle_expanded(),
            (true, Some(child)) => {
                if let Some(pos) = self.flat_list.iter().position(|i| i == &child) {
                    self.focused_idx = pos;
                }
            }
            _ => {}
        }
    }

    /// Tree depth (root = 0).
    pub fn depth_of(&self, id: &str) -> usize {
        let mut depth = 0;
        let mut cur = self.nodes.get(id).and_then(|n| n.data.parent.as_ref());
        while let Some(parent_id) = cur {
            depth += 1;
            cur = self.nodes.get(parent_id).and_then(|n| n.data.parent.as_ref());
        }
        depth
    }

    pub fn max_depth(&self) -> usize {
        self.nodes.keys().map(|id| self.depth_of(id)).max().unwrap_or(0)
    }

    /// All node ids in full DFS order, ignoring expansion (stable ordering
    /// for same-type hopping and search iteration).
    pub fn dfs_all(&self) -> Vec<String> {
        let mut out = Vec::new();
        let root = self.root_id.clone();
        self.dfs(&root, &mut out);
        out
    }

    fn dfs(&self, id: &str, out: &mut Vec<String>) {
        out.push(id.to_string());
        if let Some(n) = self.nodes.get(id) {
            for c in &n.children {
                self.dfs(c, out);
            }
        }
    }

    pub fn collapse_all(&mut self) {
        for n in self.nodes.values_mut() {
            n.expanded = false;
        }
        self.rebuild_flat_list();
        self.focused_idx = 0;
    }

    pub fn expand_all(&mut self) {
        for n in self.nodes.values_mut() {
            n.expanded = true;
        }
        self.rebuild_flat_list();
    }

    /// Expand all ancestors of `id` and move focus onto it.
    pub fn focus_on(&mut self, id: &str) {
        let chain = self.ancestry(id);
        for aid in &chain {
            if aid != id {
                if let Some(node) = self.nodes.get_mut(aid) {
                    node.expanded = true;
                }
            }
        }
        // If a search filter is active, clear it so the target is reachable.
        if !self.search_query.is_empty() {
            self.search_query.clear();
            self.search_mode = false;
        }
        self.rebuild_flat_list();
        if let Some(pos) = self.flat_list.iter().position(|x| x == id) {
            self.focused_idx = pos;
        }
    }

    /// Node ids from root down to the given node (inclusive).
    pub fn ancestry(&self, id: &str) -> Vec<String> {
        let mut chain = Vec::new();
        let mut cur = Some(id.to_string());
        while let Some(cid) = cur {
            match self.nodes.get(&cid) {
                Some(node) => {
                    chain.push(cid.clone());
                    cur = node.data.parent.clone();
                }
                None => break,
            }
        }
        chain.reverse();
        chain
    }

    /// Labels from root down to the given node.
    pub fn breadcrumb(&self, id: &str) -> Vec<String> {
        let mut chain = Vec::new();
        let mut cur = Some(id.to_string());
        while let Some(cid) = cur {
            if let Some(node) = self.nodes.get(&cid) {
                chain.push(node.data.label.clone());
                cur = node.data.parent.clone();
            } else {
                break;
            }
        }
        chain.reverse();
        chain
    }

    pub fn update_search(&mut self, query: &str) {
        self.search_query = query.to_string();
        let q = query.to_lowercase();
        if q.is_empty() {
            self.rebuild_flat_list();
            return;
        }
        let matches: HashSet<String> = self
            .nodes
            .values()
            .filter(|n| {
                n.data.label.to_lowercase().contains(&q)
                    || n.data.full_name.to_lowercase().contains(&q)
            })
            .map(|n| n.data.id.clone())
            .collect();

        let mut visible: HashSet<String> = HashSet::new();
        for m in matches.iter() {
            visible.insert(m.clone());
            let mut cur = self.nodes.get(m).and_then(|n| n.data.parent.clone());
            while let Some(pid) = cur {
                visible.insert(pid.clone());
                cur = self.nodes.get(&pid).and_then(|n| n.data.parent.clone());
            }
        }

        self.flat_list.clear();
        let root = self.root_id.clone();
        self.flatten_visible(&root, &visible);
        self.focused_idx = 0;
    }

    fn flatten_visible(&mut self, id: &str, visible: &HashSet<String>) {
        if !visible.contains(id) {
            return;
        }
        self.flat_list.push(id.to_string());
        let children = match self.nodes.get(id) {
            Some(n) => n.children.clone(),
            None => return,
        };
        for child in children.iter() {
            self.flatten_visible(child, visible);
        }
    }

    /// Is this id currently a search match? (for highlighting)
    pub fn is_match(&self, id: &str) -> bool {
        if self.search_query.is_empty() {
            return false;
        }
        let q = self.search_query.to_lowercase();
        self.nodes.get(id).map_or(false, |n| {
            n.data.label.to_lowercase().contains(&q)
                || n.data.full_name.to_lowercase().contains(&q)
        })
    }
}
