//! Making a step from a handle, mirroring the original app's quickCreate.ts, connectHandles.ts and
//! v2PointerCore.ts: four plus handles 22px outside each side, a new step one full node size past
//! the side (or centred where a drag was released), a decision leading to a plain step.
use rovelo_core::geometry;
use rovelo_core::model::{Diagram, Id, Node, Point, Rect, Shape, Size};

/// Handle diameter in screen pixels (not zoomed).
pub const HANDLE_PX: f32 = 16.0;
/// Screen-pixel gap between a node's edge and its handle's centre.
pub const OFFSET_PX: f32 = 22.0;
/// Screen-pixel pick radius around a handle.
pub const PICK_PX: f32 = 9.0;
/// Below this many screen pixels a press-release is a click, not a drag.
pub const CLICK_PX: f32 = 4.0;
/// Smallest box a create drag makes.
pub const MIN_CREATE: f32 = 8.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Top,
    Right,
    Bottom,
    Left,
}

impl Side {
    pub const ALL: [Side; 4] = [Side::Top, Side::Right, Side::Bottom, Side::Left];

    pub fn opposite(self) -> Side {
        match self {
            Side::Top => Side::Bottom,
            Side::Right => Side::Left,
            Side::Bottom => Side::Top,
            Side::Left => Side::Right,
        }
    }
}

/// Midpoint of a side, on the edge.
pub fn side_anchor(r: Rect, side: Side) -> Point {
    let (cx, cy) = (r.x + r.w / 2.0, r.y + r.h / 2.0);
    match side {
        Side::Top => Point::new(cx, r.y),
        Side::Right => Point::new(r.x + r.w, cy),
        Side::Bottom => Point::new(cx, r.y + r.h),
        Side::Left => Point::new(r.x, cy),
    }
}

/// The four handle centres (world), each `OFFSET_PX` screen pixels outside its side.
pub fn handle_points(r: Rect, zoom: f32) -> [(Side, Point); 4] {
    let off = OFFSET_PX / zoom.max(1e-3);
    Side::ALL.map(|s| {
        let a = side_anchor(r, s);
        let p = match s {
            Side::Top => Point::new(a.x, a.y - off),
            Side::Right => Point::new(a.x + off, a.y),
            Side::Bottom => Point::new(a.x, a.y + off),
            Side::Left => Point::new(a.x - off, a.y),
        };
        (s, p)
    })
}

/// The handle under a world point, if any.
pub fn pick_handle(r: Rect, world: Point, zoom: f32) -> Option<Side> {
    let radius = PICK_PX / zoom.max(1e-3);
    handle_points(r, zoom)
        .into_iter()
        .find(|(_, p)| (p.x - world.x).hypot(p.y - world.y) <= radius)
        .map(|(s, _)| s)
}

/// Where a same-size copy goes: one full node size past `side` (origin plus twice the extent).
pub fn origin(r: Rect, side: Side) -> Point {
    match side {
        Side::Right => Point::new(r.x + r.w * 2.0, r.y),
        Side::Left => Point::new(r.x - r.w * 2.0, r.y),
        Side::Bottom => Point::new(r.x, r.y + r.h * 2.0),
        Side::Top => Point::new(r.x, r.y - r.h * 2.0),
    }
}

/// The ghost a hovered handle previews: the source's size at `origin`.
pub fn ghost(r: Rect, side: Side) -> Rect {
    let o = origin(r, side);
    Rect {
        x: o.x,
        y: o.y,
        w: r.w,
        h: r.h,
    }
}

fn is_decision(n: &Node) -> bool {
    n.shape == Shape::Diamond
}

/// The default step: a blank rounded node at its measured size.
pub fn default_size(d: &Diagram) -> Size {
    let mut n = Node::new("probe", "");
    n.shape = Shape::Rounded;
    geometry::node_size(d, &n)
}

/// Plan the new step for a handle on `source`: same kind, size and look with a blank label
/// (a decision makes a plain step), centred on `drop` when a drag was released there.
pub fn plan(d: &Diagram, source: &Node, side: Side, new_id: Id, drop: Option<Point>) -> Node {
    let r = geometry::node_rect(d, source);
    let decision = is_decision(source);
    let size = if decision {
        default_size(d)
    } else {
        Size { w: r.w, h: r.h }
    };
    let o = origin(r, side);
    let centre = drop.unwrap_or(Point::new(o.x + r.w / 2.0, o.y + r.h / 2.0));
    let mut n = Node::new(new_id, "");
    if decision {
        n.shape = Shape::Rounded;
        n.role = rovelo_core::brand::Role::Primary;
    } else {
        n.shape = source.shape;
        n.role = source.role;
        n.icon = source.icon.clone();
        n.size = Some(size);
    }
    n.group = source.group.clone();
    n.pos = Some(Point::new(centre.x - size.w / 2.0, centre.y - size.h / 2.0));
    n
}

/// The box a create drag drew from `start` to `world` (Shift: square), at least `MIN_CREATE`.
pub fn drag_box(start: Point, world: Point, square: bool) -> Rect {
    let dx = (world.x - start.x).abs();
    let dy = (world.y - start.y).abs();
    let w = MIN_CREATE.max(if square { dx.max(dy) } else { dx });
    let h = MIN_CREATE.max(if square { dx.max(dy) } else { dy });
    Rect {
        x: if world.x < start.x {
            start.x - w
        } else {
            start.x
        },
        y: if world.y < start.y {
            start.y - h
        } else {
            start.y
        },
        w,
        h,
    }
}

