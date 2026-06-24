use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io;

// =============================================================================
// Data model (mirrors dod-org.json)
// =============================================================================

#[derive(Deserialize, Serialize, Clone, Debug)]
struct OrgNode {
    id: String,
    label: String,
    #[serde(rename = "fullName")]
    full_name: String,
    echelon: u8,
    #[serde(rename = "type")]
    org_type: String,
    parent: Option<String>,
    source: Option<String>,
    meta: std::collections::BTreeMap<String, serde_json::Value>,
}

#[derive(Deserialize)]
struct OrgEdge {
    source: String,
    target: String,
    relation: String,
}

#[derive(Deserialize, Clone)]
struct OrgMeta {
    title: String,
    #[serde(rename = "secondaryTitle")]
    secondary_title: Option<String>,
    #[serde(rename = "asOf", default)]
    #[allow(dead_code)]
    as_of: String,
}

#[derive(Deserialize)]
struct OrgData {
    nodes: Vec<OrgNode>,
    edges: Vec<OrgEdge>,
    meta: OrgMeta,
}

// =============================================================================
// Color mapping (mirrors the 3D version)
// =============================================================================

fn color_for_type(t: &str) -> Color {
    match t {
        "department" => Color::Rgb(227, 178, 60),     // gold
        "principal" => Color::Rgb(227, 178, 60),      // gold
        "osd" => Color::Rgb(91, 138, 192),            // blue
        "joint" => Color::Rgb(155, 127, 224),         // purple
        "mildep" => Color::Rgb(111, 167, 127),        // green
        "service" => Color::Rgb(169, 199, 160),       // light green
        "cocom-geo" => Color::Rgb(217, 138, 79),      // orange
        "cocom-func" => Color::Rgb(194, 94, 74),      // rust
        "agency" => Color::Rgb(133, 147, 166),        // slate
        _ => Color::Gray,
    }
}

fn label_for_type(t: &str) -> &'static str {
    match t {
        "department" => "Department (apex)",
        "principal" => "Secretary / Deputy",
        "osd" => "OSD staff",
        "joint" => "Joint (CJCS / Joint Staff)",
        "mildep" => "Military Department",
        "service" => "Armed Service",
        "cocom-geo" => "COCOM — geographic",
        "cocom-func" => "COCOM — functional",
        "agency" => "Defense agency",
        _ => "Unknown",
    }
}

// =============================================================================
// Tree state
// =============================================================================

struct TreeNode {
    data: OrgNode,
    children: Vec<String>, // ids of direct children
    expanded: bool,
    index: usize, // for flat traversal
}

struct TreeState {
    nodes: HashMap<String, TreeNode>,
    root_id: String,
    flat_list: Vec<String>, // flattened tree (respects expanded state)
    focused_idx: usize,
    search_mode: bool,
    search_query: String,
}

impl TreeState {
    fn new(org_data: &OrgData) -> Self {
        let mut nodes = HashMap::new();

        // Index nodes
        for node in &org_data.nodes {
            nodes.insert(
                node.id.clone(),
                TreeNode {
                    data: node.clone(),
                    children: Vec::new(),
                    expanded: node.echelon <= 2, // auto-expand top echelons
                    index: 0,
                },
            );
        }

        // Assign children (in source order, so the tree is deterministic)
        for node in &org_data.nodes {
            if let Some(parent_id) = &node.parent {
                if let Some(parent) = nodes.get_mut(parent_id) {
                    parent.children.push(node.id.clone());
                }
            }
        }

        let root_id = org_data
            .nodes
            .iter()
            .find(|n| n.parent.is_none())
            .map(|n| n.id.clone())
            .expect("No root found");

        let mut state = TreeState {
            nodes,
            root_id: root_id.clone(),
            flat_list: Vec::new(),
            focused_idx: 0,
            search_mode: false,
            search_query: String::new(),
        };

        state.rebuild_flat_list();
        state
    }

