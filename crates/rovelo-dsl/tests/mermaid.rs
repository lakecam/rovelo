//! Mermaid flowchart import.
use rovelo_core::brand::Role;
use rovelo_core::model::*;
use rovelo_dsl::{Severity, from_mermaid, parse, serialize};

fn edges(d: &Diagram) -> Vec<(String, String, String, LineStyle, Arrow)> {
    d.edges
        .iter()
        .map(|e| {
            (
                e.from.clone(),
                e.to.clone(),
                e.label.clone(),
                e.line,
                e.arrow,
            )
        })
        .collect()
}

fn e(
    from: &str,
    to: &str,
    label: &str,
    line: LineStyle,
    arrow: Arrow,
) -> (String, String, String, LineStyle, Arrow) {
    (from.into(), to.into(), label.into(), line, arrow)
}

fn count(p: &rovelo_dsl::Parsed, sev: Severity) -> usize {
    p.diagnostics.iter().filter(|d| d.severity == sev).count()
}

#[test]
fn header_sets_direction() {
    for (h, dir) in [
        ("graph TD", Direction::Down),
        ("graph TB", Direction::Down),
        ("flowchart LR", Direction::Right),
        ("flowchart RL", Direction::Left),
        ("graph BT", Direction::Up),
        ("flowchart", Direction::Down),
    ] {
        let p = from_mermaid(&format!("{h}\nA --> B"));
        assert_eq!(p.diagram.direction, dir, "{h}");
        assert_eq!(p.diagram.kind, DiagramKind::Flowchart);
        assert!(p.diagnostics.is_empty(), "{h}: {:?}", p.diagnostics);
    }
    assert_eq!(
        count(&from_mermaid("graph SIDEWAYS\nA-->B"), Severity::Warning),
        1
    );
}

#[test]
fn every_node_shape() {
    let p = from_mermaid(
        "flowchart TD\nA[rect]\nB(round)\nC([pill])\nD{diamond}\nE((circle))\nF[(db)]\nG{{hex}}\nH[/para/]\nI[\\para2\\]\nJ>note]\nK[[sub]]\nL(((double)))",
    );
    let shapes: Vec<_> = p
        .diagram
        .nodes
        .iter()
        .map(|n| (n.id.as_str(), n.shape, n.label.as_str()))
        .collect();
    assert_eq!(
        shapes,
        vec![
            ("A", Shape::Rect, "rect"),
            ("B", Shape::Rounded, "round"),
            ("C", Shape::Pill, "pill"),
            ("D", Shape::Diamond, "diamond"),
            ("E", Shape::Circle, "circle"),
            ("F", Shape::Cylinder, "db"),
            ("G", Shape::Hexagon, "hex"),
            ("H", Shape::Parallelogram, "para"),
            ("I", Shape::Parallelogram, "para2"),
            ("J", Shape::Note, "note"),
            ("K", Shape::Rect, "sub"),
            ("L", Shape::Circle, "double"),
        ]
    );
    // The subroutine and double-circle shapes are approximated, and say so.
    assert_eq!(count(&p, Severity::Info), 2);
    assert_eq!(count(&p, Severity::Warning), 0);
}

#[test]
fn quoted_text_and_line_breaks() {
    let p = from_mermaid(
        "graph TD\nA[\"Line one<br/>line two (x) ] y\"]\nB(\"a<br>b<br />c\")\nC{\"is it?\"}",
    );
    let l: Vec<_> = p.diagram.nodes.iter().map(|n| n.label.as_str()).collect();
    assert_eq!(l, vec!["Line one\nline two (x) ] y", "a\nb\nc", "is it?"]);
}

#[test]
fn every_link_kind() {
    let p = from_mermaid(
        "graph LR\nA --> B\nA --- C\nA -.-> D\nA -.- E\nA ==> F\nA === G\nA <--> H\nA --o I\nA --x J\nA ---> K\nA -..-> L",
    );
    use Arrow::*;
    use LineStyle::*;
    assert_eq!(
        edges(&p.diagram),
        vec![
            e("A", "B", "", Solid, End),
            e("A", "C", "", Solid, Arrow::None),
            e("A", "D", "", Dashed, End),
            e("A", "E", "", Dashed, Arrow::None),
            e("A", "F", "", Solid, End),
            e("A", "G", "", Solid, Arrow::None),
            e("A", "H", "", Solid, Both),
            e("A", "I", "", Solid, End),
            e("A", "J", "", Solid, End),
            e("A", "K", "", Solid, End),
            e("A", "L", "", Dashed, End),
        ]
    );
    // `--o` and `--x` each warn.
    assert_eq!(count(&p, Severity::Warning), 2);
}

