//! Radial-tree layout for the constellation: DoD at the center, each depth a
//! ring outward, children spread across an angular wedge sized by how many
//! leaves they carry. Deterministic — computed once at startup.

use crate::tree::TreeState;
use std::collections::HashMap;
use std::f32::consts::TAU;

pub struct Positions {
    /// node id -> (normalized radius 0..1, angle in radians)
    pub polar: HashMap<String, (f32, f32)>,
    pub max_depth: usize,
}

impl Positions {
    pub fn get(&self, id: &str) -> Option<(f32, f32)> {
        self.polar.get(id).copied()
    }
}

pub fn compute(tree: &TreeState) -> Positions {
    let mut leaves: HashMap<String, usize> = HashMap::new();
    count_leaves(tree, &tree.root_id, &mut leaves);

    let max_depth = tree.max_depth().max(1);
    let mut polar = HashMap::new();
    assign(
        tree,
        &tree.root_id,
        0.0,
        TAU,
        0,
        max_depth,
        &leaves,
        &mut polar,
    );
    Positions { polar, max_depth }
}

/// Number of leaf descendants (a leaf counts as 1).
fn count_leaves(tree: &TreeState, id: &str, out: &mut HashMap<String, usize>) -> usize {
    let children = match tree.node(id) {
        Some(n) => n.children.clone(),
        None => return 0,
    };
    let count = if children.is_empty() {
        1
    } else {
        children.iter().map(|c| count_leaves(tree, c, out)).sum()
    };
    out.insert(id.to_string(), count);
    count
}

#[allow(clippy::too_many_arguments)]
fn assign(
    tree: &TreeState,
    id: &str,
    a0: f32,
    a1: f32,
    depth: usize,
    max_depth: usize,
    leaves: &HashMap<String, usize>,
    out: &mut HashMap<String, (f32, f32)>,
) {
    let radius = depth as f32 / max_depth as f32;
    out.insert(id.to_string(), (radius, (a0 + a1) * 0.5));

    let children = match tree.node(id) {
        Some(n) => n.children.clone(),
        None => return,
    };
    if children.is_empty() {
        return;
    }
    let total: usize = children
        .iter()
        .map(|c| leaves.get(c).copied().unwrap_or(1))
        .sum::<usize>()
        .max(1);

    let mut cursor = a0;
    let span = a1 - a0;
    for child in &children {
        let weight = leaves.get(child).copied().unwrap_or(1) as f32;
        let child_span = span * weight / total as f32;
        assign(
            tree,
            child,
            cursor,
            cursor + child_span,
            depth + 1,
            max_depth,
            leaves,
            out,
        );
        cursor += child_span;
    }
}
