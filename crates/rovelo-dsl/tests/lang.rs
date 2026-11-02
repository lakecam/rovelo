//! The rovelo language, statement by statement.
use rovelo_core::brand::{BrandKit, EdgeRouting, NodeFill, Rgb, Role};
use rovelo_core::model::*;
use rovelo_dsl::{Severity, parse, serialize};

fn warnings(src: &str) -> Vec<String> {
    parse(src)
        .diagnostics
        .into_iter()
        .filter(|d| d.severity != Severity::Info)
        .map(|d| format!("{}: {}", d.line, d.message))
        .collect()
}

fn ids(d: &Diagram) -> Vec<&str> {
    d.nodes.iter().map(|n| n.id.as_str()).collect()
}

#[test]
fn the_spec_example_parses_without_diagnostics() {
    let src = r#"%% rovelo 1
taskflow right
title: Onboarding
brand: midnight
brand.primary: #3b5bdb
brand.background: #fafaf9
brand.font: Inter, sans-serif
brand.radius: 10
brand.stroke: 1.5
brand.size: 14
brand.routing: orthogonal
brand.fill: soft
brand.shadow: on
brand.grid: off
signup = Sign up [shape: rounded, role: primary, icon: user]
Sign up -> Verify email
signup -> verify: Email link [line: dashed, arrow: both, role: danger]
a -> b -> c
a -> b, c
group Auth { signup; verify }
group flow = "Auth flow" [role: accent] {
  c
}
todo signup: Write copy [done, due: 2026-11-10, priority: high, tags: copy ux, notes: "see brief"]
todo: Free-floating task [id: t9]
// a comment
"#;
    let p = parse(src);
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
    let d = p.diagram;
    assert_eq!(d.kind, DiagramKind::TaskFlow);
    assert_eq!(d.direction, Direction::Right);
    assert_eq!(d.title, "Onboarding");
    assert_eq!(d.brand.name, "Midnight");
    assert_eq!(d.brand.palette.primary, Rgb(0x3b, 0x5b, 0xdb));
    assert_eq!(d.brand.font_family, "Inter, sans-serif");
    assert_eq!(d.brand.edge_routing, EdgeRouting::Orthogonal);
    assert_eq!(d.brand.node_fill, NodeFill::Soft);
    assert!(d.brand.shadow && !d.brand.show_grid);
    let signup = d.node("signup").unwrap();
    assert_eq!(
        (signup.label.as_str(), signup.shape, signup.icon.as_deref()),
        ("Sign up", Shape::Rounded, Some("user"))
    );
    // `Sign up -> Verify email` resolved the first end by label, and made `verify-email`.
    assert!(d.node("verify-email").is_some());
    let e = d.edges.iter().find(|e| e.label == "Email link").unwrap();
    assert_eq!(
        (e.from.as_str(), e.to.as_str(), e.line, e.arrow, e.role),
        (
            "signup",
            "verify",
            LineStyle::Dashed,
            Arrow::Both,
            Some(Role::Danger)
        )
    );
    assert_eq!(d.groups.len(), 2);
    assert_eq!(d.node("signup").unwrap().group.as_deref(), Some("auth"));
    assert_eq!(d.node("c").unwrap().group.as_deref(), Some("flow"));
    assert_eq!(
        (d.groups[1].label.as_str(), d.groups[1].role),
        ("Auth flow", Role::Accent)
    );
    assert_eq!(d.tasks.len(), 2);
    let t = &d.tasks[0];
    assert_eq!(
        (t.id.as_str(), t.node.as_deref(), t.done, t.priority),
        ("t1", Some("signup"), true, Priority::High)
    );
    assert_eq!(
        (t.due.as_deref(), t.tags.clone(), t.notes.as_str()),
        (
            Some("2026-11-10"),
            vec!["copy".to_string(), "ux".into()],
            "see brief"
        )
    );
    assert_eq!(
        (d.tasks[1].id.as_str(), d.tasks[1].node.clone()),
        ("t9", None)
    );
}

#[test]
fn chains_and_fan_out_expand_to_separate_edges() {
    let d = parse("a -> b -> c").diagram;
    let pairs: Vec<_> = d
        .edges
        .iter()
        .map(|e| (e.from.as_str(), e.to.as_str()))
        .collect();
    assert_eq!(pairs, vec![("a", "b"), ("b", "c")]);
    let d = parse("a -> b, c\nb, c -> d").diagram;
    let pairs: Vec<_> = d
        .edges
        .iter()
        .map(|e| (e.from.as_str(), e.to.as_str()))
        .collect();
    assert_eq!(pairs, vec![("a", "b"), ("a", "c"), ("b", "d"), ("c", "d")]);
    assert_eq!(
        d.edges.iter().map(|e| e.id.as_str()).collect::<Vec<_>>(),
        vec!["e1", "e2", "e3", "e4"]
    );
}

