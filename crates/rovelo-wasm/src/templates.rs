//! Starter diagrams, built in code (same content as the desktop app's templates).
use crate::dates::plus_days;
use rovelo_core::brand::{BrandKit, Role};
use rovelo_core::layout::auto_layout;
use rovelo_core::model::{
    Arrow, Diagram, DiagramKind, Direction, Group, LineStyle, Priority, Shape, Task,
};

struct B<'a>(&'a mut Diagram);

impl B<'_> {
    fn node(&mut self, id: &str, label: &str, shape: Shape, role: Role) -> &mut Self {
        let n = self.0.ensure_node(id, label);
        n.shape = shape;
        n.role = role;
        self
    }
    fn icon(&mut self, id: &str, icon: &str) -> &mut Self {
        if let Some(n) = self.0.node_mut(id) {
            n.icon = Some(icon.into());
        }
        self
    }
    fn edge(&mut self, from: &str, to: &str, label: &str) -> &mut Self {
        self.0.add_edge(from, to).label = label.into();
        self
    }
    fn edge_styled(
        &mut self,
        from: &str,
        to: &str,
        label: &str,
        line: LineStyle,
        role: Role,
    ) -> &mut Self {
        let e = self.0.add_edge(from, to);
        e.label = label.into();
        e.line = line;
        e.role = Some(role);
        self
    }
    fn task(
        &mut self,
        node: &str,
        title: &str,
        done: bool,
        due: Option<String>,
        p: Priority,
        tags: &[&str],
    ) -> &mut Self {
        let id = self.0.fresh_id("t");
        let mut t = Task::new(id, title);
        t.node = Some(node.into());
        t.done = done;
        t.due = due;
        t.priority = p;
        t.tags = tags.iter().map(|s| s.to_string()).collect();
        self.0.tasks.push(t);
        self
    }
}

#[cfg(test)]
pub const NAMES: [&str; 3] = ["userflow", "taskflow", "blank"];

