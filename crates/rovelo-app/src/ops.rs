//! Diagram edits that do not touch the UI. Each is a single user intent.
use rovelo_core::geometry;
use rovelo_core::model::{Diagram, Group, Id, Node, Point, Shape, Task};

#[derive(Default, Clone, PartialEq, Debug)]
pub struct Selection {
    pub nodes: Vec<Id>,
    pub edges: Vec<Id>,
    pub groups: Vec<Id>,
}

impl Selection {
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.edges.clear();
        self.groups.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.edges.is_empty() && self.groups.is_empty()
    }

    pub fn only_node(&mut self, id: &str) {
        self.clear();
        self.nodes.push(id.to_string());
    }

    pub fn only_edge(&mut self, id: &str) {
        self.clear();
        self.edges.push(id.to_string());
    }

    pub fn only_group(&mut self, id: &str) {
        self.clear();
        self.groups.push(id.to_string());
    }

    pub fn has_node(&self, id: &str) -> bool {
        self.nodes.iter().any(|n| n == id)
    }

    pub fn toggle_node(&mut self, id: &str) {
        if let Some(i) = self.nodes.iter().position(|n| n == id) {
            self.nodes.remove(i);
        } else {
            self.nodes.push(id.to_string());
        }
    }

    pub fn toggle_edge(&mut self, id: &str) {
        if let Some(i) = self.edges.iter().position(|n| n == id) {
            self.edges.remove(i);
        } else {
            self.edges.push(id.to_string());
        }
    }

    /// Forget ids that no longer exist in the diagram.
    pub fn prune(&mut self, d: &Diagram) {
        self.nodes.retain(|id| d.node(id).is_some());
        self.edges.retain(|id| d.edges.iter().any(|e| &e.id == id));
        self.groups.retain(|id| d.group(id).is_some());
    }
}

/// Add a node centred on `center`; returns its id.
pub fn add_node_at(d: &mut Diagram, center: Point, label: &str) -> Id {
    let id = d.fresh_id("n");
    let mut n = Node::new(id.clone(), label);
    n.shape = Shape::Rounded;
    let size = geometry::node_size(d, &n);
    n.pos = Some(Point::new(center.x - size.w / 2.0, center.y - size.h / 2.0));
    d.nodes.push(n);
    id
}

/// Connect two existing, distinct nodes unless that exact edge exists. Returns the edge id.
pub fn connect(d: &mut Diagram, from: &str, to: &str) -> Option<Id> {
    if from == to || d.node(from).is_none() || d.node(to).is_none() {
        return None;
    }
    if d.edges.iter().any(|e| e.from == from && e.to == to) {
        return None;
    }
    Some(d.add_edge(from, to).id.clone())
}

/// Remove the selected nodes (with their edges), edges and groups.
pub fn delete_selection(d: &mut Diagram, sel: &Selection) {
    for id in &sel.nodes {
        d.remove_node(id);
    }
    d.edges.retain(|e| !sel.edges.contains(&e.id));
    for g in &sel.groups {
        d.remove_group(g);
    }
}

/// Clone nodes (and their tasks and the edges between them), shifted by `offset`.
pub fn duplicate(d: &mut Diagram, ids: &[Id], offset: (f32, f32)) -> Vec<Id> {
    let mut map: Vec<(Id, Id)> = Vec::new();
    for id in ids {
        let Some(src) = d.node(id).cloned() else {
            continue;
        };
        let new_id = d.fresh_id("n");
        let mut n = src;
        n.id = new_id.clone();
        n.pos = n.pos.map(|p| Point::new(p.x + offset.0, p.y + offset.1));
        d.nodes.push(n);
        map.push((id.clone(), new_id));
    }
    let find = |old: &str| map.iter().find(|(o, _)| o == old).map(|(_, n)| n.clone());
    let edges: Vec<_> = d
        .edges
        .iter()
        .filter_map(|e| Some((find(&e.from)?, find(&e.to)?, e.clone())))
        .collect();
    for (from, to, e) in edges {
        let id = d.fresh_id("e");
        let mut ne = e;
        ne.id = id;
        ne.from = from;
        ne.to = to;
        d.edges.push(ne);
    }
    let tasks: Vec<_> = d
        .tasks
        .iter()
        .filter_map(|t| Some((find(t.node.as_deref()?)?, t.clone())))
        .collect();
    for (node, t) in tasks {
        let id = d.fresh_id("t");
        let mut nt = t;
        nt.id = id;
        nt.node = Some(node);
        nt.done = false;
        d.tasks.push(nt);
    }
    map.into_iter().map(|(_, n)| n).collect()
}

