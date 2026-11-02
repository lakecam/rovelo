//! Sizes, rectangles and edge paths. Shared by the egui canvas and the SVG exporter so
//! both draw exactly the same diagram.
use crate::brand::EdgeRouting;
use crate::model::{Diagram, Direction, Edge, Node, Point, Rect, Shape, Size};

const CHAR_W: f32 = 0.56;
const LINE_H: f32 = 1.35;
/// Room an icon takes in front of the label (glyph plus gap).
pub const ICON_W: f32 = 26.0;
pub const GROUP_PAD: f32 = 16.0;
pub const GROUP_TITLE: f32 = 36.0;
/// How far the wavy bottom of a `Document` sits above its bounding box.
pub const DOC_WAVE: f32 = 10.0;
const SELF_LOOP: f32 = 34.0;
const LANE_GAP: f32 = 14.0;
const CURVE_SAMPLES: usize = 24;
/// Least distance between an edge (or a label chip) and a group border it does not cross.
pub const GROUP_CLEAR: f32 = 16.0;

/// Lines of a label (always at least one).
pub fn label_lines(s: &str) -> Vec<&str> {
    s.split('\n').map(|l| l.trim_end_matches('\r')).collect()
}

/// Estimated width of the widest line.
pub fn text_width(s: &str, font_size: f32) -> f32 {
    let widest = label_lines(s)
        .iter()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(0);
    widest as f32 * CHAR_W * font_size
}

/// Estimated height of all lines.
pub fn text_height(s: &str, font_size: f32) -> f32 {
    label_lines(s).len() as f32 * font_size * LINE_H
}

/// Size of an edge's label chip (cards) or pill (classic shapes); `None` for an unlabelled edge.
pub fn edge_label_size(d: &Diagram, e: &Edge) -> Option<(f32, f32)> {
    if e.label.trim().is_empty() {
        return None;
    }
    let lines = label_lines(&e.label).len() as f32;
    Some(if crate::card::uses_cards(&d.brand) {
        let fs = crate::card::LABEL_SIZE;
        (
            text_width(&e.label, fs) + 2.0 * crate::card::LABEL_PAD_X,
            crate::card::LABEL_H + (lines - 1.0) * fs * 1.25,
        )
    } else {
        let fs = d.brand.font_size.clamp(6.0, 96.0) * 0.86;
        (text_width(&e.label, fs) + 16.0, lines * fs * 1.25 + 8.0)
    })
}

/// The label rectangle of an edge whose path is already routed.
pub fn edge_label_rect(d: &Diagram, e: &Edge, path: &EdgePath) -> Option<Rect> {
    if path.points.len() < 2 {
        return None;
    }
    let (w, h) = edge_label_size(d, e)?;
    Some(Rect {
        x: path.label_at.x - w / 2.0,
        y: path.label_at.y - h / 2.0,
        w,
        h,
    })
}

fn overlaps(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
}

fn inside(a: Rect, b: Rect) -> bool {
    a.x >= b.x && a.y >= b.y && a.x + a.w <= b.x + b.w && a.y + a.h <= b.y + b.h
}

/// Rectangles of every placed group.
fn group_rects(d: &Diagram) -> Vec<Rect> {
    d.groups
        .iter()
        .filter_map(|g| group_rect(d, &g.id))
        .collect()
}

fn even_up(v: f32) -> f32 {
    (v / 2.0).ceil() * 2.0
}

/// Size of a node: its manual override, else measured from label, shape and icon.
pub fn node_size(d: &Diagram, node: &Node) -> Size {
    if let Some(s) = node.size {
        return Size {
            w: s.w.max(1.0),
            h: s.h.max(1.0),
        };
    }
    let fs = d.brand.font_size.clamp(6.0, 96.0);
    if crate::card::uses_cards(&d.brand) {
        return card_size(d, node, fs);
    }
    // Every Shape-style node draws a glyph (its icon, its kind's icon or a dot) before the label.
    let tw = text_width(&node.label, fs) + ICON_W;
    let th = text_height(&node.label, fs);
    let (w, h) = match node.shape {
        Shape::Rect | Shape::Rounded => ((tw + 32.0).max(100.0), (th + 24.0).max(48.0)),
        Shape::Pill => ((tw + 48.0).max(110.0), (th + 24.0).max(48.0)),
        Shape::Diamond => ((1.9 * (tw + 12.0)).max(120.0), (1.9 * (th + 8.0)).max(76.0)),
        Shape::Circle => {
            let dia = ((tw + 16.0).hypot(th + 8.0) + 8.0).max(72.0);
            (dia, dia)
        }
        Shape::Hexagon => {
            let h = (th + 24.0).max(60.0);
            ((tw + 32.0 + 0.6 * h).max(140.0), h)
        }
        Shape::Parallelogram => {
            let h = (th + 24.0).max(52.0);
            ((tw + 32.0 + 0.6 * h).max(150.0), h)
        }
        Shape::Cylinder => ((tw + 32.0).max(110.0), (th + 48.0).max(76.0)),
        Shape::Document => ((tw + 32.0).max(120.0), (th + 24.0 + DOC_WAVE).max(60.0)),
        Shape::Screen => ((tw + 32.0).max(120.0), (th + 140.0).max(220.0)),
        Shape::Note => ((tw + 40.0).max(120.0), (th + 24.0).max(56.0)),
    };
    Size {
        w: even_up(w),
        h: even_up(h),
    }
}

/// A card: icon tile, title and (with to-dos) a subtitle; at least 192 wide, 48 or 64 tall.
fn card_size(d: &Diagram, node: &Node, fs: f32) -> Size {
    use crate::card;
    let lines = label_lines(&node.label).len().max(1) as f32;
    let title_w = text_width(&node.label, fs);
    let sub = d.tasks_for(&node.id).next().is_some();
    let w = (card::PAD + card::TILE + card::TILE_GAP + title_w + card::PAD).max(card::MIN_W);
    let content = lines * card::TITLE_LINE * (fs / card::TITLE_SIZE).max(0.5)
        + if sub { card::SUB_LINE } else { 0.0 };
    let min_h = if sub { card::H_TWO } else { card::H_ONE };
    let h = (content + 2.0 * card::PAD).max(min_h);
    Size {
        w: even_up(w),
        h: even_up(h),
    }
}

/// The outline edges attach to: a card is a rounded rectangle whatever its kind.
pub fn outline_shape(d: &Diagram, node: &Node) -> Shape {
    if crate::card::uses_cards(&d.brand) {
        Shape::Rounded
    } else {
        node.shape
    }
}

/// Rectangle of a node at its position (origin if unpositioned).
pub fn node_rect(d: &Diagram, node: &Node) -> Rect {
    let s = node_size(d, node);
    let p = node.pos.unwrap_or_default();
    Rect {
        x: p.x,
        y: p.y,
        w: s.w,
        h: s.h,
    }
}

/// Bounding box of a group: its members' rectangles plus padding and a title strip.
/// `None` when no member has a position.
pub fn group_rect(d: &Diagram, group_id: &str) -> Option<Rect> {
    let mut acc: Option<Rect> = None;
    for n in d
        .nodes
        .iter()
        .filter(|n| n.group.as_deref() == Some(group_id) && n.pos.is_some())
    {
        let r = node_rect(d, n);
        acc = Some(acc.map_or(r, |a| a.union(&r)));
    }
    acc.map(|r| Rect {
        x: r.x - GROUP_PAD,
        y: r.y - GROUP_PAD - GROUP_TITLE,
        w: r.w + 2.0 * GROUP_PAD,
        h: r.h + 2.0 * GROUP_PAD + GROUP_TITLE,
    })
}

