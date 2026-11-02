//! Canonical rovelo text for a diagram.
use std::fmt::Write;

use rovelo_core::brand::{BrandKit, EdgeRouting, NodeFill, NodeStyle, Role};
use rovelo_core::model::{Arrow, Diagram, LineStyle, Priority};

use crate::lexer::{ident, line_value, name, needs_quote, quote};
use crate::parse::slug;

fn attr_block(items: Vec<String>) -> String {
    if items.is_empty() {
        String::new()
    } else {
        format!(" [{}]", items.join(", "))
    }
}

fn hex_line(o: &mut String, key: &str, c: rovelo_core::brand::Rgb) {
    let _ = writeln!(o, "brand.{key}: {}", c.hex());
}

fn brand(o: &mut String, k: &BrandKit) {
    let preset = BrandKit::presets()
        .into_iter()
        .find(|p| p.name.eq_ignore_ascii_case(&k.name));
    let full = preset.is_none();
    let base = match preset {
        Some(p) => {
            let _ = writeln!(o, "brand: {}", p.name.to_lowercase());
            p
        }
        None => BrandKit::default(),
    };
    // A key is written when it differs from the base, or always for a custom kit.
    let diff = |same: bool| full || !same;
    if diff(base.name == k.name) {
        let _ = writeln!(o, "brand.name: {}", line_value(&k.name));
    }
    for r in Role::ALL {
        if diff(base.palette.get(r) == k.palette.get(r)) {
            hex_line(o, r.name(), k.palette.get(r));
        }
    }
    for (key, mine, theirs) in [
        ("background", k.background, base.background),
        ("surface", k.surface, base.surface),
        ("text", k.text, base.text),
        ("muted_text", k.muted_text, base.muted_text),
    ] {
        if diff(mine == theirs) {
            hex_line(o, key, mine);
        }
    }
    if diff(base.font_family == k.font_family) {
        let _ = writeln!(o, "brand.font: {}", line_value(&k.font_family));
    }
    if diff(base.heading_family == k.heading_family) {
        let _ = writeln!(o, "brand.heading_font: {}", line_value(&k.heading_family));
    }
    if diff(base.corner_radius == k.corner_radius) {
        let _ = writeln!(o, "brand.radius: {}", k.corner_radius);
    }
    if diff(base.stroke_width == k.stroke_width) {
        let _ = writeln!(o, "brand.stroke: {}", k.stroke_width);
    }
    if diff(base.font_size == k.font_size) {
        let _ = writeln!(o, "brand.size: {}", k.font_size);
    }
    if diff(base.edge_routing == k.edge_routing) {
        let w = match k.edge_routing {
            EdgeRouting::Straight => "straight",
            EdgeRouting::Orthogonal => "orthogonal",
            EdgeRouting::Curved => "curved",
        };
        let _ = writeln!(o, "brand.routing: {w}");
    }
    if diff(base.node_fill == k.node_fill) {
        let w = match k.node_fill {
            NodeFill::Solid => "solid",
            NodeFill::Soft => "soft",
            NodeFill::Outline => "outline",
        };
        let _ = writeln!(o, "brand.fill: {w}");
    }
    if diff(base.node_style == k.node_style) {
        let w = match k.node_style {
            NodeStyle::Card => "card",
            NodeStyle::Shape => "shape",
        };
        let _ = writeln!(o, "brand.nodes: {w}");
    }
    let onoff = |b: bool| if b { "on" } else { "off" };
    if diff(base.shadow == k.shadow) {
        let _ = writeln!(o, "brand.shadow: {}", onoff(k.shadow));
    }
    if diff(base.show_grid == k.show_grid) {
        let _ = writeln!(o, "brand.grid: {}", onoff(k.show_grid));
    }
    // The logo is always quoted so a logo that reads "none" stays a logo.
    match (&k.logo_svg, &base.logo_svg) {
        (Some(s), _) => {
            let _ = writeln!(o, "brand.logo: {}", quote(s));
        }
        (None, Some(_)) => o.push_str("brand.logo: none\n"),
        (None, None) => {}
    }
}