/// Put the nodes in a new group. Returns its id, or `None` when no node exists.
pub fn group_nodes(d: &mut Diagram, ids: &[Id]) -> Option<Id> {
    let members: Vec<Id> = ids
        .iter()
        .filter(|i| d.node(i).is_some())
        .cloned()
        .collect();
    if members.is_empty() {
        return None;
    }
    let gid = d.fresh_id("g");
    d.groups.push(Group {
        id: gid.clone(),
        label: "Group".into(),
        // DESIGN.md: groups default to the kit's secondary colour
        role: rovelo_core::brand::Role::Secondary,
    });
    for m in &members {
        if let Some(n) = d.node_mut(m) {
            n.group = Some(gid.clone());
        }
    }
    Some(gid)
}

pub fn nudge(d: &mut Diagram, ids: &[Id], dx: f32, dy: f32) {
    for id in ids {
        if let Some(n) = d.node_mut(id) {
            let p = n.pos.unwrap_or_default();
            n.pos = Some(Point::new(p.x + dx, p.y + dy));
        }
    }
}

/// Add a to-do, optionally hanging off a node. Blank titles are ignored.
pub fn add_task(d: &mut Diagram, title: &str, node: Option<&str>) -> Option<Id> {
    let title = title.trim();
    if title.is_empty() {
        return None;
    }
    let id = d.fresh_id("t");
    let mut t = Task::new(id.clone(), title);
    t.node = node.filter(|n| d.node(n).is_some()).map(str::to_string);
    d.tasks.push(t);
    Some(id)
}

/// Rebuild a parsed diagram on top of the current one: brand and positions are kept by id.
pub fn merge_parsed(old: &Diagram, mut new: Diagram) -> Diagram {
    new.brand = old.brand.clone();
    for n in &mut new.nodes {
        if let Some(o) = old.node(&n.id) {
            if n.pos.is_none() {
                n.pos = o.pos;
            }
            if n.size.is_none() {
                n.size = o.size;
            }
        }
    }
    new.repair();
    new
}

#[cfg(test)]
mod tests {
    use super::*;
    use rovelo_core::brand::BrandKit;

    fn base() -> Diagram {
        let mut d = Diagram::default();
        d.ensure_node("a", "A").pos = Some(Point::new(0.0, 0.0));
        d.ensure_node("b", "B").pos = Some(Point::new(100.0, 0.0));
        d.add_edge("a", "b");
        add_task(&mut d, "do it", Some("a"));
        d
    }

    #[test]
    fn new_groups_take_the_kit_secondary_colour() {
        let mut d = base();
        let g = group_nodes(&mut d, &["a".into(), "b".into()]).unwrap();
        let g = d.groups.iter().find(|x| x.id == g).unwrap();
        assert_eq!(g.role, rovelo_core::brand::Role::Secondary);
    }

    #[test]
    fn add_node_gets_fresh_unique_ids() {
        let mut d = base();
        let a = add_node_at(&mut d, Point::new(50.0, 50.0), "x");
        let b = add_node_at(&mut d, Point::new(50.0, 50.0), "y");
        assert_ne!(a, b);
        assert_eq!(d.nodes.len(), 4);
        assert!(d.node(&a).unwrap().pos.is_some());
    }

