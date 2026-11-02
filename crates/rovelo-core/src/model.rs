use crate::brand::{BrandKit, Role};
use serde::{Deserialize, Serialize};

pub type Id = String;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Point {
        Point { x, y }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Size {
    pub w: f32,
    pub h: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn center(&self) -> Point {
        Point::new(self.x + self.w / 2.0, self.y + self.h / 2.0)
    }

    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.x && p.x <= self.x + self.w && p.y >= self.y && p.y <= self.y + self.h
    }

    pub fn union(&self, o: &Rect) -> Rect {
        let x = self.x.min(o.x);
        let y = self.y.min(o.y);
        let r = (self.x + self.w).max(o.x + o.w);
        let b = (self.y + self.h).max(o.y + o.h);
        Rect {
            x,
            y,
            w: r - x,
            h: b - y,
        }
    }

    pub fn inflate(&self, d: f32) -> Rect {
        Rect {
            x: self.x - d,
            y: self.y - d,
            w: self.w + 2.0 * d,
            h: self.h + 2.0 * d,
        }
    }
}

/// What a diagram is for. It picks defaults (direction, vocabulary, starter shapes);
/// the same model draws all three.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DiagramKind {
    #[default]
    Flowchart,
    /// Screens and the actions between them.
    UserFlow,
    /// Steps, decisions and hand-offs, with the to-dos hanging off them.
    TaskFlow,
}

impl DiagramKind {
    pub const ALL: [DiagramKind; 3] = [
        DiagramKind::Flowchart,
        DiagramKind::UserFlow,
        DiagramKind::TaskFlow,
    ];

    pub fn name(self) -> &'static str {
        match self {
            DiagramKind::Flowchart => "flowchart",
            DiagramKind::UserFlow => "userflow",
            DiagramKind::TaskFlow => "taskflow",
        }
    }

    pub fn from_name(s: &str) -> Option<DiagramKind> {
        DiagramKind::ALL.into_iter().find(|k| k.name() == s)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    #[default]
    Down,
    Right,
    Left,
    Up,
}