/// Bounds of the whole diagram including groups, with `margin` around it.
/// Unpositioned nodes are ignored; an empty diagram gets a small default canvas.
pub fn diagram_bounds(d: &Diagram, margin: f32) -> Rect {
    let mut acc: Option<Rect> = None;
    let mut add = |r: Rect| acc = Some(acc.map_or(r, |a| a.union(&r)));
    for n in d.nodes.iter().filter(|n| n.pos.is_some()) {
        add(node_rect(d, n));
    }
    for g in &d.groups {
        if let Some(r) = group_rect(d, &g.id) {
            add(r);
        }
    }
    match acc {
        Some(r) => r.inflate(margin),
        None => Rect {
            x: 0.0,
            y: 0.0,
            w: 320.0,
            h: 200.0,
        },
    }
}

/// Outline of the polygonal shapes (diamond, hexagon, parallelogram); `None` for the rest.
pub fn shape_polygon(shape: Shape, r: Rect) -> Option<Vec<Point>> {
    let (x, y, w, h) = (r.x, r.y, r.w, r.h);
    let c = r.center();
    match shape {
        Shape::Diamond => Some(vec![
            Point::new(c.x, y),
            Point::new(x + w, c.y),
            Point::new(c.x, y + h),
            Point::new(x, c.y),
        ]),
        Shape::Hexagon => {
            let i = (h * 0.3).min(w * 0.25);
            Some(vec![
                Point::new(x + i, y),
                Point::new(x + w - i, y),
                Point::new(x + w, c.y),
                Point::new(x + w - i, y + h),
                Point::new(x + i, y + h),
                Point::new(x, c.y),
            ])
        }
        Shape::Parallelogram => {
            let s = (h * 0.3).min(w * 0.25);
            Some(vec![
                Point::new(x + s, y),
                Point::new(x + w, y),
                Point::new(x + w - s, y + h),
                Point::new(x, y + h),
            ])
        }
        _ => None,
    }
}

fn padd(a: Point, b: Point) -> Point {
    Point::new(a.x + b.x, a.y + b.y)
}

fn psub(a: Point, b: Point) -> Point {
    Point::new(a.x - b.x, a.y - b.y)
}

fn pmul(a: Point, k: f32) -> Point {
    Point::new(a.x * k, a.y * k)
}

fn plen(a: Point) -> f32 {
    a.x.hypot(a.y)
}

fn unit(a: Point) -> Option<Point> {
    let l = plen(a);
    (l > 1e-4).then(|| pmul(a, 1.0 / l))
}

/// Where a ray from `origin` (inside the shape) along `dir` leaves the shape's outline.
pub fn boundary_hit(shape: Shape, r: Rect, origin: Point, dir: Point) -> Point {
    let Some(dir) = unit(dir) else { return origin };
    let t = match shape {
        Shape::Circle => ellipse_exit(r, origin, dir),
        Shape::Diamond | Shape::Hexagon | Shape::Parallelogram => {
            let poly = shape_polygon(shape, r).unwrap_or_default();
            polygon_exit(&poly, origin, dir).unwrap_or_else(|| rect_exit(r, origin, dir))
        }
        Shape::Document => rect_exit(
            Rect {
                h: r.h - DOC_WAVE,
                ..r
            },
            origin,
            dir,
        ),
        _ => rect_exit(r, origin, dir),
    };
    padd(origin, pmul(dir, t.max(0.0)))
}

fn rect_exit(r: Rect, o: Point, d: Point) -> f32 {
    let mut t = f32::MAX;
    if d.x > 1e-6 {
        t = t.min((r.x + r.w - o.x) / d.x);
    } else if d.x < -1e-6 {
        t = t.min((r.x - o.x) / d.x);
    }
    if d.y > 1e-6 {
        t = t.min((r.y + r.h - o.y) / d.y);
    } else if d.y < -1e-6 {
        t = t.min((r.y - o.y) / d.y);
    }
    if t == f32::MAX { 0.0 } else { t }
}

fn ellipse_exit(r: Rect, o: Point, d: Point) -> f32 {
    let c = r.center();
    let (a, b) = ((r.w / 2.0).max(1e-3), (r.h / 2.0).max(1e-3));
    let (ox, oy) = (o.x - c.x, o.y - c.y);
    let qa = (d.x / a).powi(2) + (d.y / b).powi(2);
    let qb = 2.0 * (ox * d.x / (a * a) + oy * d.y / (b * b));
    let qc = (ox / a).powi(2) + (oy / b).powi(2) - 1.0;
    let disc = qb * qb - 4.0 * qa * qc;
    if disc < 0.0 || qa <= 0.0 {
        return 0.0;
    }
    (-qb + disc.sqrt()) / (2.0 * qa)
}

fn polygon_exit(poly: &[Point], o: Point, d: Point) -> Option<f32> {
    let mut best: Option<f32> = None;
    for i in 0..poly.len() {
        let (p, q) = (poly[i], poly[(i + 1) % poly.len()]);
        let e = psub(q, p);
        let den = d.x * e.y - d.y * e.x;
        if den.abs() < 1e-9 {
            continue;
        }
        let w = psub(p, o);
        let t = (w.x * e.y - w.y * e.x) / den;
        let s = (w.x * d.y - w.y * d.x) / den;
        if t >= -1e-4 && (-1e-4..=1.0 + 1e-4).contains(&s) {
            best = Some(best.map_or(t, |b| b.max(t)));
        }
    }
    best
}

/// A routed edge, already flattened to a polyline (curves are sampled).
#[derive(Clone, Debug, Default)]
pub struct EdgePath {
    pub points: Vec<Point>,
    /// Where the label sits.
    pub label_at: Point,
    /// Unit vector along the last segment, the way an arrowhead at the target points
    /// (travelling into the target).
    pub end_dir: Point,
    /// Unit vector the arrowhead at the source points: the first segment reversed
    /// (travelling into the source). For `Arrow::Both`.
    pub start_dir: Point,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Side {
    Top,
    Right,
    Bottom,
    Left,
}

impl Side {
    fn normal(self) -> Point {
        match self {
            Side::Top => Point::new(0.0, -1.0),
            Side::Right => Point::new(1.0, 0.0),
            Side::Bottom => Point::new(0.0, 1.0),
            Side::Left => Point::new(-1.0, 0.0),
        }
    }