    fn rebuild_flat_list(&mut self) {
        self.flat_list.clear();
        let root = self.root_id.clone();
        self.flatten_node(&root, 0);
    }

    fn flatten_node(&mut self, id: &str, depth: usize) {
        self.flat_list.push(id.to_string());

        let (expanded, children) = match self.nodes.get(id) {
            Some(node) => (node.expanded, node.children.clone()),
            None => return,
        };
        if expanded {
            for child_id in children.iter() {
                self.flatten_node(child_id, depth + 1);
            }
        }
    }

    fn toggle_expanded(&mut self) {
        if let Some(id) = self.flat_list.get(self.focused_idx) {
            if let Some(node) = self.nodes.get_mut(id) {
                node.expanded = !node.expanded;
            }
            self.rebuild_flat_list();
        }
    }

    fn move_focus(&mut self, delta: isize) {
        let new_idx = (self.focused_idx as isize + delta).max(0) as usize;
        self.focused_idx = new_idx.min(self.flat_list.len().saturating_sub(1));
    }

    fn focused_node(&self) -> Option<&OrgNode> {
        self.flat_list
            .get(self.focused_idx)
            .and_then(|id| self.nodes.get(id))
            .map(|tn| &tn.data)
    }

    fn depth_of(&self, id: &str) -> usize {
        let mut depth = 0;
        let mut cur = self.nodes.get(id).and_then(|n| n.data.parent.as_ref());
        while let Some(parent_id) = cur {
            depth += 1;
            cur = self.nodes.get(parent_id).and_then(|n| n.data.parent.as_ref());
        }
        depth
    }

    fn update_search(&mut self, query: &str) {
        self.search_query = query.to_lowercase();
        if self.search_query.is_empty() {
            // Clear search: restore normal view
            self.rebuild_flat_list();
        } else {
            // Filter flat list to matches + their ancestors
            let matches: std::collections::HashSet<String> = self
                .nodes
                .values()
                .filter(|tn| {
                    tn.data.label.to_lowercase().contains(&self.search_query)
                        || tn.data.full_name.to_lowercase().contains(&self.search_query)
                })
                .map(|tn| tn.data.id.clone())
                .collect();

            let mut visible = std::collections::HashSet::new();
            for m in matches.iter() {
                visible.insert(m.clone());
                // Include all ancestors
                let mut cur = self.nodes.get(m).and_then(|n| n.data.parent.as_ref());
                while let Some(parent_id) = cur {
                    visible.insert(parent_id.clone());
                    cur = self.nodes.get(parent_id).and_then(|n| n.data.parent.as_ref());
                }
            }

            self.flat_list.clear();
            let root = self.root_id.clone();
            self.rebuild_list_with_visible(&root, &visible);
            self.focused_idx = 0;
        }
    }

    fn rebuild_list_with_visible(&mut self, id: &str, visible: &std::collections::HashSet<String>) {
        if !visible.contains(id) {
            return;
        }
        self.flat_list.push(id.to_string());

        let children = match self.nodes.get(id) {
            Some(node) => node.children.clone(),
            None => return,
        };
        for child_id in children.iter() {
            self.rebuild_list_with_visible(child_id, visible);
        }
    }
}

// =============================================================================
// App state
// =============================================================================

struct App {
    tree: TreeState,
    show_card: bool,
    meta: OrgMeta,
}

