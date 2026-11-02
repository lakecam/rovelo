//! Canvas drawing for the "Card" node look, card edges and tinted-glass groups.
//! Mirrors `rovelo_core::svg` so the canvas and the export match (DESIGN.md, "Diagram nodes and groups").
use crate::chrome_icons::{self, Icon};
use crate::shapes;
use crate::theme::{self, rgb};
use crate::view::{View, pt};
use egui::epaint::{Mesh, Vertex, WHITE_UV};
use egui::{Color32, FontId, Pos2, Rect, Shape as EShape, Stroke, StrokeKind, Vec2, pos2, vec2};
use rovelo_core::brand::{BrandKit, Role};
use rovelo_core::card;
use rovelo_core::geometry::{EdgePath, label_lines, text_width};
use rovelo_core::model::{Arrow, Diagram, Edge, Node};

/// What a card needs to know about its state this frame.
pub struct CardState {
    pub selected: bool,
    pub highlight: bool,
    pub editing: bool,
    pub ports: bool,
}

fn with_alpha(c: Color32, a: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(
        c.r(),
        c.g(),
        c.b(),
        (a.clamp(0.0, 1.0) * 255.0).round() as u8,
    )
}

fn title_px(b: &BrandKit) -> f32 {
    b.font_size.clamp(6.0, 96.0)
}

fn line_h(b: &BrandKit) -> f32 {
    card::TITLE_LINE * (title_px(b) / card::TITLE_SIZE).max(0.5)
}

/// Panel shadow under light cards: `0 8px 24px /.08, 0 2px 6px /.04`.
fn card_shadow(painter: &egui::Painter, sr: Rect, radius: f32, z: f32) {
    let px = |v: f32| (v * z).round().clamp(0.0, 120.0);
    for (dy, blur, alpha) in [(8.0, 24.0, 0.08), (2.0, 6.0, 0.04)] {
        let s = egui::Shadow {
            offset: [0, px(dy) as i8],
            blur: px(blur) as u8,
            spread: 0,
            color: Color32::from_rgba_unmultiplied(24, 24, 27, (alpha * 255.0) as u8),
        };
        painter.add(s.as_shape(sr, radius));
    }
}