#[test]
fn link_labels() {
    let p = from_mermaid(
        "graph LR\nA -->|yes| B\nA -- no --> C\nA -. maybe .-> D\nA == loud ==> E\nA -->|\"quoted<br/>two\"| F\nA -- plain text --- G",
    );
    use Arrow::*;
    use LineStyle::*;
    assert_eq!(
        edges(&p.diagram),
        vec![
            e("A", "B", "yes", Solid, End),
            e("A", "C", "no", Solid, End),
            e("A", "D", "maybe", Dashed, End),
            e("A", "E", "loud", Solid, End),
            e("A", "F", "quoted\ntwo", Solid, End),
            e("A", "G", "plain text", Solid, Arrow::None),
        ]
    );
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
}

#[test]
fn chains_and_ampersands() {
    let p = from_mermaid("graph TD\nA --> B --> C\nA & B --> C & D\nE --> F;G --> H");
    let pairs: Vec<_> = p
        .diagram
        .edges
        .iter()
        .map(|e| format!("{}{}", e.from, e.to))
        .collect();
    assert_eq!(pairs, vec!["AB", "BC", "AC", "AD", "BC", "BD", "EF", "GH"]);
    assert_eq!(
        p.diagram
            .edges
            .iter()
            .map(|e| e.id.as_str())
            .collect::<Vec<_>>()[7],
        "e8"
    );
}

#[test]
fn node_text_on_later_mentions_overrides() {
    let p = from_mermaid("graph TD\nA --> B\nA[Start]\nB{Choose}");
    let n: Vec<_> = p
        .diagram
        .nodes
        .iter()
        .map(|n| (n.label.as_str(), n.shape))
        .collect();
    assert_eq!(n, vec![("Start", Shape::Rect), ("Choose", Shape::Diamond)]);
}

#[test]
fn subgraphs_become_groups() {
    let p = from_mermaid(
        "flowchart TB\n  subgraph one [First part]\n    A --> B\n  end\n  subgraph two\n    C\n  end\n  B --> C\n  subgraph \"Third thing\"\n    D\n  end",
    );
    let d = &p.diagram;
    let g: Vec<_> = d
        .groups
        .iter()
        .map(|g| (g.id.as_str(), g.label.as_str()))
        .collect();
    assert_eq!(
        g,
        vec![
            ("one", "First part"),
            ("two", "two"),
            ("third-thing", "Third thing")
        ]
    );
    let m: Vec<_> = d
        .nodes
        .iter()
        .map(|n| (n.id.as_str(), n.group.as_deref()))
        .collect();
    assert_eq!(
        m,
        vec![
            ("A", Some("one")),
            ("B", Some("one")),
            ("C", Some("two")),
            ("D", Some("third-thing"))
        ]
    );
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
    // Ids are slugged so the group serializes as `group Title {`.
    let p = from_mermaid("graph TD\nsubgraph Backend\nA\nend");
    assert!(serialize(&p.diagram).contains("group Backend {\n  A\n}"));
}

#[test]
fn nested_subgraphs_flatten_with_a_warning() {
    let p = from_mermaid("graph TD\nsubgraph outer\n A\n subgraph inner\n  B\n end\n C\nend");
    assert_eq!(p.diagram.groups.len(), 2);
    let m: Vec<_> = p
        .diagram
        .nodes
        .iter()
        .map(|n| n.group.as_deref().unwrap())
        .collect();
    assert_eq!(m, vec!["outer", "inner", "outer"]);
    assert_eq!(count(&p, Severity::Warning), 1);
    assert_eq!(p.diagnostics[0].line, 4);
}

#[test]
fn direction_inside_a_subgraph_is_ignored_with_a_warning() {
    let p = from_mermaid("graph TD\nsubgraph s\ndirection LR\nA --> B\nend");
    assert_eq!(p.diagram.direction, Direction::Down);
    assert_eq!(count(&p, Severity::Warning), 1);
    assert_eq!(p.diagram.edges.len(), 1);
}

#[test]
fn edges_to_a_subgraph_are_skipped_with_a_warning() {
    let p = from_mermaid("graph TD\nA --> sub\nsubgraph sub\n B\nend");
    assert!(p.diagram.edges.is_empty());
    assert_eq!(
        p.diagram
            .nodes
            .iter()
            .map(|n| n.id.as_str())
            .collect::<Vec<_>>(),
        vec!["A", "B"]
    );
    assert_eq!(count(&p, Severity::Warning), 1);
}