pub fn serialize(d: &Diagram) -> String {
    let mut o = String::from("%% rovelo 1\n");
    let _ = writeln!(o, "{} {}", d.kind.name(), d.direction.name());
    let _ = writeln!(o, "title: {}", line_value(&d.title));
    brand(&mut o, &d.brand);

    let mut sections: Vec<String> = Vec::new();

    let mut s = String::new();
    for n in &d.nodes {
        let mut a = Vec::new();
        if n.shape != rovelo_core::model::Shape::Rect {
            a.push(format!("shape: {}", n.shape.name()));
        }
        if n.role != Role::Primary {
            a.push(format!("role: {}", n.role.name()));
        }
        if let Some(i) = &n.icon {
            a.push(format!("icon: {}", name(i)));
        }
        if let Some(z) = n.size {
            a.push(format!("size: {}x{}", z.w, z.h));
        }
        let _ = writeln!(s, "{} = {}{}", ident(&n.id), name(&n.label), attr_block(a));
    }
    sections.push(s);

    let mut s = String::new();
    for e in &d.edges {
        let arrow = match e.arrow {
            Arrow::End => "->",
            Arrow::Both => "<->",
            Arrow::None => "--",
        };
        let mut a = Vec::new();
        if e.line != LineStyle::Solid {
            a.push(format!(
                "line: {}",
                if e.line == LineStyle::Dashed {
                    "dashed"
                } else {
                    "dotted"
                }
            ));
        }
        if let Some(r) = e.role {
            a.push(format!("role: {}", r.name()));
        }
        let label = if e.label.is_empty() {
            String::new()
        } else {
            format!(": {}", name(&e.label))
        };
        let _ = writeln!(
            s,
            "{} {arrow} {}{label}{}",
            ident(&e.from),
            ident(&e.to),
            attr_block(a)
        );
    }
    sections.push(s);

    let mut s = String::new();
    for g in &d.groups {
        let head = if slug(&g.label) == g.id && !needs_quote(&g.label) {
            g.label.clone()
        } else {
            format!("{} = {}", ident(&g.id), name(&g.label))
        };
        let a = if g.role != Role::Neutral {
            attr_block(vec![format!("role: {}", g.role.name())])
        } else {
            String::new()
        };
        let members: Vec<&str> = d
            .nodes
            .iter()
            .filter(|n| n.group.as_deref() == Some(&g.id))
            .map(|n| n.id.as_str())
            .collect();
        if members.is_empty() {
            let _ = writeln!(s, "group {head}{a} {{}}");
        } else {
            let _ = writeln!(s, "group {head}{a} {{");
            for m in members {
                let _ = writeln!(s, "  {}", ident(m));
            }
            s.push_str("}\n");
        }
    }
    sections.push(s);

    let mut s = String::new();
    for (i, t) in d.tasks.iter().enumerate() {
        let mut a = Vec::new();
        if t.id != format!("t{}", i + 1) {
            a.push(format!("id: {}", ident(&t.id)));
        }
        if t.done {
            a.push("done".to_string());
        }
        if let Some(due) = &t.due {
            a.push(format!("due: {}", name(due)));
        }
        if t.priority != Priority::Normal {
            a.push(format!("priority: {}", t.priority.name()));
        }
        if !t.tags.is_empty() {
            a.push(format!(
                "tags: {}",
                t.tags
                    .iter()
                    .map(|g| ident(g))
                    .collect::<Vec<_>>()
                    .join(" ")
            ));
        }
        if !t.notes.is_empty() {
            a.push(format!("notes: {}", quote(&t.notes)));
        }
        let target = t
            .node
            .as_deref()
            .map(|n| format!(" {}", ident(n)))
            .unwrap_or_default();
        let _ = writeln!(s, "todo{target}: {}{}", name(&t.title), attr_block(a));
    }
    sections.push(s);

    for s in sections.into_iter().filter(|s| !s.is_empty()) {
        o.push('\n');
        o.push_str(&s);
    }
    o
}