#[allow(clippy::too_many_arguments)]
pub fn paint_card(
    painter: &egui::Painter,
    d: &Diagram,
    n: &Node,
    sr: Rect,
    z: f32,
    dark: bool,
    st: &CardState,
    accent: Color32,
) {
    let b = &d.brand;
    let ink = card::ink(b, dark);
    let radius = card::RADIUS * z;
    if !dark {
        card_shadow(painter, sr, radius, z);
    }
    painter.rect(
        sr,
        radius,
        rgb(ink.surface),
        Stroke::new(1.0, rgb(ink.border)),
        StrokeKind::Inside,
    );

    // icon tile
    let cy = sr.center().y;
    let tile = Rect::from_min_size(
        pos2(sr.left() + card::PAD * z, cy - card::TILE * z / 2.0),
        Vec2::splat(card::TILE * z),
    );
    painter.rect_filled(
        tile,
        card::TILE_RADIUS * z,
        with_alpha(rgb(b.palette.get(n.role)), 0.16),
    );
    let glyph = rgb(card::vibrant(b, n.role, dark));
    match card::icon_word(d, n)
        .as_deref()
        .and_then(Icon::for_node_word)
    {
        Some(icon) if card::GLYPH * z >= 4.0 => {
            chrome_icons::paint(painter, icon, tile.center(), card::GLYPH * z, glyph);
        }
        _ => {
            painter.circle_filled(tile.center(), 3.0 * z, glyph);
        }
    }

    // title and subtitle
    let fs = title_px(b) * z;
    let lh = line_h(b) * z;
    let lines = label_lines(&n.label);
    let sub = card::subtitle(d, n);
    let content = lines.len() as f32 * lh
        + if sub.is_some() {
            card::SUB_LINE * z
        } else {
            0.0
        };
    let top = cy - content / 2.0;
    let text_x = tile.right() + card::TILE_GAP * z;
    if fs >= 4.0 && !st.editing && !n.label.trim().is_empty() {
        for (i, l) in lines.iter().enumerate() {
            let g = painter.layout_no_wrap(
                l.to_string(),
                FontId::new(fs, theme::medium_family()),
                rgb(ink.text),
            );
            let y = top + i as f32 * lh + lh / 2.0 - g.size().y / 2.0;
            painter.galley(pos2(text_x, y), g, rgb(ink.text));
        }
    }
    if let Some((text, all_done)) = sub {
        let sp = card::SUB_SIZE * z;
        if sp >= 4.0 {
            let y = top + lines.len() as f32 * lh + card::SUB_LINE * z / 2.0;
            let mut x = text_x;
            if all_done {
                chrome_icons::paint(
                    painter,
                    Icon::Done,
                    pos2(x + sp / 2.0, y),
                    sp,
                    rgb(card::DONE),
                );
                x += sp + 4.0 * z;
            }
            let g = painter.layout_no_wrap(text, FontId::proportional(sp), rgb(ink.muted));
            painter.galley(pos2(x, y - g.size().y / 2.0), g, rgb(ink.muted));
        }
    }

    // selection: 2px outline 3 outside, radius 15
    if st.selected || st.highlight {
        let ring = sr.expand(card::SELECT_GAP * z + 1.0);
        let r = card::SELECT_RADIUS * z;
        if st.highlight && !st.selected {
            let mut pts = shapes::rounded_rect(ring, r);
            if let Some(f) = pts.first().copied() {
                pts.push(f);
            }
            painter.extend(EShape::dashed_line(
                &pts,
                Stroke::new(2.0, accent),
                5.0,
                4.0,
            ));
        } else {
            painter.rect_stroke(ring, r, Stroke::new(2.0, accent), StrokeKind::Middle);
        }
    }
    if st.ports {
        paint_ports(painter, sr, dark, b);
    }
}

/// Four 8px ports centred on the card's edges: surface fill, 1.5px zinc-500 ring.
pub fn paint_ports(painter: &egui::Painter, sr: Rect, dark: bool, b: &BrandKit) {
    let fill = rgb(card::ink(b, dark).surface);
    let ring = Stroke::new(1.5, Color32::from_rgb(0x71, 0x71, 0x7a));
    let r = card::PORT / 2.0;
    for p in [
        sr.center_top(),
        sr.right_center(),
        sr.center_bottom(),
        sr.left_center(),
    ] {
        painter.circle(p, r, fill, ring);
    }
}

/// A group as tinted glass: role fill, a white wash to 40 %, a vibrant edge and a top highlight.
#[allow(clippy::too_many_arguments)]
pub fn paint_glass_group(
    painter: &egui::Painter,
    b: &BrandKit,
    role: Role,
    label: &str,
    sr: Rect,
    z: f32,
    dark: bool,
    selected: bool,
    accent: Color32,
) {
    let radius = (card::GROUP_RADIUS * z).min(sr.width().min(sr.height()) / 2.0);
    painter.rect_filled(
        sr,
        radius,
        with_alpha(rgb(b.palette.get(role)), card::group_alpha(dark)),
    );
    let wash = Rect::from_min_size(sr.min, vec2(sr.width(), sr.height() * card::GLASS_STOP));
    gradient_fill(
        painter,
        &shapes::rounded_rect(wash, radius),
        wash,
        card::GLASS_TOP,
    );
    let edge = rgb(card::group_edge(b, role, dark));
    painter.rect_stroke(
        sr,
        radius,
        Stroke::new(
            card::GROUP_BORDER * z.max(0.67),
            if selected { accent } else { edge },
        ),
        StrokeKind::Inside,
    );
    if selected {
        painter.rect_stroke(
            sr.expand(card::SELECT_GAP * z + 1.0),
            radius + card::SELECT_GAP * z,
            Stroke::new(2.0, accent),
            StrokeKind::Middle,
        );
    }
    top_rim(
        painter,
        sr.shrink(1.5 * z),
        (radius - 1.5 * z).max(0.0),
        sr.height() * 0.06,
    );

    let title = rgb(card::vibrant(b, role, dark));
    let cy = sr.top() + rovelo_core::geometry::GROUP_TITLE * z / 2.0;
    let dot_x = sr.left() + (card::GROUP_TITLE_INSET + card::GROUP_DOT / 2.0) * z;
    painter.circle_filled(pos2(dot_x, cy), card::GROUP_DOT * z / 2.0, title);
    let px = card::GROUP_TITLE_SIZE * z;
    if px >= 4.0 && !label.trim().is_empty() {
        let g = painter.layout_no_wrap(
            label.to_string(),
            FontId::new(px, theme::semibold_family()),
            title,
        );
        let x = sr.left() + (card::GROUP_TITLE_INSET + card::GROUP_DOT + 8.0) * z;
        painter.galley(pos2(x, cy - g.size().y / 2.0), g, title);
    }
}