/// Build a template by name (`userflow`, `taskflow`, anything else is blank). `today` is `YYYY-MM-DD`.
pub fn build(name: &str, today: &str, brand: &BrandKit) -> Diagram {
    let mut d = Diagram {
        brand: brand.clone(),
        ..Diagram::default()
    };
    let due = |n: i64| Some(plus_days(today, n));
    match name {
        "userflow" => {
            d.title = "Sign-up flow".into();
            d.kind = DiagramKind::UserFlow;
            d.direction = Direction::Right;
            let mut b = B(&mut d);
            b.node("landing", "Landing page", Shape::Screen, Role::Primary)
                .node(
                    "has_account",
                    "Has an account?",
                    Shape::Diamond,
                    Role::Accent,
                )
                .node("login", "Log in", Shape::Screen, Role::Secondary)
                .node("signup", "Sign up", Shape::Screen, Role::Primary)
                .node("verify", "Verify email", Shape::Screen, Role::Primary)
                .node("onboarding", "Onboarding", Shape::Screen, Role::Primary)
                .node("dashboard", "Dashboard", Shape::Screen, Role::Success)
                .node("bad_login", "Wrong password", Shape::Note, Role::Danger)
                .icon("landing", "home")
                .icon("verify", "mail")
                .icon("has_account", "branch")
                .icon("dashboard", "flag")
                .edge("landing", "has_account", "Get started")
                .edge("has_account", "login", "Yes")
                .edge("has_account", "signup", "No")
                .edge("signup", "verify", "Submit")
                .edge("verify", "onboarding", "Code ok")
                .edge("onboarding", "dashboard", "Finish")
                .edge("login", "dashboard", "Success")
                .edge_styled(
                    "login",
                    "bad_login",
                    "Fails",
                    LineStyle::Dashed,
                    Role::Danger,
                )
                .task(
                    "landing",
                    "Write hero copy",
                    true,
                    None,
                    Priority::Normal,
                    &["copy"],
                )
                .task(
                    "signup",
                    "Design the form states",
                    false,
                    due(5),
                    Priority::High,
                    &["design"],
                )
                .task(
                    "signup",
                    "Validate email on the client",
                    false,
                    due(9),
                    Priority::Normal,
                    &["dev"],
                )
                .task(
                    "verify",
                    "Send the code by email",
                    false,
                    due(7),
                    Priority::High,
                    &["dev"],
                )
                .task(
                    "onboarding",
                    "Pick three onboarding questions",
                    false,
                    None,
                    Priority::Low,
                    &["research"],
                );
            if let Some(e) = d.edges.iter_mut().find(|e| e.to == "bad_login") {
                e.arrow = Arrow::End;
            }
        }
        "taskflow" => {
            d.title = "Release checklist".into();
            d.kind = DiagramKind::TaskFlow;
            d.direction = Direction::Down;
            d.groups.push(Group {
                id: "g_qa".into(),
                label: "Quality".into(),
                role: Role::Secondary,
            });
            let mut b = B(&mut d);
            b.node("start", "Feature ready", Shape::Pill, Role::Neutral)
                .node("review", "Code review", Shape::Rounded, Role::Primary)
                .node("approved", "Approved?", Shape::Diamond, Role::Accent)
                .node("fix", "Address feedback", Shape::Rounded, Role::Warning)
                .node("tests", "Run test suite", Shape::Rounded, Role::Secondary)
                .node("passes", "All green?", Shape::Diamond, Role::Accent)
                .node(
                    "notes",
                    "Write release notes",
                    Shape::Document,
                    Role::Primary,
                )
                .node("ship", "Ship it", Shape::Pill, Role::Success)
                .icon("start", "play")
                .icon("approved", "branch")
                .icon("passes", "branch")
                .icon("ship", "flag")
                .edge("start", "review", "")
                .edge("review", "approved", "")
                .edge("approved", "fix", "No")
                .edge_styled(
                    "fix",
                    "review",
                    "Re-review",
                    LineStyle::Dashed,
                    Role::Warning,
                )
                .edge("approved", "tests", "Yes")
                .edge("tests", "passes", "")
                .edge("passes", "notes", "Yes")
                .edge_styled("passes", "fix", "No", LineStyle::Dotted, Role::Danger)
                .edge("notes", "ship", "")
                .task(
                    "review",
                    "Request two reviewers",
                    true,
                    None,
                    Priority::Normal,
                    &[],
                )
                .task(
                    "review",
                    "Link the design file in the PR",
                    false,
                    due(1),
                    Priority::Normal,
                    &["docs"],
                )
                .task(
                    "tests",
                    "Run the suite on a clean checkout",
                    false,
                    due(2),
                    Priority::High,
                    &["qa"],
                )
                .task(
                    "tests",
                    "Check the flaky test list",
                    false,
                    None,
                    Priority::Low,
                    &["qa"],
                )
                .task(
                    "notes",
                    "Draft the changelog entry",
                    false,
                    due(3),
                    Priority::Normal,
                    &["docs"],
                )
                .task(
                    "ship",
                    "Tag the release and announce it",
                    false,
                    due(4),
                    Priority::High,
                    &[],
                );
            for id in ["tests", "passes"] {
                if let Some(n) = d.node_mut(id) {
                    n.group = Some("g_qa".into());
                }
            }
        }
        _ => d.title = "Untitled".into(),
    }
    auto_layout(&mut d);
    d
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn templates_are_consistent() {
        for name in NAMES {
            let d = build(name, "2026-11-02", &BrandKit::slate());
            let mut seen = HashSet::new();
            let ids = d
                .nodes
                .iter()
                .map(|n| &n.id)
                .chain(d.edges.iter().map(|e| &e.id))
                .chain(d.groups.iter().map(|g| &g.id))
                .chain(d.tasks.iter().map(|t| &t.id));
            assert!(ids.into_iter().all(|i| seen.insert(i.clone())), "{name}");
            let mut fixed = d.clone();
            fixed.repair();
            assert_eq!(d.edges, fixed.edges, "{name}");
            assert_eq!(d.tasks, fixed.tasks, "{name}");
            assert!(d.nodes.iter().all(|n| n.pos.is_some()), "{name} placed");
        }
    }

    #[test]
    fn blank_empty_taskflow_has_tasks_with_dates() {
        assert!(
            build("blank", "2026-11-02", &BrandKit::slate())
                .nodes
                .is_empty()
        );
        let t = build("taskflow", "2026-11-02", &BrandKit::slate());
        assert!(
            t.tasks
                .iter()
                .any(|t| t.due.as_deref() == Some("2026-11-03"))
        );
        assert!(t.nodes.iter().any(|n| n.group.is_some()));
    }
}
