//! Outlines for the eleven node shapes, plus filling for the one that is not convex.
use egui::epaint::{Mesh, Vertex};
use egui::{Color32, Painter, Pos2, Rect, Shape as EShape, Stroke, pos2};
use rovelo_core::geometry::DOC_WAVE;
use rovelo_core::model::Shape;
use std::f32::consts::PI;

fn arc(out: &mut Vec<Pos2>, c: Pos2, rx: f32, ry: f32, from: f32, to: f32, steps: usize) {
    for i in 0..=steps {
        let a = from + (to - from) * i as f32 / steps as f32;
        out.push(pos2(c.x + rx * a.cos(), c.y + ry * a.sin()));
    }
}

pub fn rounded_rect(r: Rect, radius: f32) -> Vec<Pos2> {
    let rad = radius.clamp(0.0, r.width().min(r.height()) / 2.0);
    if rad < 0.5 {
        return vec![
            r.left_top(),
            r.right_top(),
            r.right_bottom(),
            r.left_bottom(),
        ];
    }
    let steps = ((rad / 3.0).ceil() as usize).clamp(3, 10);
    let mut v = Vec::with_capacity(4 * (steps + 1));
    arc(
        &mut v,
        pos2(r.right() - rad, r.top() + rad),
        rad,
        rad,
        -PI / 2.0,
        0.0,
        steps,
    );
    arc(
        &mut v,
        pos2(r.right() - rad, r.bottom() - rad),
        rad,
        rad,
        0.0,
        PI / 2.0,
        steps,
    );
    arc(
        &mut v,
        pos2(r.left() + rad, r.bottom() - rad),
        rad,
        rad,
        PI / 2.0,
        PI,
        steps,
    );
    arc(
        &mut v,
        pos2(r.left() + rad, r.top() + rad),
        rad,
        rad,
        PI,
        1.5 * PI,
        steps,
    );
    v
}

fn quad(out: &mut Vec<Pos2>, p0: Pos2, c: Pos2, p1: Pos2, steps: usize) {
    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let u = 1.0 - t;
        out.push(pos2(
            u * u * p0.x + 2.0 * u * t * c.x + t * t * p1.x,
            u * u * p0.y + 2.0 * u * t * c.y + t * t * p1.y,
        ));
    }
}

fn cylinder_cap(h_world: f32) -> f32 {
    (h_world * 0.14).clamp(6.0, 14.0)
}

fn note_fold(w_world: f32, h_world: f32) -> f32 {
    16.0f32.min(h_world / 3.0).min(w_world / 3.0)
}

/// Closed outline in screen space, built to match the SVG exporter.
/// `radius` is the brand corner radius in world units; `scale` is the zoom.
pub fn outline(shape: Shape, r: Rect, radius: f32, scale: f32) -> Vec<Pos2> {
    let (w, h) = (r.width(), r.height());
    let (ww, hw) = (w / scale, h / scale);
    let c = r.center();
    match shape {
        Shape::Rect => rounded_rect(r, (radius * 0.3).min(4.0) * scale),
        Shape::Rounded => rounded_rect(r, radius * scale),
        Shape::Pill => rounded_rect(r, h / 2.0),
        Shape::Screen => rounded_rect(r, (radius * 1.8).clamp(14.0, (ww / 4.0).max(14.0)) * scale),
        Shape::Diamond => vec![
            pos2(c.x, r.top()),
            pos2(r.right(), c.y),
            pos2(c.x, r.bottom()),
            pos2(r.left(), c.y),
        ],
        Shape::Circle => {
            let mut v = Vec::new();
            arc(&mut v, c, w / 2.0, h / 2.0, 0.0, 2.0 * PI, 48);
            v.pop();
            v
        }
        Shape::Parallelogram => {
            let s = (h * 0.3).min(w * 0.25);
            vec![
                pos2(r.left() + s, r.top()),
                r.right_top(),
                pos2(r.right() - s, r.bottom()),
                r.left_bottom(),
            ]
        }
        Shape::Hexagon => {
            let i = (h * 0.3).min(w * 0.25);
            vec![
                pos2(r.left() + i, r.top()),
                pos2(r.right() - i, r.top()),
                pos2(r.right(), c.y),
                pos2(r.right() - i, r.bottom()),
                pos2(r.left() + i, r.bottom()),
                pos2(r.left(), c.y),
            ]
        }
        Shape::Cylinder => {
            let e = cylinder_cap(hw) * scale;
            let mut v = Vec::new();
            arc(&mut v, pos2(c.x, r.top() + e), w / 2.0, e, PI, 2.0 * PI, 20);
            arc(&mut v, pos2(c.x, r.bottom() - e), w / 2.0, e, 0.0, PI, 20);
            v
        }
        Shape::Document => {
            let wave = DOC_WAVE * scale;
            let yb = r.bottom() - wave;
            let mut v = vec![r.left_top(), r.right_top(), pos2(r.right(), yb)];
            quad(
                &mut v,
                pos2(r.right(), yb),
                pos2(r.left() + 0.75 * w, yb - wave),
                pos2(r.left() + 0.5 * w, yb),
                14,
            );
            quad(
                &mut v,
                pos2(r.left() + 0.5 * w, yb),
                pos2(r.left() + 0.25 * w, yb + wave),
                pos2(r.left(), yb),
                14,
            );
            v.pop();
            v
        }
        Shape::Note => {
            let f = note_fold(ww, hw) * scale;
            vec![
                r.left_top(),
                pos2(r.right() - f, r.top()),
                pos2(r.right(), r.top() + f),
                r.right_bottom(),
                r.left_bottom(),
            ]
        }
    }
}

