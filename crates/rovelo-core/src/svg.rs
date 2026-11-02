//! SVG export, branded from the diagram's `BrandKit`.
use crate::brand::Role;
use crate::card;
use crate::geometry::{
    EdgePath, diagram_bounds, edge_path, group_rect, label_lines, node_rect, text_width,
};
use crate::layout::layout_unplaced;
use crate::model::{Arrow, Diagram, LineStyle, Node, Point, Rect};
use std::collections::HashMap;
use std::fmt::Write;

#[path = "svg_shapes.rs"]
mod shapes;
pub use shapes::{ICON_NAMES, is_known_icon};
use shapes::{Paint, icon, num, outline, text_offset};

const LINE_H: f32 = 1.35;
const MARGIN: f32 = 40.0;
const HEADER: f32 = 56.0;

/// Render the whole diagram as a standalone SVG document. Unplaced nodes are laid out first
/// on a clone; the input is not modified.
pub fn to_svg(d: &Diagram) -> String {
    let laid;
    let d = if d.nodes.iter().any(|n| n.pos.is_none()) {
        let mut c = d.clone();
        layout_unplaced(&mut c);
        laid = c;
        &laid
    } else {
        d
    };
    let b = &d.brand;
    let fs = b.font_size.clamp(6.0, 96.0);
    let cards = card::uses_cards(b);
    let dark = card::is_dark(b);
    let ink = card::ink(b, dark);
    let label_fs = if cards { card::LABEL_SIZE } else { fs * 0.86 };
    let paths: Vec<EdgePath> = d.edges.iter().map(|e| edge_path(d, e)).collect();
    let pills: Vec<Option<Rect>> = d
        .edges
        .iter()
        .zip(&paths)
        .map(|(e, p)| crate::geometry::edge_label_rect(d, e, p))
        .collect();

    let mut content = diagram_bounds(d, 0.0);
    for q in paths.iter().flat_map(|p| &p.points) {
        content = content.union(&Rect {
            x: q.x,
            y: q.y,
            w: 0.0,
            h: 0.0,
        });
    }
    for r in pills.iter().flatten() {
        content = content.union(r);
    }

    let logo = b.logo_svg.as_deref().and_then(parse_logo);
    let title = d.title.trim();
    let title_fs = (fs * 1.55).max(18.0);
    let header = if title.is_empty() && logo.is_none() {
        0.0
    } else {
        HEADER
    };
    let (logo_w, logo_h) = logo.as_ref().map_or((0.0, 0.0), |l| {
        let w = (32.0 * l.aspect).min(140.0);
        (w, w / l.aspect)
    });
    let mut view = Rect {
        x: content.x - MARGIN,
        y: content.y - MARGIN - header,
        w: content.w + 2.0 * MARGIN,
        h: content.h + 2.0 * MARGIN + header,
    };
    let needed =
        48.0 + text_width(title, title_fs) + if logo.is_some() { logo_w + 24.0 } else { 0.0 };
    view.w = view.w.max(needed);

    let family = escape(&font_stack(&b.font_family));
    let heading = escape(&font_stack(&b.heading_family));
    let mut o = String::with_capacity(4096 + d.nodes.len() * 600);
    let _ = write!(
        o,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{} {} {} {}" width="{}" height="{}" font-family="{family}" font-size="{}">"#,
        num(view.x),
        num(view.y),
        num(view.w),
        num(view.h),
        view.w.ceil(),
        view.h.ceil(),
        num(fs),
    );
    if !title.is_empty() {
        let _ = write!(o, "<title>{}</title>", escape(title));
    }
    o.push_str("<defs>");
    if b.shadow {
        o.push_str(r##"<filter id="fd-shadow" x="-20%" y="-20%" width="140%" height="150%"><feDropShadow dx="0" dy="2" stdDeviation="3" flood-color="#000000" flood-opacity="0.2"/></filter>"##);
    }
    if b.show_grid {
        let _ = write!(
            o,
            r#"<pattern id="fd-grid" width="24" height="24" patternUnits="userSpaceOnUse"><circle cx="1" cy="1" r="1" fill="{}" fill-opacity="0.35"/></pattern>"#,
            b.muted_text.hex()
        );
    }
    if !d.groups.is_empty() {
        // Glass stand-in: a white wash from the top to 40 %, and a hairline highlight on the top edge.
        let _ = write!(
            o,
            r##"<linearGradient id="fd-glass" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#ffffff" stop-opacity="{}"/><stop offset="{}" stop-color="#ffffff" stop-opacity="0"/></linearGradient><linearGradient id="fd-rim" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#ffffff" stop-opacity="{}"/><stop offset="0.06" stop-color="#ffffff" stop-opacity="0"/></linearGradient>"##,
            num(card::GLASS_TOP),
            num(card::GLASS_STOP),
            num(card::HIGHLIGHT)
        );
    }
    if cards && !dark {
        o.push_str(r##"<filter id="fd-card" x="-20%" y="-30%" width="140%" height="180%"><feDropShadow dx="0" dy="2" stdDeviation="3" flood-color="#18181b" flood-opacity="0.04"/><feDropShadow dx="0" dy="8" stdDeviation="12" flood-color="#18181b" flood-opacity="0.08"/></filter>"##);
    }
    o.push_str("</defs>");
    let _ = write!(
        o,
        r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}"/>"#,
        num(view.x),
        num(view.y),
        num(view.w),
        num(view.h),
        b.background.hex()
    );
    if b.show_grid {
        let _ = write!(
            o,
            r##"<rect x="{}" y="{}" width="{}" height="{}" fill="url(#fd-grid)"/>"##,
            num(view.x),
            num(view.y),
            num(view.w),
            num(view.h)
        );
    }

    if !title.is_empty() {
        let _ = write!(
            o,
            r#"<text x="{}" y="{}" font-family="{heading}" font-size="{}" font-weight="700" fill="{}">{}</text>"#,
            num(view.x + 24.0),
            num(view.y + 24.0 + title_fs * 0.8),
            num(title_fs),
            b.text.hex(),
            escape(title)
        );
    }
    if let Some(l) = &logo {
        let _ = write!(
            o,
            r#"<svg x="{}" y="{}" width="{}" height="{}" viewBox="{}" preserveAspectRatio="xMaxYMid meet"{}>{}</svg>"#,
            num(view.x + view.w - 24.0 - logo_w),
            num(view.y + 12.0 + (32.0 - logo_h) / 2.0),
            num(logo_w),
            num(logo_h),
            escape(&l.view_box),
            l.namespaces,
            l.inner
        );
    }

    for g in &d.groups {
        let Some(r) = group_rect(d, &g.id) else {
            continue;
        };
        let fill = b.palette.get(g.role);
        let edge = card::group_edge(b, g.role, dark);
        let title_ink = card::vibrant(b, g.role, dark);
        let (x, y, w, h) = (num(r.x), num(r.y), num(r.w), num(r.h));
        let rx = num(card::GROUP_RADIUS);
        let _ = write!(
            o,
            r##"<g data-group="{gid}"><rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{rx}" fill="{f}" fill-opacity="{fa}"/><rect x="{x}" y="{y}" width="{w}" height="{gh}" rx="{rx}" fill="url(#fd-glass)"/><rect x="{x}" y="{y}" width="{w}" height="{h}" rx="{rx}" fill="none" stroke="{e}" stroke-width="{bw}"/><rect x="{ix}" y="{iy}" width="{iw}" height="{ih}" rx="{irx}" fill="none" stroke="url(#fd-rim)" stroke-width="1"/><circle cx="{dx}" cy="{dy}" r="{dr}" fill="{t}"/><text x="{tx}" y="{ty}" font-size="{fs}" font-weight="600" fill="{t}">{label}</text></g>"##,
            gid = escape(&g.id),
            f = fill.hex(),
            fa = num(card::group_alpha(dark)),
            gh = num(r.h * card::GLASS_STOP),
            e = edge.hex(),
            bw = num(card::GROUP_BORDER),
            ix = num(r.x + 1.5),
            iy = num(r.y + 1.5),
            iw = num((r.w - 3.0).max(0.0)),
            ih = num((r.h - 3.0).max(0.0)),
            irx = num(card::GROUP_RADIUS - 1.5),
            dx = num(r.x + card::GROUP_TITLE_INSET + card::GROUP_DOT / 2.0),
            dy = num(r.y + crate::geometry::GROUP_TITLE / 2.0),
            dr = num(card::GROUP_DOT / 2.0),
            t = title_ink.hex(),
            tx = num(r.x + card::GROUP_TITLE_INSET + card::GROUP_DOT + 8.0),
            ty = num(r.y + crate::geometry::GROUP_TITLE / 2.0 + card::GROUP_TITLE_SIZE * 0.35),
            fs = num(card::GROUP_TITLE_SIZE),
            label = escape(&g.label)
        );
    }

    for (e, p) in d.edges.iter().zip(&paths) {
        if p.points.len() < 2 {
            continue;
        }
        let (col, sw, (len, hw), role_edge) = if cards {
            match e.role {
                Some(role) => (
                    card::vibrant(b, role, dark).hex(),
                    card::EDGE_WIDTH,
                    (card::ARROW, card::ARROW / 2.0),
                    true,
                ),
                None => (
                    ink.edge.hex(),
                    card::EDGE_WIDTH,
                    (card::ARROW, card::ARROW / 2.0),
                    false,
                ),
            }
        } else {
            let sw = b.stroke_width.max(0.5);
            (
                b.palette.get(e.role.unwrap_or(Role::Neutral)).hex(),
                sw,
                (8.0 + sw * 1.5, (8.0 + sw * 1.5) * 0.5),
                false,
            )
        };
        let opacity = if role_edge {
            format!(r#" opacity="{}""#, num(card::ROLE_EDGE_ALPHA))
        } else {
            String::new()
        };
        let mut pts = p.points.clone();
        let at_end = matches!(e.arrow, Arrow::End | Arrow::Both);
        let at_start = matches!(e.arrow, Arrow::Both);
        if at_end {
            trim_end(&mut pts, len * 0.7);
        }
        if at_start {
            pts.reverse();
            trim_end(&mut pts, len * 0.7);
            pts.reverse();
        }
        let dash = if role_edge {
            format!(r#" stroke-dasharray="{d} {d}""#, d = num(card::DASH))
        } else {
            match e.line {
                LineStyle::Solid => String::new(),
                LineStyle::Dashed => {
                    format!(r#" stroke-dasharray="{} {}""#, num(sw * 5.0), num(sw * 3.5))
                }
                LineStyle::Dotted => format!(r#" stroke-dasharray="0.01 {}""#, num(sw * 3.2)),
            }
        };
        let cap = if e.line == LineStyle::Dotted && !role_edge {
            "round"
        } else {
            "butt"
        };
        let d_attr: Vec<String> = pts
            .iter()
            .enumerate()
            .map(|(i, q)| {
                format!(
                    "{}{} {}",
                    if i == 0 { "M" } else { "L" },
                    num(q.x),
                    num(q.y)
                )
            })
            .collect();
        let _ = write!(
            o,
            r#"<g data-edge="{}"><path d="{}" fill="none" stroke="{col}" stroke-width="{}" stroke-linecap="{cap}" stroke-linejoin="round"{dash}{opacity}/>"#,
            escape(&e.id),
            d_attr.join(" "),
            num(sw),
        );
        if at_end {
            let _ = write!(
                o,
                r#"<path d="{}" fill="{col}" stroke="{col}" stroke-width="1" stroke-linejoin="round"{opacity}/>"#,
                arrow_head(p.points[p.points.len() - 1], p.end_dir, len, hw)
            );
        }
        if at_start {
            let _ = write!(
                o,
                r#"<path d="{}" fill="{col}" stroke="{col}" stroke-width="1" stroke-linejoin="round"{opacity}/>"#,
                arrow_head(p.points[0], p.start_dir, len, hw)
            );
        }
        o.push_str("</g>");
    }

    let mut counts: HashMap<&str, (usize, usize)> = HashMap::new();
    for t in &d.tasks {
        if let Some(n) = t.node.as_deref() {
            let c = counts.entry(n).or_insert((0, 0));
            c.1 += 1;
            c.0 += usize::from(t.done);
        }
    }
    let shadow = if b.shadow {
        r##" filter="url(#fd-shadow)""##
    } else {
        ""
    };
    for n in &d.nodes {
        if cards {
            card_markup(&mut o, d, n, &ink, dark);
        } else {
            node_markup(&mut o, d, n, shadow, &ink, dark);
        }
    }

    for ((e, p), pill) in d.edges.iter().zip(&paths).zip(&pills) {
        let Some(r) = pill else { continue };
        let col = b.palette.get(e.role.unwrap_or(Role::Neutral));
        let lines = label_lines(&e.label);
        if cards {
            let _ = write!(
                o,
                r#"<g data-edge-label="{}"><rect x="{}" y="{}" width="{}" height="{}" rx="{}" fill="{}" stroke="{}" stroke-width="1"/>"#,
                escape(&e.id),
                num(r.x),
                num(r.y),
                num(r.w),
                num(r.h),
                num(card::LABEL_RADIUS),
                ink.surface.hex(),
                ink.label_border.hex(),
            );
            text_block(
                &mut o,
                p.label_at.x,
                p.label_at.y,
                &lines,
                label_fs,
                1.25,
                &ink.muted.hex(),
                r#" font-weight="500""#,
            );
            o.push_str("</g>");
            continue;
        }
        let _ = write!(
            o,
            r#"<g data-edge-label="{}"><rect x="{}" y="{}" width="{}" height="{}" rx="{}" fill="{}" stroke="{}" stroke-width="1"/>"#,
            escape(&e.id),
            num(r.x),
            num(r.y),
            num(r.w),
            num(r.h),
            num((r.h / 2.0).min(10.0)),
            b.background.hex(),
            col.mix(b.background, 0.45).hex(),
        );
        text_block(
            &mut o,
            p.label_at.x,
            p.label_at.y,
            &lines,
            label_fs,
            1.25,
            &b.text.hex(),
            "",
        );
        o.push_str("</g>");
    }

    for n in d.nodes.iter().filter(|_| !cards) {
        let Some(&(done, total)) = counts.get(n.id.as_str()) else {
            continue;
        };
        let r = node_rect(d, n);
        let txt = format!("{done}/{total}");
        let bs = 10.5;
        let (w, h) = (txt.chars().count() as f32 * 0.6 * bs + 14.0, 18.0);
        let (fill, ink) = if done == total {
            let g = b.palette.success;
            (g, g.readable_on())
        } else {
            (b.text, b.background)
        };
        let _ = write!(
            o,
            r#"<g data-badge="{}"><rect x="{}" y="{}" width="{}" height="{}" rx="9" fill="{}" stroke="{}" stroke-width="2"/>"#,
            escape(&n.id),
            num(r.x + r.w + 6.0 - w),
            num(r.y - h / 2.0),
            num(w),
            num(h),
            fill.hex(),
            b.background.hex(),
        );
        text_block(
            &mut o,
            r.x + r.w + 6.0 - w / 2.0,
            r.y,
            &[txt.as_str()],
            bs,
            1.0,
            &ink.hex(),
            r#" font-weight="700""#,
        );
        o.push_str("</g>");
    }

    o.push_str("</svg>");
    o
}

/// A card node: surface, hairline border, an icon tile in the role colour, title and subtitle.
fn card_markup(o: &mut String, d: &Diagram, n: &Node, ink: &card::Ink, dark: bool) {
    let b = &d.brand;
    let fs = b.font_size.clamp(6.0, 96.0);
    let r = node_rect(d, n);
    let role = b.palette.get(n.role);
    let glyph = card::vibrant(b, n.role, dark);
    let filter = if dark {
        ""
    } else {
        r##" filter="url(#fd-card)""##
    };
    let _ = write!(
        o,
        r#"<g data-node="{}"><rect x="{}" y="{}" width="{}" height="{}" rx="{}" fill="{}" stroke="{}" stroke-width="1"{filter}/>"#,
        escape(&n.id),
        num(r.x),
        num(r.y),
        num(r.w),
        num(r.h),
        num(card::RADIUS),
        ink.surface.hex(),
        ink.border.hex(),
    );
    let cy = r.y + r.h / 2.0;
    let (tx, ty) = (r.x + card::PAD, cy - card::TILE / 2.0);
    let _ = write!(
        o,
        r#"<rect x="{}" y="{}" width="{t}" height="{t}" rx="{}" fill="{}" fill-opacity="0.16"/>"#,
        num(tx),
        num(ty),
        num(card::TILE_RADIUS),
        role.hex(),
        t = num(card::TILE),
    );
    let gx = tx + (card::TILE - card::GLYPH) / 2.0;
    match card::icon_word(d, n)
        .and_then(|w| icon(&w, gx, cy - card::GLYPH / 2.0, card::GLYPH, &glyph.hex()))
    {
        Some(g) => o.push_str(&g),
        None => {
            let _ = write!(
                o,
                r#"<circle cx="{}" cy="{}" r="3" fill="{}"/>"#,
                num(tx + card::TILE / 2.0),
                num(cy),
                glyph.hex()
            );
        }
    }
    let lines = label_lines(&n.label);
    let sub = card::subtitle(d, n);
    let line_h = card::TITLE_LINE * (fs / card::TITLE_SIZE).max(0.5);
    let content = lines.len() as f32 * line_h + if sub.is_some() { card::SUB_LINE } else { 0.0 };
    let top = r.y + (r.h - content) / 2.0;
    let text_x = tx + card::TILE + card::TILE_GAP;
    if !n.label.trim().is_empty() {
        let _ = write!(
            o,
            r#"<text font-size="{}" font-weight="500" fill="{}">"#,
            num(fs),
            ink.text.hex()
        );
        for (i, l) in lines.iter().enumerate() {
            let y = top + i as f32 * line_h + line_h / 2.0 + fs * 0.35;
            let _ = write!(
                o,
                r#"<tspan x="{}" y="{}">{}</tspan>"#,
                num(text_x),
                num(y),
                escape(l)
            );
        }
        o.push_str("</text>");
    }
    if let Some((text, all_done)) = sub {
        let y = top + lines.len() as f32 * line_h + card::SUB_LINE / 2.0;
        let mut x = text_x;
        if all_done
            && let Some(g) = icon(
                "check",
                x,
                y - card::SUB_SIZE / 2.0,
                card::SUB_SIZE,
                &card::DONE.hex(),
            )
        {
            o.push_str(&g);
            x += card::SUB_SIZE + 4.0;
        }
        let _ = write!(
            o,
            r#"<text x="{}" y="{}" font-size="{}" fill="{}">{}</text>"#,
            num(x),
            num(y + card::SUB_SIZE * 0.35),
            num(card::SUB_SIZE),
            ink.muted.hex(),
            escape(&text)
        );
    }
    o.push_str("</g>");
}

/// The "Shape" node style: the classic outlines, drawn with the card's neutral surface and
/// border, and the kind or own icon (else a dot) in the role's vibrant step before the label.
fn node_markup(
    o: &mut String,
    d: &Diagram,
    n: &Node,
    shadow: &'static str,
    ink: &card::Ink,
    dark: bool,
) {
    let b = &d.brand;
    let fs = b.font_size.clamp(6.0, 96.0);
    let r = node_rect(d, n);
    let paint = Paint {
        fill: ink.surface.hex(),
        stroke: ink.border.hex(),
        stroke_width: 1.0,
        extra: shadow,
    };
    let _ = write!(o, r#"<g data-node="{}">"#, escape(&n.id));
    o.push_str(&outline(n.shape, r, b.corner_radius, &paint));
    let c = r.center();
    let cy = c.y + text_offset(n.shape, r);
    let lines = label_lines(&n.label);
    let text = ink.text.hex();
    let glyph = card::vibrant(b, n.role, dark).hex();
    let isz = (fs * 1.25).round();
    let tw = if n.label.trim().is_empty() {
        0.0
    } else {
        text_width(&n.label, fs)
    };
    let total = isz + 8.0 + tw;
    let ix = c.x - total / 2.0;
    match card::icon_word(d, n).and_then(|w| icon(&w, ix, cy - isz / 2.0, isz, &glyph)) {
        Some(g) => o.push_str(&g),
        None => {
            let _ = write!(
                o,
                r#"<circle cx="{}" cy="{}" r="3" fill="{glyph}"/>"#,
                num(ix + isz / 2.0),
                num(cy)
            );
        }
    }
    let text_x = ix + isz + 8.0 + tw / 2.0;
    if !n.label.trim().is_empty() {
        text_block(
            o,
            text_x,
            cy,
            &lines,
            fs,
            LINE_H,
            &text,
            r#" font-weight="500""#,
        );
    }
    o.push_str("</g>");
}

/// Centred, possibly multi-line text.
#[allow(clippy::too_many_arguments)]
fn text_block(
    o: &mut String,
    cx: f32,
    cy: f32,
    lines: &[&str],
    fs: f32,
    line_h: f32,
    fill: &str,
    extra: &str,
) {
    let lh = fs * line_h;
    let first = cy - (lines.len() as f32 - 1.0) * lh / 2.0 + fs * 0.35;
    let _ = write!(
        o,
        r#"<text text-anchor="middle" font-size="{}" fill="{fill}"{extra}>"#,
        num(fs)
    );
    for (i, l) in lines.iter().enumerate() {
        let _ = write!(
            o,
            r#"<tspan x="{}" y="{}">{}</tspan>"#,
            num(cx),
            num(first + i as f32 * lh),
            escape(l)
        );
    }
    o.push_str("</text>");
}

fn arrow_head(tip: Point, dir: Point, len: f32, hw: f32) -> String {
    let base = Point::new(tip.x - dir.x * len, tip.y - dir.y * len);
    let perp = Point::new(-dir.y, dir.x);
    format!(
        "M{} {} L{} {} L{} {} Z",
        num(tip.x),
        num(tip.y),
        num(base.x + perp.x * hw),
        num(base.y + perp.y * hw),
        num(base.x - perp.x * hw),
        num(base.y - perp.y * hw)
    )
}

/// Shorten a polyline by `dist` from its end (so a line does not poke through an arrowhead).
fn trim_end(pts: &mut Vec<Point>, mut dist: f32) {
    while pts.len() >= 2 {
        let (a, z) = (pts[pts.len() - 2], pts[pts.len() - 1]);
        let seg = (z.x - a.x).hypot(z.y - a.y);
        if seg > dist + 0.01 {
            let t = dist / seg;
            let last = pts.len() - 1;
            pts[last] = Point::new(z.x + (a.x - z.x) * t, z.y + (a.y - z.y) * t);
            return;
        }
        if pts.len() == 2 {
            return;
        }
        dist -= seg;
        pts.pop();
    }
}

/// Escape text for XML content and attribute values; characters XML forbids are dropped.
pub fn escape(s: &str) -> String {
    let mut o = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            '"' => o.push_str("&quot;"),
            '\'' => o.push_str("&apos;"),
            c if valid_char(c) => o.push(c),
            _ => {}
        }
    }
    o
}

fn valid_char(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\r') || (c as u32 >= 0x20 && c != '\u{fffe}' && c != '\u{ffff}')
}

/// Keep only what belongs in a font stack, so a brand cannot break out of the attribute.
fn font_stack(s: &str) -> String {
    let t: String = s
        .chars()
        .filter(|c| c.is_alphanumeric() || matches!(c, ' ' | ',' | '-' | '_' | '.' | '\''))
        .collect();
    let t = t.trim();
    if t.is_empty() {
        "sans-serif".into()
    } else {
        t.into()
    }
}

struct Logo {
    view_box: String,
    namespaces: String,
    inner: String,
    aspect: f32,
}

/// A logo is used only when it is well-formed XML with an `svg` root and nothing active in it.
fn parse_logo(src: &str) -> Option<Logo> {
    let info = parse_xml(src).ok()?;
    if info.risky || info.root != "svg" {
        return None;
    }
    let attr = |k: &str| {
        info.root_attrs
            .iter()
            .find(|(n, _)| n == k)
            .map(|(_, v)| v.as_str())
    };
    let dim = |k: &str| {
        attr(k).and_then(|v| {
            v.trim()
                .trim_end_matches(|c: char| c.is_ascii_alphabetic())
                .parse::<f32>()
                .ok()
        })
    };
    let vb = attr("viewBox").and_then(|v| {
        let nums: Vec<f32> = v
            .split([' ', ',', '\t', '\n', '\r'])
            .filter(|p| !p.is_empty())
            .filter_map(|p| p.parse().ok())
            .collect();
        (nums.len() == 4).then(|| [nums[0], nums[1], nums[2], nums[3]])
    });
    let (w, h) = match vb {
        Some(v) => (v[2], v[3]),
        None => (
            dim("width").unwrap_or(100.0),
            dim("height").unwrap_or(100.0),
        ),
    };
    if !(w.is_finite() && h.is_finite() && w > 0.0 && h > 0.0) {
        return None;
    }
    let view_box = vb.map_or_else(
        || format!("0 0 {} {}", num(w), num(h)),
        |v| format!("{} {} {} {}", num(v[0]), num(v[1]), num(v[2]), num(v[3])),
    );
    let namespaces: String = info
        .root_attrs
        .iter()
        .filter(|(k, _)| k.starts_with("xmlns:"))
        .map(|(k, v)| format!(r#" {k}="{}""#, v.replace('"', "&quot;")))
        .collect();
    let inner = src[info.inner.0..info.inner.1.max(info.inner.0)].to_string();
    Some(Logo {
        view_box,
        namespaces,
        inner,
        aspect: w / h,
    })
}

/// What `parse_xml` learned about a document.
#[derive(Debug, Default)]
pub struct XmlInfo {
    pub root: String,
    /// Attribute values as written (entities not decoded).
    pub root_attrs: Vec<(String, String)>,
    /// Byte range between the root's start and end tags.
    pub inner: (usize, usize),
    /// Scripts, event handlers, `javascript:` links, foreign content or entity declarations seen.
    pub risky: bool,
}

/// True when `xml` is well-formed: one root, balanced tags, quoted unique attributes, only the five
/// predefined entities and numeric references. Checks syntax only, not namespaces.
pub fn is_well_formed(xml: &str) -> bool {
    parse_xml(xml).is_ok()
}

fn is_name_start(c: char) -> bool {
    c.is_alphabetic() || c == '_' || c == ':'
}

fn is_name_char(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '_' | ':' | '-' | '.')
}

fn check_text(t: &str) -> Result<(), String> {
    if t.contains("]]>") {
        return Err("`]]>` in text".into());
    }
    if let Some(c) = t.chars().find(|c| !valid_char(*c)) {
        return Err(format!("invalid character {c:?}"));
    }
    let mut rest = t;
    while let Some(i) = rest.find('&') {
        let tail = &rest[i + 1..];
        let end = tail
            .find(';')
            .filter(|&e| e <= 10)
            .ok_or("unterminated entity")?;
        let body = &tail[..end];
        let ok = matches!(body, "amp" | "lt" | "gt" | "quot" | "apos")
            || body
                .strip_prefix("#x")
                .is_some_and(|h| !h.is_empty() && h.chars().all(|c| c.is_ascii_hexdigit()))
            || body
                .strip_prefix('#')
                .is_some_and(|dg| !dg.is_empty() && dg.chars().all(|c| c.is_ascii_digit()));
        if !ok {
            return Err(format!("unknown entity &{body};"));
        }
        rest = &tail[end + 1..];
    }
    Ok(())
}

struct StartTag {
    name: String,
    attrs: Vec<(String, String)>,
    self_closing: bool,
    end: usize,
}

fn parse_start(s: &str, at: usize) -> Result<StartTag, String> {
    let b = s.as_bytes();
    let mut j = at + 1;
    let name_len = s[j..]
        .find(|c: char| !is_name_char(c))
        .unwrap_or(s.len() - j);
    let name = &s[j..j + name_len];
    if !name.chars().next().is_some_and(is_name_start) {
        return Err("bad element name".into());
    }
    j += name_len;
    let mut attrs: Vec<(String, String)> = vec![];
    loop {
        let skipped = s[j..]
            .find(|c: char| !c.is_whitespace())
            .unwrap_or(s.len() - j);
        j += skipped;
        if j >= b.len() {
            return Err("unterminated tag".into());
        }
        if s[j..].starts_with("/>") {
            return Ok(StartTag {
                name: name.into(),
                attrs,
                self_closing: true,
                end: j + 2,
            });
        }
        if b[j] == b'>' {
            return Ok(StartTag {
                name: name.into(),
                attrs,
                self_closing: false,
                end: j + 1,
            });
        }
        if skipped == 0 && !attrs.is_empty() {
            return Err("missing space before attribute".into());
        }
        let an = s[j..]
            .find(|c: char| !is_name_char(c))
            .unwrap_or(s.len() - j);
        let aname = &s[j..j + an];
        if !aname.chars().next().is_some_and(is_name_start) {
            return Err("bad attribute name".into());
        }
        j += an;
        j += s[j..]
            .find(|c: char| !c.is_whitespace())
            .unwrap_or(s.len() - j);
        if b.get(j) != Some(&b'=') {
            return Err("attribute without value".into());
        }
        j += 1;
        j += s[j..]
            .find(|c: char| !c.is_whitespace())
            .unwrap_or(s.len() - j);
        let q = *b.get(j).ok_or("unterminated tag")?;
        if q != b'"' && q != b'\'' {
            return Err("unquoted attribute".into());
        }
        let close = s[j + 1..].find(q as char).ok_or("unterminated attribute")?;
        let value = &s[j + 1..j + 1 + close];
        if value.contains('<') {
            return Err("`<` in attribute".into());
        }
        check_text(value)?;
        if attrs.iter().any(|(n, _)| n == aname) {
            return Err(format!("duplicate attribute {aname}"));
        }
        attrs.push((aname.into(), value.into()));
        j += close + 2;
    }
}

pub fn parse_xml(s: &str) -> Result<XmlInfo, String> {
    let len = s.len();
    let mut i = if s.starts_with('\u{feff}') { 3 } else { 0 };
    let mut stack: Vec<String> = vec![];
    let mut info = XmlInfo::default();
    let mut have_root = false;
    while i < len {
        if s.as_bytes()[i] != b'<' {
            let next = s[i..].find('<').map_or(len, |p| i + p);
            let text = &s[i..next];
            if stack.is_empty() {
                if !text.trim().is_empty() {
                    return Err("text outside the root element".into());
                }
            } else {
                check_text(text)?;
            }
            i = next;
            continue;
        }
        let rest = &s[i..];
        if rest.starts_with("<?") {
            i += rest
                .find("?>")
                .ok_or("unterminated processing instruction")?
                + 2;
        } else if let Some(body) = rest.strip_prefix("<!--") {
            i += 4 + body.find("-->").ok_or("unterminated comment")? + 3;
        } else if rest.starts_with("<![CDATA[") {
            if stack.is_empty() {
                return Err("CDATA outside the root element".into());
            }
            i += rest.find("]]>").ok_or("unterminated CDATA")? + 3;
        } else if rest.starts_with("<!") {
            if have_root {
                return Err("declaration after the root element".into());
            }
            let gt = rest.find('>').ok_or("unterminated declaration")?;
            let end = if rest[..gt].contains('[') {
                rest.find("]>").ok_or("unterminated declaration")? + 2
            } else {
                gt + 1
            };
            if rest[..end].to_ascii_uppercase().contains("ENTITY") {
                info.risky = true;
            }
            i += end;
        } else if rest.starts_with("</") {
            let gt = rest.find('>').ok_or("unterminated end tag")?;
            let name = rest[2..gt].trim();
            if stack.pop().as_deref() != Some(name) {
                return Err(format!("mismatched end tag </{name}>"));
            }
            if stack.is_empty() {
                info.inner.1 = i;
            }
            i += gt + 1;
        } else {
            let tag = parse_start(s, i)?;
            let lname = tag.name.to_ascii_lowercase();
            if matches!(
                lname.as_str(),
                "script" | "foreignobject" | "iframe" | "object" | "embed"
            ) {
                info.risky = true;
            }
            for (k, v) in &tag.attrs {
                let k = k.to_ascii_lowercase();
                if (k.starts_with("on") && k.len() > 2)
                    || v.trim_start()
                        .to_ascii_lowercase()
                        .starts_with("javascript:")
                {
                    info.risky = true;
                }
            }
            if stack.is_empty() {
                if have_root {
                    return Err("more than one root element".into());
                }
                have_root = true;
                info.root = tag.name.clone();
                info.root_attrs = tag.attrs.clone();
                info.inner = (tag.end, tag.end);
            }
            if !tag.self_closing {
                stack.push(tag.name);
            }
            i = tag.end;
        }
    }
    if !stack.is_empty() {
        return Err(format!("<{}> is never closed", stack[stack.len() - 1]));
    }
    if !have_root {
        return Err("no root element".into());
    }
    Ok(info)
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    use crate::brand::{BrandKit, Rgb};
    use crate::model::{Edge, Group, Shape, Task};

    fn count(s: &str, pat: &str) -> usize {
        s.matches(pat).count()
    }

    fn sample() -> Diagram {
        let mut d = classic();
        d.title = "Checkout <flow> & \"co\"".into();
        for (i, shape) in Shape::ALL.iter().enumerate() {
            let n = d.ensure_node(&format!("n{i}"), &format!("Node {i}\nsecond line"));
            n.shape = *shape;
        }
        for i in 1..Shape::ALL.len() {
            d.add_edge(&format!("n{}", i - 1), &format!("n{i}"));
        }
        d
    }

    #[test]
    fn well_formedness_checker_works() {
        assert!(is_well_formed(
            r#"<?xml version="1.0"?><a x="1" y='2'><b/><!-- c --><c>t &amp; &#65; &#x41;</c></a>"#
        ));
        assert!(is_well_formed("<a/>"));
        for bad in [
            "<a><b></a></b>",
            "<a>",
            "",
            "<a/><b/>",
            "<a x=1/>",
            "<a x=\"1\" x=\"2\"/>",
            "<a>&nope;</a>",
            "<a>fish & chips</a>",
            "<a x=\"<\"/>",
            "text<a/>",
            "<a x=\"1\"y=\"2\"/>",
            "<a></b>",
            "</a>",
        ] {
            assert!(!is_well_formed(bad), "{bad}");
        }
    }

    #[test]
    fn output_is_well_formed_for_every_shape_and_brand() {
        for brand in BrandKit::presets() {
            let mut d = sample();
            d.brand = brand;
            d.brand.shadow = true;
            d.brand.show_grid = true;
            let svg = to_svg(&d);
            if let Err(e) = parse_xml(&svg) {
                panic!("{e}\n{svg}");
            }
            assert!(svg.starts_with("<svg "));
            assert!(svg.ends_with("</svg>"));
        }
    }

    /// The classic outline-shape style (the default kit draws cards).
    fn classic() -> Diagram {
        let mut d = Diagram::default();
        d.brand.node_style = crate::brand::NodeStyle::Shape;
        d
    }

    #[test]
    fn all_eleven_shapes_are_drawn() {
        let svg = to_svg(&sample());
        assert_eq!(count(&svg, "<g data-node="), 11);
        assert!(svg.contains("<polygon"), "diamond, hexagon, parallelogram");
        assert_eq!(count(&svg, "<polygon"), 3);
        assert!(svg.contains("<ellipse"));
        assert!(svg.contains(" A"), "cylinder arcs");
        assert!(svg.contains(" T"), "document wave");
    }

    #[test]
    fn user_text_is_escaped() {
        let mut d = Diagram::default();
        d.title = "A <b>&\"'".into();
        let n = d.ensure_node("x\"><script>", "<img src=x onerror=1> & \u{1}ok");
        n.icon = Some("\"><evil/>".into());
        d.groups.push(Group {
            id: "g\"".into(),
            label: "G<>".into(),
            role: Role::Accent,
        });
        d.nodes[0].group = Some("g\"".into());
        d.ensure_node("y", "y");
        let mut e = Edge::new("e<1>", "x\"><script>", "y");
        e.label = "yes & no <>".into();
        d.edges.push(e);
        let svg = to_svg(&d);
        assert!(is_well_formed(&svg), "{svg}");
        assert!(!svg.contains("<script"));
        assert!(!svg.contains("<img"));
        assert!(!svg.contains("<evil"));
        assert!(svg.contains("&lt;img src=x onerror=1&gt; &amp; ok"));
    }

    #[test]
    fn font_stack_is_sanitised() {
        let mut d = sample();
        d.brand.font_family = "Inter\" onload=\"alert(1)\" x=\"<a>, 'Segoe UI', sans-serif".into();
        d.brand.heading_family = "\"\"<>&;".into();
        let svg = to_svg(&d);
        assert!(is_well_formed(&svg));
        assert!(!svg.contains("onload=\"alert"));
        assert!(svg.contains("font-family=\"sans-serif\""));
        assert!(svg.contains("&apos;Segoe UI&apos;"));
    }

    #[test]
    fn brand_colours_and_features_show_up() {
        let mut d = sample();
        d.brand = BrandKit::midnight();
        d.brand.shadow = true;
        d.brand.show_grid = true;
        let svg = to_svg(&d);
        assert!(svg.contains(&d.brand.background.hex()));
        assert!(svg.contains("fd-shadow") && svg.contains("filter=\"url(#fd-shadow)\""));
        assert!(svg.contains("fd-grid"));
        // Shape style: the card's neutral surface and border, the role in the glyph.
        let ink = card::ink(&d.brand, card::is_dark(&d.brand));
        assert!(svg.contains(&ink.surface.hex()) && svg.contains(&ink.border.hex()));
        assert!(svg.contains(&card::vibrant(&d.brand, Role::Primary, true).hex()));
        d.brand.shadow = false;
        d.brand.show_grid = false;
        let plain = to_svg(&d);
        assert!(!plain.contains("fd-shadow") && !plain.contains("fd-grid"));
    }

    #[test]
    fn edges_have_dashes_arrows_and_label_pills() {
        let mut d = classic();
        for id in ["a", "b", "c", "e"] {
            d.ensure_node(id, id);
        }
        let mut e1 = Edge::new("e1", "a", "b");
        e1.line = LineStyle::Dashed;
        e1.label = "approve".into();
        let mut e2 = Edge::new("e2", "a", "c");
        e2.line = LineStyle::Dotted;
        e2.arrow = Arrow::Both;
        let mut e3 = Edge::new("e3", "a", "e");
        e3.arrow = Arrow::None;
        e3.role = Some(Role::Danger);
        d.edges.extend([e1, e2, e3]);
        let svg = to_svg(&d);
        assert!(is_well_formed(&svg));
        assert_eq!(count(&svg, "stroke-dasharray"), 2);
        // e1 one head, e2 two heads, e3 none
        let heads: usize = svg
            .split("<g data-edge=\"")
            .skip(1)
            .map(|g| count(g.split("</g>").next().unwrap_or(""), "stroke-width=\"1\""))
            .sum();
        assert_eq!(heads, 3);
        assert_eq!(count(&svg, "data-edge-label="), 1);
        assert!(svg.contains(">approve<"));
        assert!(svg.contains(&d.brand.palette.danger.hex()));
    }

    #[test]
    fn task_badges_show_progress() {
        let mut d = classic();
        d.ensure_node("a", "A");
        d.ensure_node("b", "B");
        for (i, done) in [true, false, true].into_iter().enumerate() {
            let mut t = Task::new(format!("t{i}"), "x");
            t.node = Some("a".into());
            t.done = done;
            d.tasks.push(t);
        }
        let mut t = Task::new("t9", "y");
        t.node = Some("b".into());
        t.done = true;
        d.tasks.push(t);
        d.tasks.push(Task::new("loose", "no node"));
        let svg = to_svg(&d);
        assert!(is_well_formed(&svg));
        assert_eq!(count(&svg, "data-badge="), 2);
        assert!(svg.contains(">2/3<") && svg.contains(">1/1<"));
        let green = d.brand.palette.success.hex();
        assert_eq!(
            count(
                &svg,
                &format!(
                    "fill=\"{green}\" stroke=\"{}\" stroke-width=\"2\"",
                    d.brand.background.hex()
                )
            ),
            1
        );
    }

    #[test]
    fn known_icons_draw_and_unknown_are_skipped() {
        for name in ICON_NAMES {
            let mut d = Diagram::default();
            d.ensure_node("a", "Label").icon = Some(name.into());
            let svg = to_svg(&d);
            assert!(is_well_formed(&svg), "{name}");
            assert!(svg.contains("scale("), "{name}");
        }
        let mut d = Diagram::default();
        d.ensure_node("a", "Label").icon = Some("unicorn".into());
        let svg = to_svg(&d);
        assert!(!svg.contains("scale("));
        assert!(is_known_icon("Settings") && !is_known_icon("unicorn"));
    }

    #[test]
    fn icons_are_duotone_in_the_node_text_colour() {
        let mut d = classic();
        d.ensure_node("a", "Label").icon = Some("check".into());
        let ink = d.brand.node_colors(d.nodes[0].role).text.hex();
        let svg = to_svg(&d);
        assert!(is_well_formed(&svg));
        assert!(!svg.contains("currentColor"));
        assert!(svg.contains(&format!("fill=\"{ink}\"")));
        assert!(
            svg.contains("opacity=\"0.4\""),
            "the second tone stays translucent"
        );
        assert!(
            svg.contains("scale(1)"),
            "18px glyph at the 14px default font is drawn 1:1"
        );
    }

    #[test]
    fn cards_draw_tile_title_and_progress_instead_of_a_badge() {
        let mut d = Diagram::default();
        assert!(card::uses_cards(&d.brand));
        d.ensure_node("s", "Start").shape = Shape::Pill;
        d.ensure_node("q", "Ok?").shape = Shape::Diamond;
        d.add_edge("s", "q");
        for (i, done) in [true, false, true].into_iter().enumerate() {
            let mut t = Task::new(format!("t{i}"), "x");
            t.node = Some("q".into());
            t.done = done;
            d.tasks.push(t);
        }
        let svg = to_svg(&d);
        assert!(is_well_formed(&svg));
        assert_eq!(count(&svg, "<g data-node="), 2);
        assert_eq!(count(&svg, r#"rx="12""#), 2, "card radius");
        assert_eq!(count(&svg, r#"fill-opacity="0.16""#), 2, "icon tiles");
        assert!(svg.contains("2 of 3 to-dos"));
        assert!(!svg.contains("data-badge="), "no floating badge on cards");
        assert!(
            svg.contains("filter=\"url(#fd-card)\""),
            "light cards get the panel shadow"
        );
        let ry = |n: &str| node_rect(&d, d.node(n).unwrap()).h;
        assert_eq!((ry("s"), ry("q")), (card::H_ONE, card::H_TWO));
    }

    #[test]
    fn card_edges_groups_and_labels_follow_the_spec() {
        let mut d = Diagram::default();
        d.ensure_node("a", "A").group = Some("g".into());
        d.ensure_node("b", "B");
        d.groups.push(crate::model::Group {
            id: "g".into(),
            label: "Quality".into(),
            role: Role::Secondary,
        });
        let e = d.add_edge("a", "b");
        e.label = "yes".into();
        e.role = Some(Role::Danger);
        crate::layout::auto_layout(&mut d);
        let svg = to_svg(&d);
        assert!(is_well_formed(&svg));
        assert!(
            svg.contains(r#"stroke-dasharray="4 4""#),
            "role edges are dashed 4 4"
        );
        assert!(svg.contains(r#"opacity="0.8""#), "role edges at 80 %");
        assert!(svg.contains(r#"rx="6""#), "chip label");
        assert!(
            svg.contains("url(#fd-glass)") && svg.contains("url(#fd-rim)"),
            "glass group"
        );
        assert!(svg.contains(r#"rx="20""#), "group radius");
        assert!(svg.contains(r#"fill-opacity="0.08""#), "light group tint");
    }

    #[test]
    fn logo_is_embedded_and_junk_is_skipped() {
        let mut d = sample();
        d.brand.logo_svg = Some(r##"<?xml version="1.0"?><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 40 20"><circle id="dot" cx="10" cy="10" r="8" fill="#f00"/></svg>"##.into());
        let svg = to_svg(&d);
        assert!(is_well_formed(&svg));
        assert!(svg.contains("id=\"dot\"") && svg.contains("viewBox=\"0 0 40 20\""));
        for junk in [
            "not svg at all",
            "<svg><circle></svg>",
            "<svg viewBox=\"0 0 10 10\"><script>alert(1)</script></svg>",
            "<svg viewBox=\"0 0 10 10\" onload=\"x()\"/>",
            "<div/>",
            "<svg viewBox=\"0 0 0 0\"/>",
            "",
        ] {
            d.brand.logo_svg = Some(junk.into());
            let svg = to_svg(&d);
            assert!(is_well_formed(&svg), "{junk}");
            assert!(
                !svg.contains("alert") && !svg.contains("xMaxYMid meet"),
                "{junk}"
            );
        }
    }

    #[test]
    fn logo_namespaces_survive() {
        let mut d = Diagram::default();
        d.ensure_node("a", "A");
        d.brand.logo_svg = Some(r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="48px" height="24px"><use xlink:href="#q"/></svg>"##.into());
        let svg = to_svg(&d);
        assert!(svg.contains("xmlns:xlink=") && svg.contains("viewBox=\"0 0 48 24\""));
    }

    #[test]
    fn unplaced_nodes_are_laid_out_on_a_clone() {
        let d = sample();
        assert!(d.nodes.iter().all(|n| n.pos.is_none()));
        let svg = to_svg(&d);
        assert!(d.nodes.iter().all(|n| n.pos.is_none()), "input untouched");
        assert_eq!(count(&svg, "<g data-node="), 11);
        // Different nodes land at different places.
        let mut xs: Vec<&str> = svg
            .split("<g data-node=")
            .skip(1)
            .map(|s| &s[..s.find('>').unwrap()])
            .collect();
        xs.dedup();
        assert_eq!(xs.len(), 11);
    }

    #[test]
    fn viewbox_contains_everything() {
        let mut d = sample();
        crate::layout::auto_layout(&mut d);
        let svg = to_svg(&d);
        let vb = svg
            .split("viewBox=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap();
        let v: Vec<f32> = vb.split(' ').map(|p| p.parse().unwrap()).collect();
        let b = diagram_bounds(&d, 0.0);
        assert!(v[0] <= b.x && v[1] <= b.y && v[0] + v[2] >= b.x + b.w && v[1] + v[3] >= b.y + b.h);
    }

    #[test]
    fn empty_diagram_and_missing_edge_targets_render() {
        let svg = to_svg(&Diagram::default());
        assert!(is_well_formed(&svg));
        let mut d = Diagram::default();
        d.title = String::new();
        d.ensure_node("a", "");
        d.edges.push(Edge::new("e", "a", "ghost"));
        d.edges.push(Edge::new("loop", "a", "a"));
        let svg = to_svg(&d);
        assert!(is_well_formed(&svg));
        assert!(!svg.contains("<title>"));
        assert_eq!(
            count(&svg, "data-edge=\""),
            1,
            "dangling edge skipped, self-loop drawn"
        );
    }

    #[test]
    fn colours_are_valid_hex() {
        let svg = to_svg(&sample());
        for part in svg.split('#').skip(1) {
            let hex: String = part.chars().take(6).collect();
            if part.starts_with("fd-") || part.starts_with("000000") {
                continue;
            }
            assert!(Rgb::parse(&hex).is_some(), "{hex}");
        }
    }

    #[test]
    fn dump_sample_svg() {
        let Ok(path) = std::env::var("ROVELO_SAMPLE_SVG") else {
            return;
        };
        let mut d = Diagram::default();
        d.title = "Support ticket triage".into();
        d.kind = crate::model::DiagramKind::TaskFlow;
        d.brand = BrandKit::midnight();
        d.brand.shadow = true;
        d.brand.show_grid = true;
        let spec: [(&str, &str, Shape, Role, Option<&str>); 10] = [
            (
                "new",
                "Ticket arrives",
                Shape::Pill,
                Role::Secondary,
                Some("mail"),
            ),
            (
                "auth",
                "Verify customer",
                Shape::Rounded,
                Role::Primary,
                Some("lock"),
            ),
            ("urgent", "Urgent?", Shape::Diamond, Role::Accent, None),
            (
                "page",
                "Page on-call",
                Shape::Rounded,
                Role::Danger,
                Some("bell"),
            ),
            ("queue", "Add to queue", Shape::Rounded, Role::Primary, None),
            (
                "kb",
                "Search\nknowledge base",
                Shape::Document,
                Role::Neutral,
                Some("search"),
            ),
            (
                "reply",
                "Draft reply",
                Shape::Rounded,
                Role::Primary,
                Some("user"),
            ),
            (
                "refund",
                "Refund?",
                Shape::Diamond,
                Role::Accent,
                Some("card"),
            ),
            (
                "fix",
                "Ship the fix",
                Shape::Hexagon,
                Role::Warning,
                Some("gear"),
            ),
            (
                "done",
                "Close ticket",
                Shape::Pill,
                Role::Success,
                Some("check"),
            ),
        ];
        for (id, label, shape, role, ic) in spec {
            let n = d.ensure_node(id, label);
            n.shape = shape;
            n.role = role;
            n.icon = ic.map(String::from);
        }
        d.groups.push(Group {
            id: "triage".into(),
            label: "Triage".into(),
            role: Role::Secondary,
        });
        for id in ["auth", "urgent", "queue"] {
            d.node_mut(id).unwrap().group = Some("triage".into());
        }
        for (a, b, label) in [
            ("new", "auth", ""),
            ("auth", "urgent", ""),
            ("urgent", "page", "yes"),
            ("urgent", "queue", "no"),
            ("page", "kb", ""),
            ("queue", "kb", ""),
            ("kb", "reply", ""),
            ("reply", "refund", ""),
            ("refund", "fix", "bug"),
            ("refund", "done", "no"),
            ("fix", "done", ""),
            ("done", "new", "reopen"),
        ] {
            let e = d.add_edge(a, b);
            e.label = label.into();
            if label == "reopen" {
                e.line = LineStyle::Dashed;
            }
        }
        for (i, (node, title, done)) in [
            ("auth", "Check plan tier", true),
            ("auth", "Confirm email", true),
            ("page", "Open incident", false),
            ("kb", "Link related article", true),
            ("reply", "Write draft", false),
            ("reply", "Get review", false),
            ("fix", "Open PR", false),
            ("done", "Send survey", false),
        ]
        .into_iter()
        .enumerate()
        {
            let mut t = Task::new(format!("t{i}"), title);
            t.node = Some(node.into());
            t.done = done;
            d.tasks.push(t);
        }
        std::fs::write(&path, to_svg(&d)).unwrap();
    }
}