    fn vertical(self) -> bool {
        matches!(self, Side::Top | Side::Bottom)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    /// Target lies ahead along the flow axis.
    Forward,
    /// Target lies behind the source along the flow axis.
    Backward,
    /// Side by side across the flow.
    Lateral,
}

/// Exit side on `a` and entry side on `b` for an elbow or curved edge.
fn pick_sides(dir: Direction, a: Rect, b: Rect) -> Option<(Side, Side, Kind)> {
    let vertical = matches!(dir, Direction::Down | Direction::Up);
    let (ap, bp, a_s, b_s) = if vertical {
        (
            (a.y, a.y + a.h),
            (b.y, b.y + b.h),
            (a.x, a.x + a.w),
            (b.x, b.x + b.w),
        )
    } else {
        (
            (a.x, a.x + a.w),
            (b.x, b.x + b.w),
            (a.y, a.y + a.h),
            (b.y, b.y + b.h),
        )
    };
    let (lo, hi) = if vertical {
        (Side::Top, Side::Bottom)
    } else {
        (Side::Left, Side::Right)
    };
    let (s_lo, s_hi) = if vertical {
        (Side::Left, Side::Right)
    } else {
        (Side::Top, Side::Bottom)
    };
    let toward_high = matches!(dir, Direction::Down | Direction::Right);
    let (ahead, behind) = if toward_high {
        (Kind::Forward, Kind::Backward)
    } else {
        (Kind::Backward, Kind::Forward)
    };
    if bp.0 >= ap.1 {
        Some((hi, lo, ahead))
    } else if bp.1 <= ap.0 {
        Some((lo, hi, behind))
    } else if b_s.0 >= a_s.1 {
        Some((s_hi, s_lo, Kind::Lateral))
    } else if b_s.1 <= a_s.0 {
        Some((s_lo, s_hi, Kind::Lateral))
    } else {
        None
    }
}

/// Shift for this edge among the edges joining the same two nodes, so they run side by side.
fn lane_offset(d: &Diagram, edge: &Edge, limit: f32) -> f32 {
    let same = |e: &Edge| {
        (e.from == edge.from && e.to == edge.to) || (e.from == edge.to && e.to == edge.from)
    };
    let mut n = 0usize;
    let mut idx = 0usize;
    for e in d.edges.iter().filter(|e| same(e)) {
        if std::ptr::eq(e, edge) || e.id == edge.id {
            idx = n;
        }
        n += 1;
    }
    if n <= 1 {
        return 0.0;
    }
    let off = (idx as f32 - (n - 1) as f32 / 2.0) * LANE_GAP;
    off.clamp(-limit, limit)
}

fn cubic(p0: Point, c1: Point, c2: Point, p3: Point) -> Vec<Point> {
    (0..=CURVE_SAMPLES)
        .map(|i| {
            let t = i as f32 / CURVE_SAMPLES as f32;
            let u = 1.0 - t;
            let (a, b, c, e) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
            Point::new(
                a * p0.x + b * c1.x + c * c2.x + e * p3.x,
                a * p0.y + b * c1.y + c * c2.y + e * p3.y,
            )
        })
        .collect()
}

/// Drop repeated points and points in the middle of straight runs.
fn simplify(pts: Vec<Point>) -> Vec<Point> {
    let mut out: Vec<Point> = Vec::with_capacity(pts.len());
    for p in pts {
        if out.last().is_some_and(|l| plen(psub(*l, p)) < 1e-3) {
            continue;
        }
        while out.len() >= 2 {
            let (a, b) = (out[out.len() - 2], out[out.len() - 1]);
            let cross = (b.x - a.x) * (p.y - b.y) - (b.y - a.y) * (p.x - b.x);
            let dot = (b.x - a.x) * (p.x - b.x) + (b.y - a.y) * (p.y - b.y);
            if cross.abs() < 1e-3 && dot > 0.0 {
                out.pop();
            } else {
                break;
            }
        }
        out.push(p);
    }
    out
}

/// Replace each interior corner with a short sampled arc of the given radius.
fn round_corners(p: &[Point], radius: f32) -> Vec<Point> {
    if radius <= 0.5 || p.len() < 3 {
        return p.to_vec();
    }
    let mut out = vec![p[0]];
    for i in 1..p.len() - 1 {
        let (prev, cur, next) = (p[i - 1], p[i], p[i + 1]);
        let (to_prev, to_next) = (psub(prev, cur), psub(next, cur));
        let r = radius.min(plen(to_prev) / 2.0).min(plen(to_next) / 2.0);
        let (Some(up), Some(un)) = (unit(to_prev), unit(to_next)) else {
            out.push(cur);
            continue;
        };
        if r < 0.5 {
            out.push(cur);
            continue;
        }
        let (a, b) = (padd(cur, pmul(up, r)), padd(cur, pmul(un, r)));
        const STEPS: usize = 5;
        for s in 0..=STEPS {
            let t = s as f32 / STEPS as f32;
            let u = 1.0 - t;
            out.push(Point::new(
                u * u * a.x + 2.0 * u * t * cur.x + t * t * b.x,
                u * u * a.y + 2.0 * u * t * cur.y + t * t * b.y,
            ));
        }
    }
    out.push(p[p.len() - 1]);
    out
}

fn finish(points: Vec<Point>) -> EdgePath {
    let mut path = EdgePath {
        points,
        ..EdgePath::default()
    };
    path.label_at = midpoint(&path.points);
    let seg = |a: Point, b: Point| unit(psub(b, a));
    let n = path.points.len();
    path.end_dir = (1..n)
        .rev()
        .find_map(|i| seg(path.points[i - 1], path.points[i]))
        .unwrap_or(Point::new(0.0, 1.0));
    path.start_dir = (1..n)
        .find_map(|i| seg(path.points[i - 1], path.points[i]))
        .map_or(Point::new(0.0, -1.0), |u| pmul(u, -1.0));
    path
}

/// Point halfway along a polyline by length.
fn midpoint(pts: &[Point]) -> Point {
    point_at(pts, 0.5)
}

/// Point at fraction `t` of a polyline's length.
fn point_at(pts: &[Point], t: f32) -> Point {
    let total: f32 = pts.windows(2).map(|w| plen(psub(w[1], w[0]))).sum();
    if pts.is_empty() {
        return Point::default();
    }
    let mut left = total * t.clamp(0.0, 1.0);
    for w in pts.windows(2) {
        let l = plen(psub(w[1], w[0]));
        if l >= left && l > 0.0 {
            let t = left / l;
            return padd(w[0], pmul(psub(w[1], w[0]), t));
        }
        left -= l;
    }
    pts[pts.len() - 1]
}

/// Route an edge between its two nodes according to the diagram's brand `edge_routing`.
/// Orthogonal corners are rounded in proportion to the brand's corner radius.
pub fn edge_path(d: &Diagram, edge: &Edge) -> EdgePath {
    let radius = if crate::card::uses_cards(&d.brand) {
        crate::card::EDGE_RADIUS
    } else {
        (d.brand.corner_radius * 0.8).clamp(0.0, 14.0)
    };
    edge_path_with_radius(d, edge, radius)
}

/// Like `edge_path` with an explicit corner radius for elbow routing (0 = sharp corners).
pub fn edge_path_with_radius(d: &Diagram, edge: &Edge, radius: f32) -> EdgePath {
    let (Some(a), Some(b)) = (d.node(&edge.from), d.node(&edge.to)) else {
        return EdgePath::default();
    };
    let (ra, rb) = (node_rect(d, a), node_rect(d, b));
    let routing = d.brand.edge_routing;
    let pts = if std::ptr::eq(a, b) {
        self_loop(d, a, ra, routing)
    } else {
        match routing {
            EdgeRouting::Straight => straight(d, edge, a, ra, b, rb),
            EdgeRouting::Orthogonal | EdgeRouting::Curved => {
                elbow(d, edge, (a, ra), (b, rb), routing)
            }
        }
    };
    let pts = if routing == EdgeRouting::Orthogonal {
        round_corners(&simplify(pts), radius)
    } else {
        pts
    };
    let mut path = finish(pts);
    path.label_at = place_label(d, edge, &path.points, path.label_at);
    path
}

/// Slide a label along its edge until its chip is clear of group borders, group title strips and
/// nodes. Without groups (or with no clear spot) it stays at the middle.
fn place_label(d: &Diagram, edge: &Edge, pts: &[Point], mid: Point) -> Point {
    let groups = group_rects(d);
    if groups.is_empty() || pts.len() < 2 {
        return mid;
    }
    let Some((w, h)) = edge_label_size(d, edge) else {
        return mid;
    };
    let nodes: Vec<Rect> = d
        .nodes
        .iter()
        .filter(|n| n.pos.is_some())
        .map(|n| node_rect(d, n))
        .collect();
    let clear = |c: Point| {
        let r = Rect {
            x: c.x - w / 2.0,
            y: c.y - h / 2.0,
            w,
            h,
        };
        let m = GROUP_CLEAR / 2.0;
        groups.iter().all(|g| {
            let title = Rect {
                h: GROUP_TITLE,
                ..*g
            };
            let straddles = overlaps(r, g.inflate(m)) && !inside(r, g.inflate(-m));
            !straddles && !overlaps(r, title)
        }) && nodes.iter().all(|n| !overlaps(r, n.inflate(4.0)))
    };
    if clear(mid) {
        return mid;
    }
    (1..=16)
        .flat_map(|k| {
            let t = k as f32 * 0.025;
            [0.5 - t, 0.5 + t]
        })
        .map(|t| point_at(pts, t))
        .find(|p| clear(*p))
        .unwrap_or(mid)
}

/// Groups an edge from `a` to `b` must route around: those holding neither end.
fn foreign_groups(d: &Diagram, a: &Node, b: &Node) -> Vec<Rect> {
    d.groups
        .iter()
        .filter(|g| {
            a.group.as_deref() != Some(g.id.as_str()) && b.group.as_deref() != Some(g.id.as_str())
        })
        .filter_map(|g| group_rect(d, &g.id))
        .collect()
}

/// Where an elbow's cross segment runs, along the flow axis: halfway between `from` and `to`,
/// unless that line is within `GROUP_CLEAR` of a group border it would run along (or inside a
/// title strip); then the nearest clear line between the ends. `across` is the segment's extent.
fn cross_line(d: &Diagram, f: Frame, from: f32, to: f32, across: (f32, f32)) -> f32 {
    let mid = (from + to) / 2.0;
    let groups = group_rects(d);
    if groups.is_empty() {
        return mid;
    }
    let (s_lo, s_hi) = (across.0.min(across.1), across.0.max(across.1));
    let mut bad: Vec<(f32, f32)> = vec![];
    for g in &groups {
        let (plo, phi, slo, shi) = f.span(*g);
        if shi < s_lo || slo > s_hi {
            continue;
        }
        let title = if f.vertical { GROUP_TITLE } else { 0.0 };
        bad.push((plo - GROUP_CLEAR, plo + GROUP_CLEAR.max(title)));
        bad.push((phi - GROUP_CLEAR, phi + GROUP_CLEAR));
    }
    let (lo, hi) = (from.min(to) + 12.0, from.max(to) - 12.0);
    if lo > hi {
        return mid;
    }
    let ok = |v: f32| v >= lo && v <= hi && bad.iter().all(|&(a, b)| v <= a || v >= b);
    let mut cands: Vec<f32> = vec![mid];
    for &(a, b) in &bad {
        cands.push(a);
        cands.push(b);
        // the middle of the free space either side of a band
        cands.push((lo + a) / 2.0);
        cands.push((b + hi) / 2.0);
    }
    cands
        .into_iter()
        .filter(|v| ok(*v))
        .min_by(|a, b| (a - mid).abs().total_cmp(&(b - mid).abs()))
        .unwrap_or(mid)
}

fn straight(d: &Diagram, edge: &Edge, a: &Node, ra: Rect, b: &Node, rb: Rect) -> Vec<Point> {
    let (ca, cb) = (ra.center(), rb.center());
    let dir = unit(psub(cb, ca)).unwrap_or(Point::new(0.0, 1.0));
    let perp = Point::new(-dir.y, dir.x);
    let limit = (ra.w.min(ra.h).min(rb.w).min(rb.h) * 0.35).max(0.0);
    let off = pmul(perp, lane_offset(d, edge, limit));
    let p0 = boundary_hit(outline_shape(d, a), ra, padd(ca, off), dir);
    let p3 = boundary_hit(outline_shape(d, b), rb, padd(cb, off), pmul(dir, -1.0));
    vec![p0, p3]
}

/// Flow axis helper: `prim` runs along the flow, `sec` across it.
#[derive(Clone, Copy)]
struct Frame {
    vertical: bool,
}

impl Frame {
    fn pt(self, prim: f32, sec: f32) -> Point {
        if self.vertical {
            Point::new(sec, prim)
        } else {
            Point::new(prim, sec)
        }
    }