impl Direction {
    pub fn name(self) -> &'static str {
        match self {
            Direction::Down => "down",
            Direction::Right => "right",
            Direction::Left => "left",
            Direction::Up => "up",
        }
    }

    /// Accepts the canonical words and Mermaid's TB/TD/LR/RL/BT.
    pub fn from_name(s: &str) -> Option<Direction> {
        match s {
            "down" | "TB" | "TD" => Some(Direction::Down),
            "right" | "LR" => Some(Direction::Right),
            "left" | "RL" => Some(Direction::Left),
            "up" | "BT" => Some(Direction::Up),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Shape {
    #[default]
    Rect,
    Rounded,
    Pill,
    Diamond,
    Circle,
    Parallelogram,
    Hexagon,
    Cylinder,
    Document,
    /// A phone-shaped screen, for user flows.
    Screen,
    /// Sticky-note style callout.
    Note,
}

impl Shape {
    pub const ALL: [Shape; 11] = [
        Shape::Rect,
        Shape::Rounded,
        Shape::Pill,
        Shape::Diamond,
        Shape::Circle,
        Shape::Parallelogram,
        Shape::Hexagon,
        Shape::Cylinder,
        Shape::Document,
        Shape::Screen,
        Shape::Note,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Shape::Rect => "rect",
            Shape::Rounded => "rounded",
            Shape::Pill => "pill",
            Shape::Diamond => "diamond",
            Shape::Circle => "circle",
            Shape::Parallelogram => "parallelogram",
            Shape::Hexagon => "hexagon",
            Shape::Cylinder => "cylinder",
            Shape::Document => "document",
            Shape::Screen => "screen",
            Shape::Note => "note",
        }
    }

    pub fn from_name(s: &str) -> Option<Shape> {
        Shape::ALL.into_iter().find(|k| k.name() == s)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LineStyle {
    #[default]
    Solid,
    Dashed,
    Dotted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Arrow {
    #[default]
    End,
    Both,
    None,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub id: Id,
    pub label: String,
    #[serde(default)]
    pub shape: Shape,
    /// Top-left corner. `None` until a layout or a drag sets it.
    #[serde(default)]
    pub pos: Option<Point>,
    /// Manual size override; `None` means measure from the label.
    #[serde(default)]
    pub size: Option<Size>,
    #[serde(default)]
    pub group: Option<Id>,
    #[serde(default = "default_role")]
    pub role: Role,
    /// Short icon word drawn before the label (e.g. "user", "lock"); see `icons` in the SVG exporter.
    #[serde(default)]
    pub icon: Option<String>,
}

fn default_role() -> Role {
    Role::Primary
}

impl Node {
    pub fn new(id: impl Into<Id>, label: impl Into<String>) -> Node {
        Node {
            id: id.into(),
            label: label.into(),
            shape: Shape::default(),
            pos: None,
            size: None,
            group: None,
            role: Role::Primary,
            icon: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    pub id: Id,
    pub from: Id,
    pub to: Id,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub line: LineStyle,
    #[serde(default)]
    pub arrow: Arrow,
    /// `None` uses the neutral role.
    #[serde(default)]
    pub role: Option<Role>,
}

impl Edge {
    pub fn new(id: impl Into<Id>, from: impl Into<Id>, to: impl Into<Id>) -> Edge {
        Edge {
            id: id.into(),
            from: from.into(),
            to: to.into(),
            label: String::new(),
            line: LineStyle::Solid,
            arrow: Arrow::End,
            role: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Group {
    pub id: Id,
    pub label: String,
    #[serde(default = "neutral_role")]
    pub role: Role,
}

fn neutral_role() -> Role {
    Role::Neutral
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Low,
    #[default]
    Normal,
    High,
}

impl Priority {
    pub fn name(self) -> &'static str {
        match self {
            Priority::Low => "low",
            Priority::Normal => "normal",
            Priority::High => "high",
        }
    }

    pub fn from_name(s: &str) -> Option<Priority> {
        [Priority::Low, Priority::Normal, Priority::High]
            .into_iter()
            .find(|p| p.name() == s)
    }
}

/// A to-do. It can stand alone or hang off a node, so a flow doubles as its own checklist.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Task {
    pub id: Id,
    pub title: String,
    #[serde(default)]
    pub done: bool,
    /// Node this task belongs to.
    #[serde(default)]
    pub node: Option<Id>,
    /// ISO date, `YYYY-MM-DD`.
    #[serde(default)]
    pub due: Option<String>,
    #[serde(default)]
    pub priority: Priority,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub notes: String,
}

impl Task {
    pub fn new(id: impl Into<Id>, title: impl Into<String>) -> Task {
        Task {
            id: id.into(),
            title: title.into(),
            done: false,
            node: None,
            due: None,
            priority: Priority::Normal,
            tags: vec![],
            notes: String::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Diagram {
    pub title: String,
    #[serde(default)]
    pub kind: DiagramKind,
    #[serde(default)]
    pub direction: Direction,
    #[serde(default)]
    pub brand: BrandKit,
    #[serde(default)]
    pub nodes: Vec<Node>,
    #[serde(default)]
    pub edges: Vec<Edge>,
    #[serde(default)]
    pub groups: Vec<Group>,
    #[serde(default)]
    pub tasks: Vec<Task>,
}

impl Default for Diagram {
    fn default() -> Self {
        Diagram {
            title: "Untitled".into(),
            kind: DiagramKind::Flowchart,
            direction: Direction::Down,
            brand: BrandKit::default(),
            nodes: vec![],
            edges: vec![],
            groups: vec![],
            tasks: vec![],
        }
    }
}

impl Diagram {
    pub fn node(&self, id: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }

    pub fn node_mut(&mut self, id: &str) -> Option<&mut Node> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }

    pub fn group(&self, id: &str) -> Option<&Group> {
        self.groups.iter().find(|g| g.id == id)
    }

    pub fn task_mut(&mut self, id: &str) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|t| t.id == id)
    }

    pub fn tasks_for<'a>(&'a self, node: &'a str) -> impl Iterator<Item = &'a Task> + 'a {
        self.tasks
            .iter()
            .filter(move |t| t.node.as_deref() == Some(node))
    }

    /// `(done, total)` over every task.
    pub fn progress(&self) -> (usize, usize) {
        (
            self.tasks.iter().filter(|t| t.done).count(),
            self.tasks.len(),
        )
    }

    /// A fresh id with the given prefix that no node, edge, group or task uses.
    pub fn fresh_id(&self, prefix: &str) -> Id {
        let used = |id: &str| {
            self.nodes.iter().any(|n| n.id == id)
                || self.edges.iter().any(|e| e.id == id)
                || self.groups.iter().any(|g| g.id == id)
                || self.tasks.iter().any(|t| t.id == id)
        };
        (1..)
            .map(|i| format!("{prefix}{i}"))
            .find(|id| !used(id))
            .unwrap()
    }

    /// Add a node, or return the existing one with that id.
    pub fn ensure_node(&mut self, id: &str, label: &str) -> &mut Node {
        if let Some(i) = self.nodes.iter().position(|n| n.id == id) {
            return &mut self.nodes[i];
        }
        self.nodes.push(Node::new(id, label));
        self.nodes.last_mut().unwrap()
    }

    pub fn add_edge(&mut self, from: &str, to: &str) -> &mut Edge {
        let id = self.fresh_id("e");
        self.edges.push(Edge::new(id, from, to));
        self.edges.last_mut().unwrap()
    }

    /// Remove a node with its edges, and detach its tasks (the tasks stay, unlinked).
    pub fn remove_node(&mut self, id: &str) {
        self.nodes.retain(|n| n.id != id);
        self.edges.retain(|e| e.from != id && e.to != id);
        for t in &mut self.tasks {
            if t.node.as_deref() == Some(id) {
                t.node = None;
            }
        }
    }

    /// Remove a group; its members become ungrouped.
    pub fn remove_group(&mut self, id: &str) {
        self.groups.retain(|g| g.id != id);
        for n in &mut self.nodes {
            if n.group.as_deref() == Some(id) {
                n.group = None;
            }
        }
    }

    /// Drop edges and links that point at things that no longer exist.
    pub fn repair(&mut self) {
        let ids: std::collections::HashSet<_> = self.nodes.iter().map(|n| n.id.clone()).collect();
        self.edges
            .retain(|e| ids.contains(&e.from) && ids.contains(&e.to));
        let gids: std::collections::HashSet<_> = self.groups.iter().map(|g| g.id.clone()).collect();
        for n in &mut self.nodes {
            if n.group.as_ref().is_some_and(|g| !gids.contains(g)) {
                n.group = None;
            }
        }
        for t in &mut self.tasks {
            if t.node.as_ref().is_some_and(|n| !ids.contains(n)) {
                t.node = None;
            }
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("diagram serialises")
    }

    pub fn from_json(s: &str) -> Result<Diagram, String> {
        let mut d: Diagram = serde_json::from_str(s).map_err(|e| e.to_string())?;
        d.repair();
        Ok(d)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Diagram {
        let mut d = Diagram::default();
        d.ensure_node("a", "Start");
        d.ensure_node("b", "Next");
        d.add_edge("a", "b");
        let mut t = Task::new("t1", "Write copy");
        t.node = Some("b".into());
        d.tasks.push(t);
        d
    }

    #[test]
    fn removing_a_node_drops_edges_and_unlinks_tasks() {
        let mut d = sample();
        d.remove_node("b");
        assert!(d.edges.is_empty());
        assert_eq!(d.tasks.len(), 1);
        assert_eq!(d.tasks[0].node, None);
    }

    #[test]
    fn json_round_trip_and_repair() {
        let d = sample();
        let back = Diagram::from_json(&d.to_json()).unwrap();
        assert_eq!(d, back);
        let broken = d.to_json().replace("\"to\": \"b\"", "\"to\": \"zzz\"");
        assert!(Diagram::from_json(&broken).unwrap().edges.is_empty());
    }

    #[test]
    fn fresh_ids_skip_used() {
        let d = sample();
        assert_eq!(d.fresh_id("e"), "e2");
        assert_eq!(d.fresh_id("t"), "t2");
    }

    #[test]
    fn progress_counts() {
        let mut d = sample();
        d.tasks[0].done = true;
        assert_eq!(d.progress(), (1, 1));
    }
}