impl App {
    fn new(org_data: OrgData) -> Self {
        let meta = org_data.meta.clone();
        App {
            tree: TreeState::new(&org_data),
            show_card: false,
            meta,
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if self.tree.search_mode {
            match key.code {
                KeyCode::Esc => {
                    self.tree.search_mode = false;
                    self.tree.search_query.clear();
                    self.tree.rebuild_flat_list();
                }
                KeyCode::Enter => {
                    self.tree.search_mode = false;
                }
                KeyCode::Char(c) => {
                    self.tree.search_query.push(c);
                    self.tree.update_search(&self.tree.search_query.clone());
                }
                KeyCode::Backspace => {
                    self.tree.search_query.pop();
                    self.tree.update_search(&self.tree.search_query.clone());
                }
                _ => {}
            }
        } else {
            match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => std::process::exit(0),
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    std::process::exit(0)
                }
                KeyCode::Up | KeyCode::Char('k') => self.tree.move_focus(-1),
                KeyCode::Down | KeyCode::Char('j') => self.tree.move_focus(1),
                KeyCode::Left | KeyCode::Char('h') => {
                    if let Some(id) = self.tree.flat_list.get(self.tree.focused_idx) {
                        if let Some(node) = self.tree.nodes.get(id) {
                            if node.expanded {
                                self.tree.toggle_expanded();
                            } else if let Some(parent_id) = &node.data.parent {
                                // Jump to parent
                                if let Some(pos) = self
                                    .tree
                                    .flat_list
                                    .iter()
                                    .position(|id| id == parent_id)
                                {
                                    self.tree.focused_idx = pos;
                                }
                            }
                        }
                    }
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    let info = self
                        .tree
                        .flat_list
                        .get(self.tree.focused_idx)
                        .and_then(|id| self.tree.nodes.get(id))
                        .map(|node| (node.expanded, node.children.first().cloned()));
                    if let Some((expanded, first_child)) = info {
                        if !expanded && first_child.is_some() {
                            self.tree.toggle_expanded();
                        } else if expanded {
                            // Jump to first child
                            if let Some(child_id) = first_child {
                                if let Some(pos) =
                                    self.tree.flat_list.iter().position(|id| id == &child_id)
                                {
                                    self.tree.focused_idx = pos;
                                }
                            }
                        }
                    }
                }
                KeyCode::Char(' ') | KeyCode::Enter => self.tree.toggle_expanded(),
                KeyCode::Char('d') => self.show_card = !self.show_card,
                KeyCode::Char('/') => self.tree.search_mode = true,
                KeyCode::Esc => self.show_card = false,
                _ => {}
            }
        }
    }
}

// =============================================================================
// UI rendering
// =============================================================================

fn ui(f: &mut Frame, app: &App) {
    let size = f.size();

    // Top banner
    let banner_height = 3;
    let banner = Layout::default()
        .constraints([Constraint::Length(banner_height), Constraint::Min(0)])
        .split(size);

    let title = format!(
        "{} {}",
        app.meta.title,
        app.meta
            .secondary_title
            .as_ref()
            .map(|s| format!("({})", s))
            .unwrap_or_default()
    );
    let banner_text = Line::from(vec![
        Span::styled("UNCLASSIFIED", Style::default().fg(Color::Green)),
        Span::raw(" — "),
        Span::styled(&title, Style::default().bold().fg(Color::Yellow)),
    ]);
    f.render_widget(
        Paragraph::new(banner_text)
            .alignment(Alignment::Center)
            .style(Style::default().bg(Color::Black)),
        banner[0],
    );

    // Main content
    let main = Layout::default()
        .constraints([
            Constraint::Min(1),
            Constraint::Length(if app.tree.search_mode { 3 } else { 0 }),
        ])
        .direction(Direction::Vertical)
        .split(banner[1]);

    // Tree and card layout
    let (tree_area, card_area) = if app.show_card {
        let cols = Layout::default()
            .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
            .direction(Direction::Horizontal)
            .split(main[0]);
        (cols[0], Some(cols[1]))
    } else {
        (main[0], None)
    };

    // ---- Tree view ----
    render_tree(f, app, tree_area);

    // ---- Unit data card ----
    if let Some(area) = card_area {
        render_card(f, app, area);
    }

    // ---- Search bar ----
    if app.tree.search_mode {
        let search_block = Block::default()
            .title("Search (Esc to exit)")
            .borders(Borders::ALL);
        let search_text = format!("> {}", app.tree.search_query);
        f.render_widget(
            Paragraph::new(search_text)
                .block(search_block)
                .style(Style::default().fg(Color::Yellow)),
            main[1],
        );
    }

    // ---- Footer help ----
    let help = "↑↓ navigate  Space/Enter expand  / search  d data  q quit";
    let help_para = Paragraph::new(Span::raw(help))
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    let footer = Layout::default()
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .direction(Direction::Vertical)
        .split(size);
    f.render_widget(help_para, footer[1]);
}