    fn prim(self, p: Point) -> f32 {
        if self.vertical { p.y } else { p.x }
    }

    fn sec(self, p: Point) -> f32 {
        if self.vertical { p.x } else { p.y }
    }

    /// `(prim_lo, prim_hi, sec_lo, sec_hi)` of a rectangle.
    fn span(self, r: Rect) -> (f32, f32, f32, f32) {
        if self.vertical {
            (r.y, r.y + r.h, r.x, r.x + r.w)
        } else {
            (r.x, r.x + r.w, r.y, r.y + r.h)
        }
    }
}

const DETOUR: f32 = 28.0;

/// True when any segment's bounding box comes within a few pixels of an obstacle.
fn hits(points: &[Point], obstacles: &[Rect]) -> bool {
    const PAD: f32 = 4.0;
    points.windows(2).any(|w| {
        let (x0, x1) = (w[0].x.min(w[1].x), w[0].x.max(w[1].x));
        let (y0, y1) = (w[0].y.min(w[1].y), w[0].y.max(w[1].y));
        obstacles.iter().any(|o| {
            x1 > o.x - PAD && x0 < o.x + o.w + PAD && y1 > o.y - PAD && y0 < o.y + o.h + PAD
        })
    })
}

/// Around the nodes that share a row (or column) with `r`: how far they reach in direction `dir`.
fn band_reach(f: Frame, r: Rect, all: &[Rect], dir: f32) -> f32 {
    let (lo, hi, _, _) = f.span(r);
    let mut reach = if dir > 0.0 { hi } else { lo };
    for o in all {
        let (olo, ohi, _, _) = f.span(*o);
        if ohi > lo && olo < hi {
            reach = if dir > 0.0 {
                reach.max(ohi)
            } else {
                reach.min(olo)
            };
        }
    }
    reach
}

/// Re-route a forward elbow through a free channel when the direct route crosses other nodes.
fn channel_route(
    f: Frame,
    ends: (Point, Side, Rect),
    to: (Point, Side, Rect),
    obstacles: &[Rect],
) -> Option<Vec<Point>> {
    let ((pa, sa, ra), (pb, sb, rb)) = (ends, to);
    let mut all: Vec<Rect> = obstacles.to_vec();
    all.extend([ra, rb]);
    let (s, t) = (f.prim(sa.normal()), f.prim(sb.normal()));
    let m1 = band_reach(f, ra, &all, s) + 18.0 * s;
    let m2 = band_reach(f, rb, &all, t) + 18.0 * t;
    if (m2 - m1) * s <= 8.0 {
        return None;
    }
    let (lo, hi) = (m1.min(m2), m1.max(m2));
    let mid = (f.sec(pa) + f.sec(pb)) / 2.0;
    let mut xs: Vec<f32> = vec![mid];
    for o in obstacles {
        let (plo, phi, slo, shi) = f.span(*o);
        if phi > lo && plo < hi {
            xs.push(shi + 18.0);
            xs.push(slo - 18.0);
        }
    }
    xs.sort_by(|a, b| {
        (a - mid)
            .abs()
            .total_cmp(&(b - mid).abs())
            .then(a.total_cmp(b))
    });
    xs.dedup();
    for x in xs.into_iter().take(16) {
        let route = vec![
            pa,
            f.pt(m1, f.sec(pa)),
            f.pt(m1, x),
            f.pt(m2, x),
            f.pt(m2, f.sec(pb)),
            pb,
        ];
        if !hits(&route, obstacles) {
            return Some(route);
        }
    }
    None
}

/// Route a backward edge around the outside of everything in its span.
fn back_route(
    f: Frame,
    d: &Diagram,
    a: (&Node, Rect),
    b: (&Node, Rect),
    curved: bool,
) -> Vec<Point> {
    let ((na, ra), (nb, rb)) = (a, b);
    let (lo, hi) = {
        let (a0, a1, _, _) = f.span(ra);
        let (b0, b1, _, _) = f.span(rb);
        (a0.min(b0), a1.max(b1))
    };
    let (mut smin, mut smax) = (f32::MAX, f32::MIN);
    for r in d
        .nodes
        .iter()
        .filter(|n| n.pos.is_some())
        .map(|n| node_rect(d, n))
        .chain(group_rects(d))
    {
        let (plo, phi, slo, shi) = f.span(r);
        if phi > lo && plo < hi {
            smin = smin.min(slo);
            smax = smax.max(shi);
        }
    }
    let (ca, cb) = (ra.center(), rb.center());
    let mid = (f.sec(ca) + f.sec(cb)) / 2.0;
    let (x_pos, x_neg) = (smax + DETOUR, smin - DETOUR);
    let positive = (x_pos - mid).abs() <= (x_neg - mid).abs();
    let x = if positive { x_pos } else { x_neg };
    let side = match (f.vertical, positive) {
        (true, true) => Side::Right,
        (true, false) => Side::Left,
        (false, true) => Side::Bottom,
        (false, false) => Side::Top,
    };
    let pa = boundary_hit(outline_shape(d, na), ra, ca, side.normal());
    let pb = boundary_hit(outline_shape(d, nb), rb, cb, side.normal());
    if curved {
        let xc = x + (x - (f.sec(pa) + f.sec(pb)) / 2.0) / 3.0;
        cubic(pa, f.pt(f.prim(pa), xc), f.pt(f.prim(pb), xc), pb)
    } else {
        vec![pa, f.pt(f.prim(pa), x), f.pt(f.prim(pb), x), pb]
    }
}

fn elbow(
    d: &Diagram,
    edge: &Edge,
    a: (&Node, Rect),
    b: (&Node, Rect),
    routing: EdgeRouting,
) -> Vec<Point> {
    let ((na, ra), (nb, rb)) = (a, b);
    let Some((sa, sb, kind)) = pick_sides(d.direction, ra, rb) else {
        return straight(d, edge, na, ra, nb, rb);
    };
    let frame = Frame {
        vertical: matches!(d.direction, Direction::Down | Direction::Up),
    };
    if kind == Kind::Backward {
        return back_route(frame, d, a, b, routing == EdgeRouting::Curved);
    }
    let lateral = if sa.vertical() {
        ra.w.min(rb.w)
    } else {
        ra.h.min(rb.h)
    };
    let off = lane_offset(d, edge, (lateral / 2.0 - 8.0).max(0.0));
    let (ca, cb) = (ra.center(), rb.center());
    let (mut oa, mut ob) = if sa.vertical() {
        (Point::new(ca.x + off, ca.y), Point::new(cb.x + off, cb.y))
    } else {
        (Point::new(ca.x, ca.y + off), Point::new(cb.x, cb.y + off))
    };
    // Nearly aligned: share one line instead of a pointless jog.
    if sa.vertical() && (oa.x - ob.x).abs() < 3.0 {
        let m = (oa.x + ob.x) / 2.0;
        (oa.x, ob.x) = (m, m);
    } else if !sa.vertical() && (oa.y - ob.y).abs() < 3.0 {
        let m = (oa.y + ob.y) / 2.0;
        (oa.y, ob.y) = (m, m);
    }
    let p0 = boundary_hit(outline_shape(d, na), ra, oa, sa.normal());
    let p3 = boundary_hit(outline_shape(d, nb), rb, ob, sb.normal());
    let aligned = if sa.vertical() {
        (p0.x - p3.x).abs() < 0.01
    } else {
        (p0.y - p3.y).abs() < 0.01
    };
    let direct = if aligned {
        vec![p0, p3]
    } else {
        match routing {
            EdgeRouting::Curved => {
                let span = if sa.vertical() {
                    (p3.y - p0.y).abs()
                } else {
                    (p3.x - p0.x).abs()
                };
                let k = (span * 0.5).clamp(24.0, 140.0);
                cubic(
                    p0,
                    padd(p0, pmul(sa.normal(), k)),
                    padd(p3, pmul(sb.normal(), k)),
                    p3,
                )
            }
            _ if sa.vertical() => {
                let f = Frame { vertical: true };
                let m = cross_line(d, f, p0.y, p3.y, (p0.x, p3.x));
                vec![p0, Point::new(p0.x, m), Point::new(p3.x, m), p3]
            }
            _ => {
                let f = Frame { vertical: false };
                let m = cross_line(d, f, p0.x, p3.x, (p0.y, p3.y));
                vec![p0, Point::new(m, p0.y), Point::new(m, p3.y), p3]
            }
        }
    };
    if kind != Kind::Forward {
        return direct;
    }
    // Other nodes, and groups holding neither end (grown so a route keeps GROUP_CLEAR from them;
    // `hits` adds 4 of its own).
    let obstacles: Vec<Rect> = d
        .nodes
        .iter()
        .filter(|n| n.pos.is_some() && !std::ptr::eq(*n, na) && !std::ptr::eq(*n, nb))
        .map(|n| node_rect(d, n))
        .chain(
            foreign_groups(d, na, nb)
                .into_iter()
                .map(|g| g.inflate(GROUP_CLEAR - 4.0)),
        )
        .collect();
    if !hits(&direct, &obstacles) {
        return direct;
    }
    match channel_route(frame, (p0, sa, ra), (p3, sb, rb), &obstacles) {
        Some(route) if routing == EdgeRouting::Curved => round_corners(&simplify(route), 30.0),
        Some(route) => route,
        None => direct,
    }
}

fn self_loop(d: &Diagram, node: &Node, r: Rect, routing: EdgeRouting) -> Vec<Point> {
    let vertical_flow = matches!(d.direction, Direction::Down | Direction::Up);
    // Loop out of the side the layout does not use for flow.
    let (n, t, extent) = if vertical_flow {
        (Point::new(1.0, 0.0), Point::new(0.0, 1.0), r.h)
    } else {
        (Point::new(0.0, 1.0), Point::new(1.0, 0.0), r.w)
    };
    let q = (extent * 0.2).clamp(6.0, 14.0);
    let c = r.center();
    let p0 = boundary_hit(outline_shape(d, node), r, psub(c, pmul(t, q)), n);
    let p3 = boundary_hit(outline_shape(d, node), r, padd(c, pmul(t, q)), n);
    match routing {
        EdgeRouting::Curved => {
            let out = pmul(n, SELF_LOOP * 1.4);
            let spread = pmul(t, q * 1.2);
            cubic(
                p0,
                psub(padd(p0, out), spread),
                padd(padd(p3, out), spread),
                p3,
            )
        }
        _ => {
            let out = pmul(n, SELF_LOOP);
            vec![p0, padd(p0, out), padd(p3, out), p3]
        }
    }
}

/// Distance from `p` to the nearest point on the edge's polyline.
pub fn distance_to_path(path: &EdgePath, p: Point) -> f32 {
    let mut best = f32::MAX;
    for w in path.points.windows(2) {
        let (a, b) = (w[0], w[1]);
        let (dx, dy) = (b.x - a.x, b.y - a.y);
        let len2 = dx * dx + dy * dy;
        let t = if len2 == 0.0 {
            0.0
        } else {
            (((p.x - a.x) * dx + (p.y - a.y) * dy) / len2).clamp(0.0, 1.0)
        };
        let (cx, cy) = (a.x + t * dx, a.y + t * dy);
        best = best.min(((p.x - cx).powi(2) + (p.y - cy).powi(2)).sqrt());
    }
    best
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    use crate::brand::EdgeRouting;

    fn placed(id: &str, label: &str, x: f32, y: f32) -> Node {
        let mut n = Node::new(id, label);
        n.pos = Some(Point::new(x, y));
        n
    }

    fn two(routing: EdgeRouting, dir: Direction, pa: (f32, f32), pb: (f32, f32)) -> Diagram {
        let mut d = Diagram::default();
        // Boundary tests exercise the classic outline shapes.
        d.brand.node_style = crate::brand::NodeStyle::Shape;
        d.brand.edge_routing = routing;
        d.direction = dir;
        d.nodes.push(placed("a", "A", pa.0, pa.1));
        d.nodes.push(placed("b", "B", pb.0, pb.1));
        d.add_edge("a", "b");
        d
    }

    fn strictly_inside(r: Rect, p: Point) -> bool {
        p.x > r.x + 0.5 && p.x < r.x + r.w - 0.5 && p.y > r.y + 0.5 && p.y < r.y + r.h - 0.5
    }

    #[test]
    fn size_grows_with_label_and_honours_override() {
        let d = Diagram::default();
        let short = node_size(&d, &Node::new("a", "Hi"));
        let long = node_size(
            &d,
            &Node::new("a", "A considerably longer label than before"),
        );
        assert!(long.w > short.w);
        assert!(short.w >= 100.0 && short.h >= 48.0);
        let multi = node_size(&d, &Node::new("a", "one\ntwo\nthree"));
        assert!(multi.h > short.h);
        let mut n = Node::new("a", "x");
        n.size = Some(Size { w: 300.0, h: 90.0 });
        assert_eq!(node_size(&d, &n), Size { w: 300.0, h: 90.0 });
    }

    #[test]
    fn shapes_get_their_own_minimums() {
        let mut d = Diagram::default();
        d.brand.node_style = crate::brand::NodeStyle::Shape;
        let mut n = Node::new("a", "Is it valid?");
        let rect = node_size(&d, &n);
        n.shape = Shape::Diamond;
        let dia = node_size(&d, &n);
        assert!(dia.w > rect.w && dia.h > rect.h);
        n.shape = Shape::Screen;
        let scr = node_size(&d, &n);
        assert!(scr.h >= 220.0 && scr.w >= 120.0 && scr.h > scr.w);
        n.shape = Shape::Circle;
        let c = node_size(&d, &n);
        assert_eq!(c.w, c.h);
        for s in Shape::ALL {
            n.shape = s;
            let sz = node_size(&d, &n);
            assert!(sz.w > 0.0 && sz.h > 0.0, "{s:?}");
        }
        let plain = node_size(&d, &Node::new("a", "Label"));
        let mut with_icon = Node::new("a", "Label");
        with_icon.icon = Some("user".into());
        // Every Shape-style node already keeps room for its glyph (icon, kind icon or dot).
        assert_eq!(node_size(&d, &with_icon).w, plain.w);
        let wide = Node::new("a", "A fairly long label that needs space");
        assert!(node_size(&d, &wide).w >= text_width(&wide.label, d.brand.font_size) + ICON_W);
    }

    #[test]
    fn group_rect_pads_and_adds_title_strip() {
        let mut d = Diagram::default();
        let mut a = placed("a", "A", 100.0, 100.0);
        a.group = Some("g".into());
        let mut b = placed("b", "B", 300.0, 200.0);
        b.group = Some("g".into());
        d.nodes.push(a);
        d.nodes.push(b);
        d.nodes.push(Node::new("c", "loose"));
        let ra = node_rect(&d, &d.nodes[0]);
        let rb = node_rect(&d, &d.nodes[1]);
        let g = group_rect(&d, "g").unwrap();
        assert_eq!(g.x, 100.0 - GROUP_PAD);
        assert_eq!(g.y, 100.0 - GROUP_PAD - GROUP_TITLE);
        assert_eq!(g.x + g.w, rb.x + rb.w + GROUP_PAD);
        assert_eq!(g.y + g.h, rb.y + rb.h + GROUP_PAD);
        assert!(g.contains(ra.center()));
        assert!(group_rect(&d, "nope").is_none());
        d.nodes[0].pos = None;
        d.nodes[1].pos = None;
        assert!(group_rect(&d, "g").is_none());
    }

    #[test]
    fn bounds_cover_nodes_groups_and_margin() {
        let mut d = Diagram::default();
        let e = diagram_bounds(&d, 10.0);
        assert!(e.w > 0.0 && e.h > 0.0);
        let mut a = placed("a", "A", 0.0, 0.0);
        a.group = Some("g".into());
        d.nodes.push(a);
        d.groups.push(crate::model::Group {
            id: "g".into(),
            label: "G".into(),
            role: crate::brand::Role::Neutral,
        });
        let r = node_rect(&d, &d.nodes[0]);
        let b = diagram_bounds(&d, 10.0);
        let g = group_rect(&d, "g").unwrap();
        assert!(b.x <= g.x - 10.0 + 0.01 && b.y <= g.y - 10.0 + 0.01);
        assert!(b.x + b.w >= r.x + r.w + 10.0 - 0.01);
    }

    #[test]
    fn straight_edge_anchors_on_rect_borders() {
        let d = two(
            EdgeRouting::Straight,
            Direction::Down,
            (0.0, 0.0),
            (0.0, 200.0),
        );
        let p = edge_path(&d, &d.edges[0]);
        let (ra, rb) = (node_rect(&d, &d.nodes[0]), node_rect(&d, &d.nodes[1]));
        assert_eq!(p.points.len(), 2);
        assert!((p.points[0].y - (ra.y + ra.h)).abs() < 0.01);
        assert!((p.points[1].y - rb.y).abs() < 0.01);
        assert!((p.end_dir.y - 1.0).abs() < 1e-4 && p.end_dir.x.abs() < 1e-4);
        assert!((p.start_dir.y + 1.0).abs() < 1e-4);
    }

    #[test]
    fn diamond_and_circle_anchor_on_true_boundary() {
        let mut d = two(
            EdgeRouting::Straight,
            Direction::Down,
            (0.0, 0.0),
            (300.0, 200.0),
        );
        d.nodes[0].shape = Shape::Diamond;
        d.nodes[1].shape = Shape::Circle;
        let p = edge_path(&d, &d.edges[0]);
        let ra = node_rect(&d, &d.nodes[0]);
        let c = ra.center();
        let k =
            (p.points[0].x - c.x).abs() / (ra.w / 2.0) + (p.points[0].y - c.y).abs() / (ra.h / 2.0);
        assert!((k - 1.0).abs() < 1e-3, "{k}");
        let rb = node_rect(&d, &d.nodes[1]);
        let cb = rb.center();
        let e = p.points[1];
        let k = ((e.x - cb.x) / (rb.w / 2.0)).powi(2) + ((e.y - cb.y) / (rb.h / 2.0)).powi(2);
        assert!((k - 1.0).abs() < 1e-3, "{k}");
    }

    #[test]
    fn hexagon_and_parallelogram_hit_polygon_edges() {
        for shape in [Shape::Hexagon, Shape::Parallelogram] {
            let mut d = two(
                EdgeRouting::Straight,
                Direction::Down,
                (0.0, 0.0),
                (400.0, 40.0),
            );
            d.nodes[0].shape = shape;
            let p = edge_path(&d, &d.edges[0]);
            let ra = node_rect(&d, &d.nodes[0]);
            let poly = shape_polygon(shape, ra).unwrap();
            let q = p.points[0];
            let on_edge = (0..poly.len()).any(|i| {
                let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
                let ab = psub(b, a);
                let t = (((q.x - a.x) * ab.x + (q.y - a.y) * ab.y) / (ab.x * ab.x + ab.y * ab.y))
                    .clamp(0.0, 1.0);
                plen(psub(q, padd(a, pmul(ab, t)))) < 0.05
            });
            assert!(on_edge, "{shape:?}");
        }
    }

    #[test]
    fn orthogonal_is_axis_aligned_and_leaves_along_layout_axis() {
        for (dir, pb, vertical) in [
            (Direction::Down, (200.0, 200.0), true),
            (Direction::Up, (200.0, -200.0), true),
            (Direction::Right, (300.0, 100.0), false),
            (Direction::Left, (-300.0, 100.0), false),
        ] {
            let d = two(EdgeRouting::Orthogonal, dir, (0.0, 0.0), pb);
            let p = edge_path_with_radius(&d, &d.edges[0], 0.0);
            assert!(p.points.len() >= 3, "{dir:?}");
            for w in p.points.windows(2) {
                assert!(w[0].x == w[1].x || w[0].y == w[1].y, "{dir:?} {w:?}");
            }
            let first = psub(p.points[1], p.points[0]);
            if vertical {
                assert!(first.x == 0.0 && first.y != 0.0, "{dir:?}");
            } else {
                assert!(first.y == 0.0 && first.x != 0.0, "{dir:?}");
            }
            let (ra, rb) = (node_rect(&d, &d.nodes[0]), node_rect(&d, &d.nodes[1]));
            for w in p.points.windows(2) {
                let m = Point::new((w[0].x + w[1].x) / 2.0, (w[0].y + w[1].y) / 2.0);
                assert!(
                    !strictly_inside(ra, m) && !strictly_inside(rb, m),
                    "{dir:?}"
                );
            }
            // Ends sit on the borders.
            assert!(!strictly_inside(ra, p.points[0]) && ra.inflate(0.01).contains(p.points[0]));
            assert!(rb.inflate(0.01).contains(*p.points.last().unwrap()));
        }
    }

    #[test]
    fn aligned_nodes_get_a_straight_line() {
        let d = two(
            EdgeRouting::Orthogonal,
            Direction::Down,
            (0.0, 0.0),
            (0.0, 200.0),
        );
        let p = edge_path(&d, &d.edges[0]);
        assert_eq!(p.points.len(), 2);
    }

    #[test]
    fn rounded_corners_stay_close_to_the_elbow() {
        let d = two(
            EdgeRouting::Orthogonal,
            Direction::Down,
            (0.0, 0.0),
            (200.0, 200.0),
        );
        let sharp = edge_path_with_radius(&d, &d.edges[0], 0.0);
        let round = edge_path_with_radius(&d, &d.edges[0], 12.0);
        assert!(round.points.len() > sharp.points.len());
        assert_eq!(round.points[0], sharp.points[0]);
        assert_eq!(round.points.last(), sharp.points.last());
    }

    #[test]
    fn curved_is_sampled_and_ends_on_borders() {
        let d = two(
            EdgeRouting::Curved,
            Direction::Down,
            (0.0, 0.0),
            (240.0, 220.0),
        );
        let p = edge_path(&d, &d.edges[0]);
        assert_eq!(p.points.len(), CURVE_SAMPLES + 1);
        let (ra, rb) = (node_rect(&d, &d.nodes[0]), node_rect(&d, &d.nodes[1]));
        assert!((p.points[0].y - (ra.y + ra.h)).abs() < 0.01);
        assert!((p.points[CURVE_SAMPLES].y - rb.y).abs() < 0.01);
        assert!((plen(p.end_dir) - 1.0).abs() < 1e-3 && p.end_dir.y > 0.5);
    }

    #[test]
    fn self_loop_is_a_visible_loop_for_every_routing() {
        for routing in [
            EdgeRouting::Straight,
            EdgeRouting::Orthogonal,
            EdgeRouting::Curved,
        ] {
            for dir in [Direction::Down, Direction::Right] {
                let mut d = Diagram::default();
                d.brand.edge_routing = routing;
                d.direction = dir;
                d.nodes.push(placed("a", "Loop", 50.0, 50.0));
                d.add_edge("a", "a");
                let p = edge_path(&d, &d.edges[0]);
                let r = node_rect(&d, &d.nodes[0]);
                assert!(p.points.len() >= 4, "{routing:?}");
                let out_x = p
                    .points
                    .iter()
                    .map(|q| q.x - (r.x + r.w))
                    .fold(f32::MIN, f32::max);
                let out_y = p
                    .points
                    .iter()
                    .map(|q| q.y - (r.y + r.h))
                    .fold(f32::MIN, f32::max);
                assert!(out_x.max(out_y) > 24.0, "{routing:?} {dir:?}");
                assert!(plen(psub(p.points[0], *p.points.last().unwrap())) > 4.0);
                assert!(!strictly_inside(r, p.label_at));
            }
        }
    }

    #[test]
    fn back_edges_loop_around_the_outside() {
        for routing in [EdgeRouting::Orthogonal, EdgeRouting::Curved] {
            for (dir, pa, pb) in [
                (Direction::Down, (0.0, 300.0), (0.0, 0.0)),
                (Direction::Right, (400.0, 0.0), (0.0, 0.0)),
            ] {
                let mut d = two(routing, dir, pa, pb);
                d.nodes.push(placed("mid", "In the way", 20.0, 150.0));
                d.nodes[2].pos = Some(if dir == Direction::Down {
                    Point::new(20.0, 150.0)
                } else {
                    Point::new(200.0, 0.0)
                });
                let p = edge_path_with_radius(&d, &d.edges[0], 0.0);
                let rects: Vec<Rect> = d.nodes.iter().map(|n| node_rect(&d, n)).collect();
                let along = |q: Point| if dir == Direction::Down { q.x } else { q.y };
                let (reach_hi, reach_lo) = (
                    p.points.iter().map(|q| along(*q)).fold(f32::MIN, f32::max),
                    p.points.iter().map(|q| along(*q)).fold(f32::MAX, f32::min),
                );
                let (far_hi, far_lo) = if dir == Direction::Down {
                    (
                        rects.iter().map(|r| r.x + r.w).fold(f32::MIN, f32::max),
                        rects.iter().map(|r| r.x).fold(f32::MAX, f32::min),
                    )
                } else {
                    (
                        rects.iter().map(|r| r.y + r.h).fold(f32::MIN, f32::max),
                        rects.iter().map(|r| r.y).fold(f32::MAX, f32::min),
                    )
                };
                assert!(
                    reach_hi > far_hi + 10.0 || reach_lo < far_lo - 10.0,
                    "{routing:?} {dir:?}"
                );
                for q in &p.points {
                    assert!(
                        !rects.iter().any(|r| strictly_inside(*r, *q)),
                        "{routing:?} {dir:?}"
                    );
                }
                assert!(
                    !hits(&p.points, &rects[2..]),
                    "{routing:?} {dir:?} passes through the node in the way"
                );
            }
        }
    }

    #[test]
    fn forward_edges_detour_around_nodes_in_the_way() {
        for routing in [EdgeRouting::Orthogonal, EdgeRouting::Curved] {
            let mut d = two(routing, Direction::Down, (0.0, 0.0), (0.0, 400.0));
            d.nodes.push(placed("mid", "Blocker", 0.0, 190.0));
            let p = edge_path_with_radius(&d, &d.edges[0], 0.0);
            let blocker = node_rect(&d, &d.nodes[2]);
            assert!(!hits(&p.points, &[blocker]), "{routing:?}");
            let (ra, rb) = (node_rect(&d, &d.nodes[0]), node_rect(&d, &d.nodes[1]));
            assert!(
                ra.inflate(0.01).contains(p.points[0])
                    && rb.inflate(0.01).contains(*p.points.last().unwrap())
            );
            assert!(p.end_dir.y > 0.5, "still arrives from above");
        }
    }

    #[test]
    fn parallel_edges_do_not_overlap() {
        let mut d = two(
            EdgeRouting::Orthogonal,
            Direction::Down,
            (0.0, 0.0),
            (0.0, 200.0),
        );
        d.add_edge("b", "a");
        let p1 = edge_path(&d, &d.edges[0]);
        let p2 = edge_path(&d, &d.edges[1]);
        assert!((p1.points[0].x - p2.points[0].x).abs() > 8.0);
    }

    #[test]
    fn label_at_is_midpoint_by_length() {
        let mut p = finish(vec![
            Point::new(0.0, 0.0),
            Point::new(10.0, 0.0),
            Point::new(10.0, 30.0),
        ]);
        assert!((p.label_at.x - 10.0).abs() < 1e-4 && (p.label_at.y - 10.0).abs() < 1e-4);
        p = finish(vec![Point::new(0.0, 0.0), Point::new(100.0, 0.0)]);
        assert_eq!(p.label_at, Point::new(50.0, 0.0));
        assert_eq!(finish(vec![]).points.len(), 0);
    }

    #[test]
    fn missing_endpoint_gives_empty_path() {
        let mut d = two(
            EdgeRouting::Straight,
            Direction::Down,
            (0.0, 0.0),
            (0.0, 200.0),
        );
        d.edges[0].to = "zzz".into();
        assert!(edge_path(&d, &d.edges[0]).points.is_empty());
    }

    #[test]
    fn distance_to_path_works() {
        let p = finish(vec![Point::new(0.0, 0.0), Point::new(100.0, 0.0)]);
        assert!((distance_to_path(&p, Point::new(50.0, 7.0)) - 7.0).abs() < 1e-4);
    }

    /// The release checklist shape: a group in the middle of a downward flow, with edges that
    /// enter it, leave it and pass it by.
    fn grouped_flow() -> Diagram {
        let mut d = Diagram::default();
        d.direction = Direction::Down;
        d.groups.push(crate::model::Group {
            id: "g".into(),
            label: "Quality".into(),
            role: crate::brand::Role::Secondary,
        });
        for (id, label) in [
            ("start", "Feature ready"),
            ("review", "Code review"),
            ("approved", "Approved?"),
            ("fix", "Address feedback"),
            ("tests", "Run test suite"),
            ("passes", "All green?"),
            ("notes", "Write release notes"),
            ("ship", "Ship it"),
        ] {
            d.ensure_node(id, label);
        }
        d.node_mut("tests").unwrap().group = Some("g".into());
        d.node_mut("passes").unwrap().group = Some("g".into());
        for (a, b, l) in [
            ("start", "review", ""),
            ("review", "approved", ""),
            ("approved", "fix", "No"),
            ("fix", "review", "Re-review"),
            ("approved", "tests", "Yes"),
            ("tests", "passes", ""),
            ("passes", "notes", "Yes"),
            ("passes", "fix", "No"),
            ("notes", "ship", ""),
        ] {
            d.add_edge(a, b).label = l.into();
        }
        crate::layout::auto_layout(&mut d);
        d
    }

    #[test]
    fn edges_keep_clear_of_group_borders_they_run_along() {
        let d = grouped_flow();
        let g = group_rect(&d, "g").unwrap();
        for e in &d.edges {
            let p = edge_path(&d, e);
            for w in p.points.windows(2) {
                let (a, b) = (w[0], w[1]);
                let horizontal = (a.y - b.y).abs() < 0.01 && (a.x - b.x).abs() > 4.0;
                let vertical = (a.x - b.x).abs() < 0.01 && (a.y - b.y).abs() > 4.0;
                if horizontal && a.x.max(b.x) > g.x && a.x.min(b.x) < g.x + g.w {
                    for line in [g.y, g.y + g.h] {
                        assert!(
                            (a.y - line).abs() >= GROUP_CLEAR - 0.5,
                            "{} runs {:.1} from a horizontal group border",
                            e.id,
                            (a.y - line).abs()
                        );
                    }
                }
                if vertical && a.y.max(b.y) > g.y && a.y.min(b.y) < g.y + g.h {
                    for line in [g.x, g.x + g.w] {
                        assert!(
                            (a.x - line).abs() >= GROUP_CLEAR - 0.5,
                            "{} runs {:.1} from a vertical group border",
                            e.id,
                            (a.x - line).abs()
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn label_chips_stay_off_group_borders_and_titles() {
        let d = grouped_flow();
        let g = group_rect(&d, "g").unwrap();
        let title = Rect {
            h: GROUP_TITLE,
            ..g
        };
        for e in d.edges.iter().filter(|e| !e.label.is_empty()) {
            let r = edge_label_rect(&d, e, &edge_path(&d, e)).unwrap();
            assert!(!overlaps(r, title), "{} label on the title strip", e.id);
            let straddles = overlaps(r, g) && !inside(r, g);
            assert!(!straddles, "{} label on the group border", e.id);
            assert!(
                (r.h - crate::card::LABEL_H).abs() < 0.01,
                "cards use 20px chips"
            );
        }
    }
}