/// Marks drawn inside a shape on top of its fill.
pub enum Detail {
    Line(Vec<Pos2>),
    /// A small filled bar (the phone's notch and home indicator).
    Bar(Rect),
    /// The folded corner of a note.
    Fold(Vec<Pos2>),
}

pub fn details(shape: Shape, r: Rect, scale: f32) -> Vec<Detail> {
    let (w, h) = (r.width(), r.height());
    let c = r.center();
    match shape {
        Shape::Cylinder => {
            let e = cylinder_cap(h / scale) * scale;
            let mut v = Vec::new();
            arc(&mut v, pos2(c.x, r.top() + e), w / 2.0, e, 0.0, PI, 20);
            vec![Detail::Line(v)]
        }
        Shape::Note => {
            let f = note_fold(w / scale, h / scale) * scale;
            vec![Detail::Fold(vec![
                pos2(r.right() - f, r.top()),
                pos2(r.right() - f, r.top() + f),
                pos2(r.right(), r.top() + f),
            ])]
        }
        Shape::Screen => vec![
            Detail::Bar(Rect::from_min_size(
                pos2(c.x - 12.0 * scale, r.top() + 10.0 * scale),
                egui::vec2(24.0, 4.0) * scale,
            )),
            Detail::Bar(Rect::from_min_size(
                pos2(c.x - 18.0 * scale, r.bottom() - 14.0 * scale),
                egui::vec2(36.0, 4.0) * scale,
            )),
        ],
        _ => vec![],
    }
}

/// Vertical shift of the label centre, in world units.
pub fn text_offset(shape: Shape, h_world: f32) -> f32 {
    match shape {
        Shape::Cylinder => cylinder_cap(h_world) * 0.5,
        Shape::Document => -DOC_WAVE * 0.6,
        _ => 0.0,
    }
}

fn is_convex(p: &[Pos2]) -> bool {
    let n = p.len();
    if n < 4 {
        return true;
    }
    let mut sign = 0.0f32;
    for i in 0..n {
        let (a, b, c) = (p[i], p[(i + 1) % n], p[(i + 2) % n]);
        let z = (b.x - a.x) * (c.y - b.y) - (b.y - a.y) * (c.x - b.x);
        if z.abs() < 1e-4 {
            continue;
        }
        if sign == 0.0 {
            sign = z.signum();
        } else if z.signum() != sign {
            return false;
        }
    }
    true
}

fn area2(p: &[Pos2]) -> f32 {
    (0..p.len())
        .map(|i| {
            let (a, b) = (p[i], p[(i + 1) % p.len()]);
            a.x * b.y - b.x * a.y
        })
        .sum()
}

fn in_triangle(p: Pos2, a: Pos2, b: Pos2, c: Pos2) -> bool {
    let s = |p1: Pos2, p2: Pos2, p3: Pos2| {
        (p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y)
    };
    let (d1, d2, d3) = (s(p, a, b), s(p, b, c), s(p, c, a));
    let neg = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
    let pos = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
    !(neg && pos)
}