fn render_tree(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(format!(" {} ", app.meta.title))
        .borders(Borders::ALL);

    let mut lines = vec![];
    for (i, node_id) in app.tree.flat_list.iter().enumerate() {
        if let Some(node) = app.tree.nodes.get(node_id) {
            let depth = app.tree.depth_of(node_id);
            let indent = " ".repeat(depth * 2);

            let has_children = !node.children.is_empty();
            let expand_char = if has_children {
                if node.expanded { "▼ " } else { "▶ " }
            } else {
                "  "
            };

            let is_focused = i == app.tree.focused_idx;
            let color = color_for_type(&node.data.org_type);
            let style = if is_focused {
                Style::default().bg(Color::DarkGray).fg(color).bold()
            } else {
                Style::default().fg(color)
            };

            let label = format!(
                "{}{}{} {}",
                indent,
                expand_char,
                node.data.label,
                if is_focused { " ◄" } else { "" }
            );
            lines.push(Line::from(Span::styled(label, style)));
        }
    }

    f.render_widget(
        Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: true }),
        area,
    );
}

fn render_card(f: &mut Frame, app: &App, area: Rect) {
    if let Some(node) = app.tree.focused_node() {
        let color = color_for_type(&node.org_type);
        let block = Block::default()
            .title(format!(" {} ", node.label))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(color));

        let parent_str = if let Some(parent_id) = &node.parent {
            if let Some(parent) = app.tree.nodes.get(parent_id) {
                format!("{} — {}", parent.data.label, parent.data.full_name)
            } else {
                "—".to_string()
            }
        } else {
            "— (apex)".to_string()
        };

        let mut text = vec![
            Line::from(Span::styled(&node.full_name, Style::default().bold())),
            Line::from(""),
            Line::from(vec![
                Span::raw("Type: "),
                Span::styled(label_for_type(&node.org_type), Style::default().fg(color)),
            ]),
            Line::from(format!("Echelon: E{}", node.echelon)),
            Line::from(vec![Span::raw("Reports to: "), Span::raw(parent_str)]),
        ];

        if let Some(source) = &node.source {
            text.push(Line::from(""));
            text.push(Line::from(Span::styled(
                "Authority:",
                Style::default().bold().fg(Color::Yellow),
            )));
            text.push(Line::from(Span::raw(source)));
        } else {
            text.push(Line::from(""));
            text.push(Line::from(Span::styled(
                "Authority: Unverified — reporting line to confirm",
                Style::default().italic().fg(Color::DarkGray),
            )));
        }

        if let Some(note) = node.meta.get("note") {
            text.push(Line::from(""));
            text.push(Line::from(Span::styled(
                format!("Note: {}", note),
                Style::default().italic().fg(Color::DarkGray),
            )));
        }

        f.render_widget(
            Paragraph::new(text)
                .block(block)
                .wrap(Wrap { trim: true }),
            area,
        );
    }
}

// =============================================================================
// Main
// =============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let json_path = if args.len() > 1 {
        &args[1]
    } else {
        "dod-org-data-research-ingest.json"
    };

    // Read JSON
    let json_str = fs::read_to_string(json_path)
        .map_err(|e| format!("Failed to read {}: {}", json_path, e))?;
    let org_data: OrgData =
        serde_json::from_str(&json_str).map_err(|e| format!("Invalid JSON: {}", e))?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new(org_data);
    let res = run_app(&mut terminal, app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {}", err);
    }

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                app.handle_key(key);
            }
        }
    }
}