#[test]
fn arrows_set_the_edge_ends() {
    let d = parse("a -> b\nb <-> c\nc -- d\nd --> e").diagram;
    let arrows: Vec<_> = d.edges.iter().map(|e| e.arrow).collect();
    assert_eq!(
        arrows,
        vec![Arrow::End, Arrow::Both, Arrow::None, Arrow::End]
    );
    // An explicit attribute wins over the arrow token.
    assert_eq!(
        parse("a -> b [arrow: none]").diagram.edges[0].arrow,
        Arrow::None
    );
}

#[test]
fn endpoint_resolves_id_then_label_then_creates() {
    let d = parse("x = Start\ny = x\nStart -> x -> New thing").diagram;
    assert_eq!(ids(&d), vec!["x", "y", "new-thing"]);
    // `Start` is x's label; `x` is x's id (not y's label, which is also "x").
    let pairs: Vec<_> = d
        .edges
        .iter()
        .map(|e| (e.from.as_str(), e.to.as_str()))
        .collect();
    assert_eq!(pairs, vec![("x", "x"), ("x", "new-thing")]);
}

#[test]
fn ids_from_labels_are_slugs_and_unique() {
    let d = parse("Sign up\nSign-up\nSIGN UP!\n???\n日本").diagram;
    assert_eq!(
        ids(&d),
        vec!["sign-up", "sign-up-2", "sign-up-3", "node", "日本"]
    );
}

#[test]
fn declaring_again_updates_the_node() {
    let p = parse("a -> b\na = Alpha [shape: pill]\na = Again");
    let a = p.diagram.node("a").unwrap();
    assert_eq!((a.label.as_str(), a.shape), ("Again", Shape::Pill));
    assert_eq!(p.diagnostics.len(), 1);
    assert_eq!(p.diagnostics[0].line, 3);
}

#[test]
fn bare_names_with_attributes_style_the_node() {
    let d = parse("Sign up [shape: pill, role: success]").diagram;
    assert_eq!(
        (d.nodes[0].shape, d.nodes[0].role),
        (Shape::Pill, Role::Success)
    );
}

#[test]
fn groups_single_line_multi_line_and_reopened() {
    let d =
        parse("a\nb\nc\ngroup Auth { a; b }\ngroup G = \"Other\" [role: accent] {\n  c\n}").diagram;
    assert_eq!(d.groups[0].id, "auth");
    assert_eq!(d.groups[1].label, "Other");
    assert_eq!(d.groups[1].role, Role::Accent);
    let g: Vec<_> = d.nodes.iter().map(|n| n.group.clone()).collect();
    assert_eq!(
        g,
        vec![Some("auth".into()), Some("auth".into()), Some("G".into())]
    );
    // Edges and declarations inside a block pull their nodes in.
    let d = parse("group G {\n  x = Ex\n  y -> z\n}").diagram;
    assert!(
        d.nodes.iter().all(|n| n.group.as_deref() == Some("g")),
        "{:?}",
        d.nodes
    );
    // An explicit id that exists is reopened, not duplicated.
    let d = parse("group g = G { a }\ngroup g = G2 { b }").diagram;
    assert_eq!(d.groups.len(), 1);
    assert_eq!((d.nodes.len(), d.groups[0].label.as_str()), (2, "G2"));
}

#[test]
fn nested_groups_warn_and_flatten() {
    let p = parse("group Outer {\n  a\n  group Inner { b }\n  c\n}");
    assert!(
        p.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Warning && d.message.contains("nest")),
        "{:?}",
        p.diagnostics
    );
    assert_eq!(p.diagram.groups.len(), 2);
    assert_eq!(p.diagram.node("b").unwrap().group.as_deref(), Some("inner"));
    assert_eq!(p.diagram.node("c").unwrap().group.as_deref(), Some("outer"));
}

#[test]
fn unclosed_and_stray_braces_are_reported() {
    assert!(
        warnings("group G {\n a")
            .iter()
            .any(|w| w.starts_with("1:") && w.contains("never closed"))
    );
    assert!(warnings("a }").iter().any(|w| w.contains("closes nothing")));
}

#[test]
fn todos_link_to_nodes_and_get_sequential_ids() {
    let d = parse("a = A\ntodo a: One\ntodo: Two [id: t5]\ntodo: Three\ntodo Four").diagram;
    let t: Vec<_> = d
        .tasks
        .iter()
        .map(|t| (t.id.as_str(), t.title.as_str(), t.node.as_deref()))
        .collect();
    assert_eq!(
        t,
        vec![
            ("t1", "One", Some("a")),
            ("t5", "Two", None),
            ("t3", "Three", None),
            ("t4", "Four", None)
        ]
    );
}