/// The least pan (screen pixels) that brings `r` fully inside `free`, zoom kept.
pub fn pan_to_uncover(r: egui::Rect, free: egui::Rect) -> egui::Vec2 {
    // Too big to fit, or past the near edge: align the near edges. Past the far edge: align the far ones.
    let axis = |lo: f32, hi: f32, flo: f32, fhi: f32| -> f32 {
        if hi - lo > fhi - flo || lo < flo {
            flo - lo
        } else if hi > fhi {
            fhi - hi
        } else {
            0.0
        }
    };
    egui::vec2(
        axis(r.left(), r.right(), free.left(), free.right()),
        axis(r.top(), r.bottom(), free.top(), free.bottom()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect() -> Rect {
        Rect {
            x: 100.0,
            y: 50.0,
            w: 160.0,
            h: 48.0,
        }
    }

    #[test]
    fn origin_per_side_is_one_full_size_away() {
        let r = rect();
        assert_eq!(origin(r, Side::Right), Point::new(420.0, 50.0));
        assert_eq!(origin(r, Side::Left), Point::new(-220.0, 50.0));
        assert_eq!(origin(r, Side::Bottom), Point::new(100.0, 146.0));
        assert_eq!(origin(r, Side::Top), Point::new(100.0, -46.0));
        assert_eq!(Side::Top.opposite(), Side::Bottom);
    }

    #[test]
    fn handles_sit_22px_outside_and_pick_within_9px() {
        let r = rect();
        let pts = handle_points(r, 2.0);
        assert_eq!(pts[1], (Side::Right, Point::new(260.0 + 11.0, 74.0)));
        assert_eq!(
            pick_handle(r, Point::new(271.0, 78.0), 2.0),
            Some(Side::Right)
        );
        assert_eq!(pick_handle(r, Point::new(271.0, 80.0), 2.0), None);
        assert_eq!(
            pick_handle(r, Point::new(180.0, 50.0 - 22.0), 1.0),
            Some(Side::Top)
        );
    }

    #[test]
    fn a_decision_leads_to_a_plain_step() {
        let mut d = Diagram::default();
        let q = d.ensure_node("q", "Ok?");
        q.shape = Shape::Diamond;
        q.pos = Some(Point::new(0.0, 0.0));
        let src = d.node("q").unwrap().clone();
        let n = plan(&d, &src, Side::Bottom, "n1".into(), None);
        assert_eq!(n.shape, Shape::Rounded);
        assert!(n.size.is_none());
        assert_eq!(n.label, "");
        // centred on the axis where a same-size copy would sit
        let r = geometry::node_rect(&d, &src);
        let nr = geometry::node_size(&d, &n);
        let p = n.pos.unwrap();
        assert!((p.x + nr.w / 2.0 - (r.x + r.w / 2.0)).abs() < 1e-3);
    }

    #[test]
    fn a_step_copies_kind_size_and_look_and_centres_on_a_drop() {
        let mut d = Diagram::default();
        let s = d.ensure_node("s", "Screen");
        s.shape = Shape::Screen;
        s.role = rovelo_core::brand::Role::Accent;
        s.pos = Some(Point::new(0.0, 0.0));
        let src = d.node("s").unwrap().clone();
        let r = geometry::node_rect(&d, &src);
        let n = plan(
            &d,
            &src,
            Side::Right,
            "n1".into(),
            Some(Point::new(500.0, 300.0)),
        );
        assert_eq!(
            (n.shape, n.role),
            (Shape::Screen, rovelo_core::brand::Role::Accent)
        );
        assert_eq!(n.size, Some(Size { w: r.w, h: r.h }));
        let p = n.pos.unwrap();
        assert_eq!((p.x + r.w / 2.0, p.y + r.h / 2.0), (500.0, 300.0));
        let no_drop = plan(&d, &src, Side::Right, "n2".into(), None).pos.unwrap();
        assert_eq!(no_drop, origin(r, Side::Right));
    }

    #[test]
    fn drag_box_grows_from_the_start_and_squares_with_shift() {
        let b = drag_box(Point::new(10.0, 10.0), Point::new(-30.0, 30.0), false);
        assert_eq!((b.x, b.y, b.w, b.h), (-30.0, 10.0, 40.0, 20.0));
        let s = drag_box(Point::new(0.0, 0.0), Point::new(50.0, 20.0), true);
        assert_eq!((s.w, s.h), (50.0, 50.0));
        let tiny = drag_box(Point::new(0.0, 0.0), Point::new(1.0, 1.0), false);
        assert_eq!((tiny.w, tiny.h), (MIN_CREATE, MIN_CREATE));
    }

    #[test]
    fn pan_is_the_least_that_uncovers() {
        let free = egui::Rect::from_min_max(egui::pos2(300.0, 70.0), egui::pos2(1000.0, 800.0));
        let inside = egui::Rect::from_min_size(egui::pos2(400.0, 100.0), egui::vec2(100.0, 50.0));
        assert_eq!(pan_to_uncover(inside, free), egui::Vec2::ZERO);
        let under_panel =
            egui::Rect::from_min_size(egui::pos2(250.0, 100.0), egui::vec2(100.0, 50.0));
        assert_eq!(pan_to_uncover(under_panel, free), egui::vec2(50.0, 0.0));
        let below = egui::Rect::from_min_size(egui::pos2(400.0, 780.0), egui::vec2(100.0, 50.0));
        assert_eq!(pan_to_uncover(below, free), egui::vec2(0.0, -30.0));
    }
}