/// Ear-clipping triangulation of a simple polygon (either winding).
pub fn triangulate(pts: &[Pos2]) -> Vec<[usize; 3]> {
    let n = pts.len();
    if n < 3 {
        return vec![];
    }
    let mut idx: Vec<usize> = (0..n).collect();
    if area2(pts) < 0.0 {
        idx.reverse();
    }
    let mut out = Vec::with_capacity(n - 2);
    let mut guard = 0;
    while idx.len() > 3 && guard < n * n {
        guard += 1;
        let m = idx.len();
        let mut clipped = false;
        for i in 0..m {
            let (ia, ib, ic) = (idx[(i + m - 1) % m], idx[i], idx[(i + 1) % m]);
            let (a, b, c) = (pts[ia], pts[ib], pts[ic]);
            if (b.x - a.x) * (c.y - b.y) - (b.y - a.y) * (c.x - b.x) <= 1e-6 {
                continue;
            }
            let blocked = idx
                .iter()
                .any(|&j| j != ia && j != ib && j != ic && in_triangle(pts[j], a, b, c));
            if blocked {
                continue;
            }
            out.push([ia, ib, ic]);
            idx.remove(i);
            clipped = true;
            break;
        }
        if !clipped {
            break;
        }
    }
    if idx.len() == 3 {
        out.push([idx[0], idx[1], idx[2]]);
    }
    out
}

/// Fill a closed outline, convex or not.
pub fn fill(painter: &Painter, pts: &[Pos2], color: Color32) {
    if pts.len() < 3 || color.a() == 0 {
        return;
    }
    if is_convex(pts) {
        painter.add(EShape::convex_polygon(pts.to_vec(), color, Stroke::NONE));
    } else {
        let mut mesh = Mesh::default();
        for p in pts {
            mesh.vertices.push(Vertex {
                pos: *p,
                uv: egui::epaint::WHITE_UV,
                color,
            });
        }
        for t in triangulate(pts) {
            mesh.indices.extend(t.iter().map(|&i| i as u32));
        }
        painter.add(EShape::mesh(mesh));
    }
}

pub fn stroke_closed(painter: &Painter, pts: &[Pos2], stroke: Stroke) {
    if pts.len() >= 2 && stroke.width > 0.0 {
        painter.add(EShape::closed_line(pts.to_vec(), stroke));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::vec2;

    fn tri_area(p: &[Pos2], t: [usize; 3]) -> f32 {
        let (a, b, c) = (p[t[0]], p[t[1]], p[t[2]]);
        (((b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y)) / 2.0).abs()
    }

    #[test]
    fn every_shape_has_a_finite_outline() {
        let r = Rect::from_min_size(pos2(10.0, 20.0), vec2(160.0, 56.0));
        for s in Shape::ALL {
            let o = outline(s, r, 10.0, 1.0);
            assert!(o.len() >= 4, "{s:?}");
            assert!(
                o.iter().all(|p| p.x.is_finite() && p.y.is_finite()),
                "{s:?}"
            );
            assert!(
                o.iter().all(|p| p.x >= r.left() - 0.01
                    && p.x <= r.right() + 0.01
                    && p.y >= r.top() - 0.01
                    && p.y <= r.bottom() + 0.01),
                "{s:?}"
            );
        }
    }

    #[test]
    fn degenerate_rects_do_not_panic() {
        for s in Shape::ALL {
            for r in [
                Rect::from_min_size(Pos2::ZERO, egui::Vec2::ZERO),
                Rect::from_min_size(pos2(1e6, 1e6), vec2(0.5, 0.5)),
            ] {
                let _ = outline(s, r, 10.0, 1.0);
                let _ = details(s, r, 1.0);
            }
        }
    }

    #[test]
    fn triangulation_covers_the_document_shape() {
        let r = Rect::from_min_size(pos2(0.0, 0.0), vec2(160.0, 56.0));
        let o = outline(Shape::Document, r, 10.0, 1.0);
        assert!(!is_convex(&o));
        let tris = triangulate(&o);
        assert_eq!(tris.len(), o.len() - 2);
        let total: f32 = tris.iter().map(|t| tri_area(&o, *t)).sum();
        assert!(
            (total - area2(&o).abs() / 2.0).abs() < 0.5,
            "{total} vs {}",
            area2(&o).abs() / 2.0
        );
    }

    #[test]
    fn triangulates_either_winding_and_convex_shapes() {
        let mut sq = vec![
            pos2(0.0, 0.0),
            pos2(10.0, 0.0),
            pos2(10.0, 10.0),
            pos2(0.0, 10.0),
        ];
        assert_eq!(triangulate(&sq).len(), 2);
        sq.reverse();
        assert_eq!(triangulate(&sq).len(), 2);
        assert!(triangulate(&[pos2(0.0, 0.0)]).is_empty());
        assert!(is_convex(&outline(
            Shape::Hexagon,
            Rect::from_min_size(Pos2::ZERO, vec2(100.0, 50.0)),
            8.0,
            1.0
        )));
    }
}