#[test]
fn styling_lines_are_dropped_with_one_info_each() {
    let p = from_mermaid(
        "graph TD\nA --> B\nclassDef big fill:#f9f\nclass A big\nstyle B fill:#bbf\nlinkStyle 0 stroke:red\nclick A \"https://x.y\"\nA:::big --> B",
    );
    assert_eq!(count(&p, Severity::Info), 6);
    assert_eq!(p.diagram.edges.len(), 2);
    let lines: Vec<_> = p.diagnostics.iter().map(|d| d.line).collect();
    assert_eq!(lines, vec![3, 4, 5, 6, 7, 8]);
}

#[test]
fn comments_and_blank_lines_are_ignored() {
    let p = from_mermaid(
        "%% top\n\ngraph TD\n%% note\nA --> B %% not a comment mid-line? still ends the text\n%%{init: {}}%%\n",
    );
    assert_eq!(p.diagram.nodes.len(), 2);
}

#[test]
fn front_matter_title() {
    let p = from_mermaid("---\ntitle: Checkout flow\n---\nflowchart LR\nA --> B");
    assert_eq!(p.diagram.title, "Checkout flow");
    assert_eq!(p.diagram.edges.len(), 1);
}

#[test]
fn non_flowcharts_are_one_error_and_empty() {
    for src in [
        "sequenceDiagram\nA->>B: hi",
        "classDiagram\nclass A",
        "gantt\ntitle x",
        "pie\n\"a\": 1",
        "erDiagram\nA ||--o{ B : has",
        "stateDiagram-v2\n[*] --> A",
        "A --> B",
        "",
    ] {
        let p = from_mermaid(src);
        assert_eq!(p.diagnostics.len(), 1, "{src}");
        assert_eq!(p.diagnostics[0].severity, Severity::Error, "{src}");
        assert!(
            p.diagram.nodes.is_empty() && p.diagram.edges.is_empty() && p.diagram.groups.is_empty(),
            "{src}"
        );
    }
}

#[test]
fn roles_follow_the_documented_rule() {
    // Diamonds are Warning. A pill is Success only if it is the first node and nothing points at it;
    // any other pill (like this end marker) stays Primary.
    let p = from_mermaid(
        "graph TD\nS([Start]) --> Q{Ok?}\nQ -->|yes| F([Finish])\nQ -->|no| P([Pill 2])\nX([Not first])",
    );
    let r: Vec<_> = p
        .diagram
        .nodes
        .iter()
        .map(|n| (n.id.as_str(), n.role))
        .collect();
    assert_eq!(
        r,
        vec![
            ("S", Role::Success),
            ("Q", Role::Warning),
            ("F", Role::Primary),
            ("P", Role::Primary),
            ("X", Role::Primary)
        ]
    );
    // A first pill that something points at is not a start marker.
    let p = from_mermaid("graph TD\nS([Loop]) --> A\nA --> S");
    assert_eq!(p.diagram.nodes[0].role, Role::Primary);
}

#[test]
fn garbled_statements_warn_and_move_on() {
    let p = from_mermaid("graph TD\nA --> B\n??? --> ###\nC -->\nD -- text\nE --> F");
    assert_eq!(p.diagram.edges.iter().filter(|e| e.from == "E").count(), 1);
    assert!(count(&p, Severity::Warning) >= 3);
}

#[test]
fn a_realistic_chart_round_trips_through_rovelo_text() {
    let src = "flowchart TD\n  A([Start]) --> B[Enter email]\n  B --> C{Valid?}\n  C -->|yes| D[(Save user)]\n  C -->|no| B\n  D --> E((Done))\n";
    let d = from_mermaid(src).diagram;
    let text = serialize(&d);
    let back = parse(&text).diagram;
    assert_eq!(back.nodes, d.nodes);
    assert_eq!(back.edges, d.edges);
    assert_eq!(serialize(&back), text);
}

#[test]
fn odd_but_valid_spellings() {
    let p = from_mermaid(
        "graph LR\nA-->B-->C\nmy-node --> other_node\nR~~~S\nK e1@--> L\nG@{ shape: rect, label: \"Hi there\" } --> H",
    );
    let pairs: Vec<_> = p
        .diagram
        .edges
        .iter()
        .map(|e| format!("{}>{}", e.from, e.to))
        .collect();
    assert_eq!(
        pairs,
        vec!["A>B", "B>C", "my-node>other_node", "K>L", "G>H"]
    );
    assert_eq!(p.diagram.node("G").unwrap().label, "Hi there");
    assert!(
        p.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Info && d.message.contains("invisible"))
    );
}