/// Fill a convex outline with white fading from `top_alpha` at `band.top()` to 0 at `band.bottom()`.
fn gradient_fill(painter: &egui::Painter, pts: &[Pos2], band: Rect, top_alpha: f32) {
    if pts.len() < 3 || band.height() < 1.0 {
        return;
    }
    let col = |y: f32| {
        let t = ((y - band.top()) / band.height()).clamp(0.0, 1.0);
        let a = ((1.0 - t) * top_alpha * 255.0).round() as u8;
        Color32::from_white_alpha(a)
    };
    let mut mesh = Mesh::default();
    let c = band.center();
    mesh.vertices.push(Vertex {
        pos: c,
        uv: WHITE_UV,
        color: col(c.y),
    });
    for p in pts {
        mesh.vertices.push(Vertex {
            pos: *p,
            uv: WHITE_UV,
            color: col(p.y),
        });
    }
    let n = pts.len() as u32;
    for i in 0..n {
        mesh.add_triangle(0, 1 + i, 1 + (i + 1) % n);
    }
    painter.add(EShape::mesh(mesh));
}

/// The 1px white 8 % highlight along the top edge, fading out over `fade` px.
fn top_rim(painter: &egui::Painter, r: Rect, radius: f32, fade: f32) {
    let fade = fade.max(1.0);
    let rad = radius.min(r.width() / 2.0).min(r.height() / 2.0);
    let mut pts = Vec::new();
    let steps = 10;
    for i in 0..=steps {
        let a = std::f32::consts::PI + std::f32::consts::FRAC_PI_2 * i as f32 / steps as f32;
        pts.push(pos2(
            r.left() + rad + rad * a.cos(),
            r.top() + rad + rad * a.sin(),
        ));
    }
    for i in 0..=steps {
        let a = 1.5 * std::f32::consts::PI + std::f32::consts::FRAC_PI_2 * i as f32 / steps as f32;
        pts.push(pos2(
            r.right() - rad + rad * a.cos(),
            r.top() + rad + rad * a.sin(),
        ));
    }
    for w in pts.windows(2) {
        let y = (w[0].y + w[1].y) / 2.0;
        let t = ((y - r.top()) / fade).clamp(0.0, 1.0);
        let a = ((1.0 - t) * card::HIGHLIGHT * 255.0).round() as u8;
        if a > 0 {
            painter.line_segment([w[0], w[1]], Stroke::new(1.0, Color32::from_white_alpha(a)));
        }
    }
}