#[test]
fn todo_for_a_missing_node_creates_it_with_a_warning() {
    let p = parse("todo ghost: Boo");
    assert_eq!(p.diagram.tasks[0].node.as_deref(), Some("ghost"));
    assert!(
        p.diagnostics
            .iter()
            .any(|d| d.line == 1 && d.severity == Severity::Warning)
    );
}

#[test]
fn duplicate_task_ids_are_replaced() {
    let p = parse("todo: A [id: x]\ntodo: B [id: x]");
    assert_eq!(p.diagram.tasks[1].id, "t2");
    assert_eq!(p.diagnostics.len(), 1);
}

#[test]
fn comments_need_whitespace_before_the_slashes() {
    let d = parse("a = See https://example.com/x // trailing\n// whole line\nb = \"keep // this\"")
        .diagram;
    assert_eq!(d.nodes[0].label, "See https://example.com/x");
    assert_eq!(d.nodes[1].label, "keep // this");
}

#[test]
fn quoted_values_unescape() {
    let d = parse(r#"a = "He said \"hi\"\nthen left" [notes: x]"#).diagram;
    assert_eq!(d.nodes[0].label, "He said \"hi\"\nthen left");
    let d = parse(r#"todo: T [notes: "line\\one\ntwo"]"#).diagram;
    assert_eq!(d.tasks[0].notes, "line\\one\ntwo");
}

#[test]
fn tags_split_on_spaces_and_keep_quoted_phrases() {
    let d = parse(r#"todo: T [tags: a "b c" d]"#).diagram;
    assert_eq!(d.tasks[0].tags, vec!["a", "b c", "d"]);
}

#[test]
fn family_line_and_directions() {
    for (line, kind, dir) in [
        ("flowchart", DiagramKind::Flowchart, Direction::Down),
        ("userflow right", DiagramKind::UserFlow, Direction::Right),
        ("taskflow LR", DiagramKind::TaskFlow, Direction::Right),
        ("flowchart TD", DiagramKind::Flowchart, Direction::Down),
        ("flowchart RL", DiagramKind::Flowchart, Direction::Left),
        ("userflow BT", DiagramKind::UserFlow, Direction::Up),
        ("taskflow left", DiagramKind::TaskFlow, Direction::Left),
    ] {
        let d = parse(line).diagram;
        assert_eq!((d.kind, d.direction), (kind, dir), "{line}");
    }
    assert_eq!(warnings("flowchart sideways").len(), 1);
}

#[test]
fn brand_preset_then_overrides() {
    let d = parse("brand: paper\nbrand.primary: #f80\nbrand.shadow: on\nbrand.name: Mine").diagram;
    let mut want = BrandKit::paper();
    want.palette.primary = Rgb(0xff, 0x88, 0x00);
    want.shadow = true;
    want.name = "Mine".into();
    assert_eq!(d.brand, want);
    assert_eq!(parse("").diagram.brand, BrandKit::default());
}

#[test]
fn bad_brand_values_warn_and_keep_the_default() {
    let w = warnings(
        "brand: neon\nbrand.primary: blue\nbrand.radius: lots\nbrand.radius: 1e40\nbrand.shadow: maybe\nbrand.wat: 1\nbrand.routing: wavy",
    );
    assert_eq!(w.len(), 7, "{w:?}");
    assert!(w[0].starts_with("1:") && w[6].starts_with("7:"));
    assert_eq!(
        parse("brand.primary: blue").diagram.brand,
        BrandKit::default()
    );
}

#[test]
fn non_ascii_colours_do_not_panic() {
    let p = parse("brand.primary: #\u{e9}\u{e9}\u{e9}\nbrand.text: \u{1f642}\u{1f642}");
    assert_eq!(p.diagnostics.len(), 2);
}

#[test]
fn unknown_attributes_and_values_warn_with_their_line() {
    let w = warnings(
        "a\nb [shape: blob, wat, role: neon, size: big]\nc -> d [line: wavy, arrow: up, color: red]\ntodo: t [priority: urgent, due: soon, mood: x]",
    );
    assert_eq!(w.len(), 10, "{w:?}");
    assert!(w.iter().all(|m| !m.starts_with("1:")));
    assert!(w[0].starts_with("2:") && w[4].starts_with("3:"));
}

#[test]
fn garbage_lines_still_yield_the_rest() {
    let p = parse("a -> b\n]]] ??? {{{ \n-> ->\n: :\nc -> d");
    assert_eq!(p.diagram.edges.len(), 2);
    assert!(!p.diagnostics.is_empty());
}

#[test]
fn unterminated_string_warns() {
    let p = parse("a = \"open");
    assert_eq!(p.diagram.nodes[0].label, "open");
    assert_eq!(p.diagnostics.len(), 1);
}

#[test]
fn newer_versions_warn_but_parse() {
    let p = parse("%% rovelo 7\na -> b");
    assert_eq!(p.diagram.edges.len(), 1);
    assert_eq!(p.diagnostics.len(), 1);
    assert!(parse("%% some other comment\na").diagnostics.is_empty());
}

#[test]
fn crlf_and_bom_are_fine() {
    let d = parse("\u{feff}title: T\r\na -> b\r\n").diagram;
    assert_eq!((d.title.as_str(), d.edges.len()), ("T", 1));
}

#[test]
fn serialize_is_canonical_and_readable() {
    let src = "taskflow right\ntitle: Demo\nSign up -> Verify email: Email link [line: dashed]\ngroup Auth { sign-up }\ntodo sign-up: Write copy [done, priority: high, tags: a b, due: 2026-11-10]\n";
    let text = serialize(&parse(src).diagram);
    let want = "\
%% rovelo 1
taskflow right
title: Demo
brand: rovelo

sign-up = Sign up
verify-email = Verify email

sign-up -> verify-email: Email link [line: dashed]

group Auth {
  sign-up
}

todo sign-up: Write copy [done, due: 2026-11-10, priority: high, tags: a b]
";
    assert_eq!(text, want);
    assert_eq!(serialize(&parse(&text).diagram), text);
}

#[test]
fn serializer_quotes_what_would_not_lex_back() {
    let mut d = Diagram::default();
    d.nodes.push(Node::new("a", "Wait -> then go"));
    d.nodes.push(Node::new("b", "group chat"));
    d.nodes.push(Node::new("c", "two\nlines"));
    d.nodes.push(Node::new("d", "plain label"));
    let t = serialize(&d);
    assert!(t.contains("a = \"Wait -> then go\""));
    assert!(t.contains("b = \"group chat\""));
    assert!(t.contains("c = \"two\\nlines\""));
    assert!(t.contains("d = plain label\n"));
}

#[test]
fn brand_serializes_only_what_differs_from_its_preset() {
    let mut d = Diagram {
        brand: BrandKit::midnight(),
        ..Diagram::default()
    };
    assert!(serialize(&d).contains("brand: midnight\n\n") || !serialize(&d).contains("brand."));
    d.brand.palette.accent = Rgb(1, 2, 3);
    d.brand.shadow = true;
    let t = serialize(&d);
    let brand_lines: Vec<_> = t.lines().filter(|l| l.starts_with("brand")).collect();
    assert_eq!(
        brand_lines,
        vec![
            "brand: midnight",
            "brand.accent: #010203",
            "brand.shadow: on"
        ]
    );
    // A custom name gets the whole kit.
    d.brand.name = "Acme".into();
    let t = serialize(&d);
    assert!(
        t.contains("brand.name: Acme")
            && t.contains("brand.primary:")
            && t.contains("brand.grid: off")
    );
    assert!(!t.contains("brand: midnight"));
    assert_eq!(parse(&t).diagram.brand, d.brand);
}

#[test]
fn same_text_yields_the_same_diagram_every_time() {
    let src = "a -> b\ngroup G { a }\ntodo a: x";
    assert_eq!(parse(src).diagram, parse(src).diagram);
}

#[test]
fn the_worked_examples_in_grammar_md_are_true() {
    let md = include_str!("../GRAMMAR.md");
    let blocks: Vec<&str> = md
        .split("```")
        .skip(1)
        .step_by(2)
        .map(|b| b.split_once('\n').map_or("", |x| x.1))
        .collect();
    let find = |needle: &str| {
        blocks
            .iter()
            .position(|b| b.contains(needle))
            .unwrap_or_else(|| panic!("no block with {needle}"))
    };
    let user = blocks[find("userflow right")];
    assert_eq!(serialize(&parse(user).diagram), user);
    let hand = find("Write copy -> Review");
    assert_eq!(serialize(&parse(blocks[hand]).diagram), blocks[hand + 1]);
    let mm = find("classDef hot");
    let got = rovelo_dsl::from_mermaid(blocks[mm]);
    assert_eq!(serialize(&got.diagram), blocks[mm + 1]);
    assert_eq!(got.diagnostics.len(), 1);
}

#[test]
fn node_style_round_trips() {
    use rovelo_core::brand::NodeStyle;
    let p = parse("brand: slate\nbrand.nodes: card\na = A");
    assert_eq!(p.diagram.brand.node_style, NodeStyle::Card);
    let text = serialize(&p.diagram);
    assert!(text.contains("brand.nodes: card"), "{text}");
    assert_eq!(parse(&text).diagram.brand.node_style, NodeStyle::Card);
    let bad = parse("brand.nodes: blob");
    assert!(
        bad.diagnostics
            .iter()
            .any(|d| d.message.contains("card or shape"))
    );
}