    #[test]
    fn connect_rejects_self_missing_and_duplicates() {
        let mut d = base();
        assert!(connect(&mut d, "a", "a").is_none());
        assert!(connect(&mut d, "a", "zzz").is_none());
        assert!(connect(&mut d, "a", "b").is_none());
        assert!(connect(&mut d, "b", "a").is_some());
        assert_eq!(d.edges.len(), 2);
    }

    #[test]
    fn delete_selection_handles_stale_ids() {
        let mut d = base();
        let sel = Selection {
            nodes: vec!["a".into(), "ghost".into()],
            edges: vec!["nope".into()],
            groups: vec![],
        };
        delete_selection(&mut d, &sel);
        assert_eq!(d.nodes.len(), 1);
        assert!(d.edges.is_empty());
        assert_eq!(d.tasks[0].node, None);
        delete_selection(&mut Diagram::default(), &sel);
    }

    #[test]
    fn duplicate_copies_internal_edges_and_tasks() {
        let mut d = base();
        let ids = duplicate(
            &mut d,
            &["a".into(), "b".into(), "ghost".into()],
            (24.0, 24.0),
        );
        assert_eq!(ids.len(), 2);
        assert_eq!(d.nodes.len(), 4);
        assert_eq!(d.edges.len(), 2);
        assert_eq!(d.tasks.len(), 2);
        let copy = d.node(&ids[0]).unwrap();
        assert_eq!(copy.pos, Some(Point::new(24.0, 24.0)));
        assert_eq!(d.tasks[1].node.as_deref(), Some(ids[0].as_str()));
        let mut one = base();
        duplicate(&mut one, &["a".into()], (8.0, 8.0));
        assert_eq!(one.edges.len(), 1);
    }

    #[test]
    fn grouping() {
        let mut d = base();
        assert!(group_nodes(&mut d, &["ghost".into()]).is_none());
        let g = group_nodes(&mut d, &["a".into(), "b".into()]).unwrap();
        assert_eq!(d.node("a").unwrap().group.as_deref(), Some(g.as_str()));
        assert_eq!(d.groups.len(), 1);
    }

    #[test]
    fn blank_task_is_ignored_and_bad_node_unlinks() {
        let mut d = base();
        assert!(add_task(&mut d, "   ", None).is_none());
        let id = add_task(&mut d, " hi ", Some("ghost")).unwrap();
        let t = d.tasks.iter().find(|t| t.id == id).unwrap();
        assert_eq!(t.title, "hi");
        assert_eq!(t.node, None);
    }

    #[test]
    fn merge_keeps_positions_and_brand() {
        let mut old = base();
        old.brand = BrandKit::midnight();
        let mut new = Diagram::default();
        new.ensure_node("a", "A2");
        new.ensure_node("c", "C");
        let m = merge_parsed(&old, new);
        assert_eq!(m.node("a").unwrap().pos, Some(Point::new(0.0, 0.0)));
        assert_eq!(m.node("c").unwrap().pos, None);
        assert_eq!(m.brand.name, "Midnight");
    }

    #[test]
    fn selection_prune_and_toggle() {
        let d = base();
        let mut s = Selection::default();
        s.only_node("a");
        s.toggle_node("b");
        s.toggle_node("ghost");
        s.prune(&d);
        assert_eq!(s.nodes, vec!["a".to_string(), "b".to_string()]);
        s.toggle_node("a");
        assert!(!s.has_node("a"));
    }

    #[test]
    fn nudge_moves_unplaced_nodes_too() {
        let mut d = Diagram::default();
        d.ensure_node("a", "A");
        nudge(&mut d, &["a".into()], 8.0, -8.0);
        assert_eq!(d.node("a").unwrap().pos, Some(Point::new(8.0, -8.0)));
    }
}