/// A card-style edge: neutral 1.5px, or the role's vibrant step at 80 % dashed 4 4; arrowhead 8.
#[allow(clippy::too_many_arguments)]
pub fn paint_card_edge(
    painter: &egui::Painter,
    e: &Edge,
    path: &EdgePath,
    b: &BrandKit,
    view: &View,
    origin: Pos2,
    dark: bool,
    selected: bool,
    accent: Color32,
) {
    let z = view.zoom;
    let ink = card::ink(b, dark);
    let (col, dashed) = match e.role {
        Some(role) => (
            with_alpha(rgb(card::vibrant(b, role, dark)), card::ROLE_EDGE_ALPHA),
            true,
        ),
        None => (rgb(ink.edge), false),
    };
    let pts: Vec<Pos2> = path
        .points
        .iter()
        .map(|p| view.to_screen(origin, pt(*p)))
        .collect();
    let n = pts.len();
    if n < 2 || pts.iter().any(|p| !p.x.is_finite() || !p.y.is_finite()) {
        return;
    }
    let sw = (card::EDGE_WIDTH * z).max(1.0);
    let head = card::ARROW * z;
    let end_dir = dir_or(pts[n - 2], pts[n - 1], vec2(path.end_dir.x, path.end_dir.y));
    let start_dir = dir_or(pts[1], pts[0], vec2(path.start_dir.x, path.start_dir.y));
    let mut line = pts.clone();
    if e.arrow != Arrow::None {
        line[n - 1] -= end_dir * (head * 0.7).min((pts[n - 1] - pts[n - 2]).length());
    }
    if e.arrow == Arrow::Both {
        line[0] -= start_dir * (head * 0.7).min((pts[0] - pts[1]).length());
    }
    if selected {
        painter.add(EShape::line(
            pts.clone(),
            Stroke::new(sw + 6.0, with_alpha(accent, 0.28)),
        ));
    }
    if dashed {
        let dash = (card::DASH * z).max(1.5);
        painter.extend(EShape::dashed_line(&line, Stroke::new(sw, col), dash, dash));
    } else {
        painter.add(EShape::line(line, Stroke::new(sw, col)));
    }
    let arrow = |tip: Pos2, dir: Vec2| {
        let perp = vec2(-dir.y, dir.x);
        let base = tip - dir * head;
        painter.add(EShape::convex_polygon(
            vec![tip, base + perp * head * 0.5, base - perp * head * 0.5],
            col,
            Stroke::NONE,
        ));
    };
    if e.arrow != Arrow::None {
        arrow(pts[n - 1], end_dir);
    }
    if e.arrow == Arrow::Both {
        arrow(pts[0], start_dir);
    }
}

/// An edge label as a chip: height 20, padding 0 8, radius 6, surface fill, text-xs muted.
pub fn paint_label_chip(
    painter: &egui::Painter,
    label: &str,
    at: Pos2,
    b: &BrandKit,
    z: f32,
    dark: bool,
) {
    let ink = card::ink(b, dark);
    let lines = label_lines(label);
    let fs = card::LABEL_SIZE;
    let w = text_width(label, fs) + 2.0 * card::LABEL_PAD_X;
    let h = card::LABEL_H + (lines.len() as f32 - 1.0).max(0.0) * fs * 1.25;
    let px = fs * z;
    let g = painter.layout_no_wrap(
        lines.join("\n"),
        FontId::new(px, theme::medium_family()),
        rgb(ink.muted),
    );
    let size = vec2((w * z).max(g.size().x + 2.0 * card::LABEL_PAD_X * z), h * z);
    let r = Rect::from_center_size(at, size);
    painter.rect(
        r,
        card::LABEL_RADIUS * z,
        rgb(ink.surface),
        Stroke::new(1.0, rgb(ink.label_border)),
        StrokeKind::Inside,
    );
    if px >= 4.0 {
        painter.galley(at - g.size() / 2.0, g, rgb(ink.muted));
    }
}

fn dir_or(a: Pos2, b: Pos2, fallback: Vec2) -> Vec2 {
    let v = b - a;
    if v.length() > 1e-3 {
        v.normalized()
    } else if fallback.length() > 1e-3 {
        fallback.normalized()
    } else {
        vec2(1.0, 0.0)
    }
}
