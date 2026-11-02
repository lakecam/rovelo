//! Round-trip laws, checked over generated diagrams and garbled text.
use rovelo_core::brand::{BrandKit, EdgeRouting, NodeFill, Rgb, Role};
use rovelo_core::model::*;
use rovelo_dsl::{from_mermaid, parse, serialize};

/// `ROVELO_FUZZ=20` multiplies every loop count, for a longer soak run.
fn iters(base: usize) -> usize {
    base * std::env::var("ROVELO_FUZZ")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1)
}

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
    fn chance(&mut self, pct: usize) -> bool {
        self.below(100) < pct
    }
    fn pick<'a, T>(&mut self, xs: &'a [T]) -> &'a T {
        &xs[self.below(xs.len())]
    }
    fn word(&mut self, xs: &[&'static str]) -> &'static str {
        xs[self.below(xs.len())]
    }
}

const PIECES: &[&str] = &[
    "Sign up",
    "a",
    "B",
    "x y",
    "  lead",
    "trail  ",
    "a->b",
    "<->",
    "--",
    "x: y",
    "[1]",
    "{c}",
    "a;b",
    "a,b",
    "k=v",
    "say \"hi\"",
    "back\\slash",
    "tab\there",
    "line\nbreak",
    "\r",
    "//",
    "http://x.y",
    "group",
    "todo list",
    "Title",
    "brand",
    "flowchart",
    "%% c",
    "\u{1f642} emoji",
    "\u{65e5}\u{672c}\u{8a9e}",
    "\u{e9}",
    "\u{a0}nbsp",
    "\u{2028}",
    "\u{7}",
    "-",
    "<",
    "> x",
    "",
    "'quoted'",
    "#hash",
    "a  b",
    "?",
    "|pipe|",
    "e1",
    "t1",
    "\\",
];

fn text(r: &mut Rng) -> String {
    let n = 1 + r.below(3);
    let mut s = String::new();
    for i in 0..n {
        if i > 0 {
            s.push_str(r.word(&[" ", "", "-", "  "]));
        }
        s.push_str(r.word(PIECES));
    }
    s
}

fn word(r: &mut Rng) -> String {
    r.word(&[
        "alpha", "beta", "start", "end", "sign-up", "todo", "group", "e1", "t1", "A", "x y", "a:b",
        "\u{65e5}", "q\"", "a--b",
    ])
    .to_string()
}

fn color(r: &mut Rng) -> Rgb {
    Rgb(r.below(256) as u8, r.below(256) as u8, r.below(256) as u8)
}

fn num(r: &mut Rng) -> f32 {
    r.below(400) as f32 / 8.0
}

fn brand(r: &mut Rng) -> BrandKit {
    let mut k = match r.below(4) {
        0 => BrandKit::slate(),
        1 => BrandKit::paper(),
        2 => BrandKit::midnight(),
        _ => BrandKit::slate(),
    };
    if r.chance(40) {
        k.name = match r.below(3) {
            0 => k.name.to_uppercase(),
            1 => text(r),
            _ => format!("{} ", k.name),
        };
    }
    for role in Role::ALL {
        if r.chance(20) {
            k.palette.set(role, color(r));
        }
    }
    if r.chance(20) {
        k.background = color(r);
        k.surface = color(r);
        k.text = color(r);
        k.muted_text = color(r);
    }
    if r.chance(20) {
        k.font_family = text(r);
        k.heading_family = text(r);
    }
    if r.chance(30) {
        k.corner_radius = num(r);
        k.stroke_width = num(r);
        k.font_size = num(r) + 1.0;
    }
    if r.chance(20) {
        k.edge_routing = *r.pick(&[
            EdgeRouting::Straight,
            EdgeRouting::Orthogonal,
            EdgeRouting::Curved,
        ]);
        k.node_fill = *r.pick(&[NodeFill::Solid, NodeFill::Soft, NodeFill::Outline]);
        k.shadow = r.chance(50);
        k.show_grid = r.chance(50);
    }
    if r.chance(20) {
        k.logo_svg = Some(match r.below(3) {
            0 => "none".to_string(),
            1 => "<svg xmlns=\"http://www.w3.org/2000/svg\">\n<path d=\"M0 0\"/></svg>".to_string(),
            _ => text(r),
        });
    }
    k
}

fn diagram(r: &mut Rng) -> Diagram {
    let mut d = Diagram {
        title: text(r),
        ..Diagram::default()
    };
    d.kind = *r.pick(&DiagramKind::ALL);
    d.direction = *r.pick(&[
        Direction::Down,
        Direction::Right,
        Direction::Left,
        Direction::Up,
    ]);
    d.brand = brand(r);
    let mut gids: Vec<String> = vec![];
    for i in 0..r.below(4) {
        let label = if r.chance(50) {
            "Auth flow".to_string()
        } else {
            text(r)
        };
        let mut id = if r.chance(50) {
            format!("g{i}")
        } else {
            word(r)
        };
        if label == "Auth flow" && r.chance(50) {
            id = "auth-flow".into();
        }
        if gids.contains(&id) {
            id = format!("g{i}-x");
        }
        gids.push(id.clone());
        d.groups.push(Group {
            id,
            label,
            role: *r.pick(&Role::ALL),
        });
    }
    let mut nids: Vec<String> = vec![];
    for i in 0..r.below(9) {
        let mut id = if r.chance(60) {
            format!("n{i}")
        } else {
            word(r)
        };
        if nids.contains(&id) {
            id = format!("n{i}-x");
        }
        nids.push(id.clone());
        let mut n = Node::new(id, text(r));
        n.shape = *r.pick(&Shape::ALL);
        n.role = *r.pick(&Role::ALL);
        if r.chance(25) {
            n.icon = Some(text(r));
        }
        if r.chance(15) {
            n.size = Some(Size {
                w: num(r) + 1.0,
                h: num(r) + 1.0,
            });
        }
        if r.chance(20) {
            n.pos = Some(Point::new(num(r), num(r)));
        }
        if !gids.is_empty() && r.chance(50) {
            n.group = Some(r.pick(&gids).clone());
        }
        d.nodes.push(n);
    }
    if !nids.is_empty() {
        for i in 0..r.below(11) {
            let mut e = Edge::new(
                format!("e{}", i + 1),
                r.pick(&nids).clone(),
                r.pick(&nids).clone(),
            );
            if r.chance(50) {
                e.label = text(r);
            }
            e.line = *r.pick(&[LineStyle::Solid, LineStyle::Dashed, LineStyle::Dotted]);
            e.arrow = *r.pick(&[Arrow::End, Arrow::Both, Arrow::None]);
            if r.chance(30) {
                e.role = Some(*r.pick(&Role::ALL));
            }
            d.edges.push(e);
        }
    }
    let mut tids: Vec<String> = vec![];
    for i in 0..r.below(6) {
        let mut id = if r.chance(70) {
            format!("t{}", i + 1)
        } else {
            format!("t{}", r.below(9) + 1)
        };
        if tids.contains(&id) {
            id = format!("x{i}");
        }
        tids.push(id.clone());
        let mut t = Task::new(id, text(r));
        t.done = r.chance(40);
        if !nids.is_empty() && r.chance(60) {
            t.node = Some(r.pick(&nids).clone());
        }
        if r.chance(50) {
            t.due = Some(if r.chance(70) {
                "2026-11-10".to_string()
            } else {
                text(r)
            });
        }
        t.priority = *r.pick(&[Priority::Low, Priority::Normal, Priority::High]);
        for _ in 0..r.below(4) {
            t.tags.push(text(r));
        }
        if r.chance(50) {
            t.notes = text(r);
        }
        d.tasks.push(t);
    }
    d
}

/// Positions are never serialized and edge ids are renumbered; everything else must survive.
fn normalized(d: &Diagram) -> Diagram {
    let mut d = d.clone();
    for n in &mut d.nodes {
        n.pos = None;
    }
    for (i, e) in d.edges.iter_mut().enumerate() {
        e.id = format!("e{}", i + 1);
    }
    d
}

fn assert_round_trip(d: &Diagram, what: &str) {
    let text = serialize(d);
    let back = parse(&text).diagram;
    assert_eq!(
        normalized(d),
        normalized(&back),
        "{what}\n--- text ---\n{text}"
    );
    assert_eq!(serialize(&back), text, "{what}: not a fixpoint\n{text}");
}

#[test]
fn law2_generated_diagrams_survive_a_round_trip() {
    let mut r = Rng(0x9e37_79b9_7f4a_7c15);
    for i in 0..iters(1500) {
        let d = diagram(&mut r);
        assert_round_trip(&d, &format!("generated diagram #{i}"));
    }
}

#[test]
fn law2_nasty_ids_are_quoted_not_lost() {
    let mut d = Diagram::default();
    for (i, id) in [
        "todo",
        "group",
        "my id",
        "a:b",
        "",
        "x\"y",
        "%%",
        "a--b",
        "->",
        "-",
        "\u{65e5}",
        "brand.primary",
        "flowchart",
        "e1",
    ]
    .into_iter()
    .enumerate()
    {
        d.nodes.push(Node::new(id, format!("label {i}")));
    }
    d.edges.push(Edge::new("e1", "todo", "my id"));
    d.edges.push(Edge::new("e2", "", "a:b"));
    d.groups.push(Group {
        id: "group".into(),
        label: "Group".into(),
        role: Role::Neutral,
    });
    d.nodes[2].group = Some("group".into());
    let mut t = Task::new("t1", "x");
    t.node = Some("my id".into());
    d.tasks.push(t);
    assert_round_trip(&d, "nasty ids");
}

#[test]
fn law2_empty_and_default_diagrams() {
    assert_round_trip(&Diagram::default(), "default");
    let d = Diagram {
        title: String::new(),
        ..Diagram::default()
    };
    assert_round_trip(&d, "empty title");
}

fn garble(r: &mut Rng) -> String {
    const BITS: &[&str] = &[
        "->",
        "<->",
        "--",
        "-->",
        "[",
        "]",
        "{",
        "}",
        ";",
        ",",
        ":",
        "=",
        "\"",
        "\\",
        "//",
        "%%",
        "group",
        "todo",
        "title:",
        "brand.primary:",
        "brand:",
        "taskflow",
        "subgraph",
        "end",
        "A",
        "B",
        "\n",
        "\n",
        "\n",
        " ",
        " ",
        "x",
        "[shape: pill]",
        "[done, due: 2026-11-10]",
        "\u{e9}",
        "\u{1f642}",
        "\u{0}",
        "\r\n",
        "#fff",
        "|",
        "(",
        ")",
        "((",
        "))",
        ">",
        "<",
        "&",
        "~~~",
        "==>",
        "-.->",
        "--o",
        "group A {",
        "}",
        "todo a: b",
        "a = B",
        "[tags: x y, notes: \"q\"]",
        "brand.radius: 1e40",
        "flowchart TD",
        "graph LR",
        "id = \"",
        "\u{2028}",
    ];
    let n = 1 + r.below(40);
    let mut s = String::new();
    for _ in 0..n {
        if r.chance(85) {
            s.push_str(r.word(BITS));
        } else {
            s.push(char::from_u32(r.below(0x3000) as u32).unwrap_or('?'));
        }
    }
    s
}

#[test]
fn garbled_text_never_panics_and_keeps_the_laws() {
    let mut r = Rng(0x1234_5678_9abc_def1);
    for i in 0..iters(800) {
        let x = garble(&mut r);
        let p = parse(&x);
        let _ = from_mermaid(&x);
        let once = serialize(&p.diagram);
        let twice = serialize(&parse(&once).diagram);
        assert_eq!(once, twice, "fixpoint failed for input #{i}: {x:?}");
        assert_round_trip(&p.diagram, &format!("input #{i}: {x:?}"));
    }
}

#[test]
fn garbled_mermaid_never_panics_and_serializes_stably() {
    let mut r = Rng(0xdead_beef_cafe_f00d);
    for i in 0..iters(400) {
        let x = format!("graph TD\n{}", garble(&mut r));
        let m = from_mermaid(&x);
        assert_round_trip(&m.diagram, &format!("mermaid input #{i}: {x:?}"));
    }
}

#[test]
fn random_unicode_soup_never_panics() {
    let mut r = Rng(42);
    for _ in 0..iters(300) {
        let n = r.below(80);
        let s: String = (0..n)
            .map(|_| char::from_u32(r.below(0x11_0000) as u32).unwrap_or('x'))
            .collect();
        let p = parse(&s);
        let _ = from_mermaid(&s);
        let _ = serialize(&p.diagram);
    }
}

#[test]
fn hard_line_limit_gives_one_error_and_stops() {
    let mut src = String::new();
    for i in 0..20_050 {
        src.push_str(&format!("n{i} = Node {i}\n"));
    }
    let p = parse(&src);
    let errors: Vec<_> = p
        .diagnostics
        .iter()
        .filter(|d| d.severity == rovelo_dsl::Severity::Error)
        .collect();
    assert_eq!(errors.len(), 1);
    assert_eq!(p.diagram.nodes.len(), 20_000);
}

#[test]
fn pathological_inputs_finish_quickly() {
    let t = std::time::Instant::now();
    parse(&"a -> ".repeat(50_000));
    parse(&"[".repeat(100_000));
    parse(&"\"".repeat(100_000));
    parse(&"group g {\n".repeat(5_000));
    parse(&"{".repeat(100_000));
    parse(&format!(
        "{} -> {}",
        (0..300)
            .map(|i| format!("a{i}"))
            .collect::<Vec<_>>()
            .join(", "),
        (0..300)
            .map(|i| format!("b{i}"))
            .collect::<Vec<_>>()
            .join(", ")
    ));
    from_mermaid(&format!("graph TD\n{}", "A --> B & ".repeat(20_000)));
    from_mermaid(&format!("graph TD\n{}", "A[".repeat(20_000)));
    assert!(t.elapsed().as_secs() < 20, "took {:?}", t.elapsed());
}
