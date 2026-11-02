//! Pure canvas maths: the pan/zoom transform, snapping, marquee selection, fitting.
use egui::{Pos2, Rect, Vec2, pos2, vec2};

pub const MIN_ZOOM: f32 = 0.05;
pub const MAX_ZOOM: f32 = 6.0;
pub const GRID: f32 = 8.0;

/// `screen = origin + pan + world * zoom`, where `origin` is the canvas rect's top-left.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct View {
    pub pan: Vec2,
    pub zoom: f32,
}

impl Default for View {
    fn default() -> Self {
        View {
            pan: vec2(120.0, 90.0),
            zoom: 1.0,
        }
    }
}

impl View {
    pub fn to_screen(self, origin: Pos2, p: Pos2) -> Pos2 {
        origin + self.pan + p.to_vec2() * self.zoom
    }

    pub fn to_world(self, origin: Pos2, s: Pos2) -> Pos2 {
        ((s - origin - self.pan) / self.zoom).to_pos2()
    }

    pub fn rect_to_screen(self, origin: Pos2, r: Rect) -> Rect {
        Rect::from_min_max(self.to_screen(origin, r.min), self.to_screen(origin, r.max))
    }

    /// Repair non-finite or out-of-range state so nothing downstream panics.
    pub fn sanitize(&mut self) {
        if !self.zoom.is_finite() {
            self.zoom = 1.0;
        }
        self.zoom = self.zoom.clamp(MIN_ZOOM, MAX_ZOOM);
        if !self.pan.x.is_finite() || !self.pan.y.is_finite() {
            self.pan = View::default().pan;
        }
        self.pan = self.pan.clamp(vec2(-1.0e7, -1.0e7), vec2(1.0e7, 1.0e7));
    }

    /// Zoom by `factor`, keeping the world point under `anchor` (screen) fixed.
    pub fn zoom_about(&mut self, origin: Pos2, anchor: Pos2, factor: f32) {
        self.sanitize();
        if !factor.is_finite() || factor <= 0.0 {
            return;
        }
        let w = self.to_world(origin, anchor);
        self.zoom = (self.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
        self.pan = anchor - origin - w.to_vec2() * self.zoom;
        self.sanitize();
    }

    /// View that shows `bounds` (world) centred in a viewport of `size` with `margin` px around it.
    pub fn fit(bounds: Rect, size: Vec2, margin: f32) -> View {
        let avail = vec2(
            (size.x - 2.0 * margin).max(40.0),
            (size.y - 2.0 * margin).max(40.0),
        );
        let bw = bounds.width().max(1.0);
        let bh = bounds.height().max(1.0);
        let zoom = (avail.x / bw).min(avail.y / bh).clamp(MIN_ZOOM, 1.6);
        let c = bounds.center();
        let mut v = View {
            pan: size / 2.0 - c.to_vec2() * zoom,
            zoom,
        };
        v.sanitize();
        v
    }

    pub fn center_on(&mut self, world: Pos2, size: Vec2) {
        self.sanitize();
        self.pan = size / 2.0 - world.to_vec2() * self.zoom;
        self.sanitize();
    }
}

pub fn snap(v: f32, grid: f32) -> f32 {
    if grid <= 0.0 {
        v
    } else {
        (v / grid).round() * grid
    }
}

/// Movement to apply to every dragged item so the grabbed one lands on the grid.
/// `total` is the unsnapped cumulative drag; `enabled` is false while Alt is held.
pub fn snapped_delta(anchor_origin: Pos2, total: Vec2, grid: f32, enabled: bool) -> Vec2 {
    if !enabled {
        return total;
    }
    vec2(
        snap(anchor_origin.x + total.x, grid) - anchor_origin.x,
        snap(anchor_origin.y + total.y, grid) - anchor_origin.y,
    )
}

/// Ids whose rectangles intersect the marquee.
pub fn marquee_hits(items: &[(String, Rect)], marquee: Rect) -> Vec<String> {
    items
        .iter()
        .filter(|(_, r)| r.intersects(marquee))
        .map(|(id, _)| id.clone())
        .collect()
}

pub fn from_core(r: rovelo_core::model::Rect) -> Rect {
    Rect::from_min_size(pos2(r.x, r.y), vec2(r.w, r.h))
}

pub fn pt(p: rovelo_core::model::Point) -> Pos2 {
    pos2(p.x, p.y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoom_keeps_anchor_fixed() {
        let origin = pos2(100.0, 50.0);
        let mut v = View::default();
        let anchor = pos2(400.0, 300.0);
        let before = v.to_world(origin, anchor);
        v.zoom_about(origin, anchor, 1.7);
        let after = v.to_world(origin, anchor);
        assert!((before - after).length() < 1e-3);
        assert!((v.zoom - 1.7).abs() < 1e-6);
    }

    #[test]
    fn zoom_is_clamped_and_survives_garbage() {
        let o = Pos2::ZERO;
        let mut v = View::default();
        for _ in 0..200 {
            v.zoom_about(o, pos2(10.0, 10.0), 3.0);
        }
        assert_eq!(v.zoom, MAX_ZOOM);
        for _ in 0..400 {
            v.zoom_about(o, pos2(10.0, 10.0), 0.3);
        }
        assert_eq!(v.zoom, MIN_ZOOM);
        v.zoom = f32::NAN;
        v.pan = vec2(f32::INFINITY, 0.0);
        v.sanitize();
        assert!(v.zoom.is_finite() && v.pan.x.is_finite());
        v.zoom_about(o, Pos2::ZERO, f32::NAN);
    }

    #[test]
    fn screen_world_round_trip() {
        let v = View {
            pan: vec2(13.0, -7.0),
            zoom: 2.5,
        };
        let o = pos2(5.0, 5.0);
        let p = pos2(33.0, 71.0);
        assert!((v.to_world(o, v.to_screen(o, p)) - p).length() < 1e-4);
    }

    #[test]
    fn snapping() {
        assert_eq!(snap(13.0, 8.0), 16.0);
        assert_eq!(snap(-3.0, 8.0), 0.0);
        assert_eq!(snap(5.0, 0.0), 5.0);
        let d = snapped_delta(pos2(3.0, 3.0), vec2(10.0, 1.0), 8.0, true);
        assert_eq!(d, vec2(13.0, 5.0));
        assert_eq!(
            snapped_delta(pos2(3.0, 3.0), vec2(10.0, 1.0), 8.0, false),
            vec2(10.0, 1.0)
        );
    }

    #[test]
    fn marquee_selects_intersecting() {
        let items = vec![
            (
                "a".to_string(),
                Rect::from_min_size(pos2(0.0, 0.0), vec2(10.0, 10.0)),
            ),
            (
                "b".to_string(),
                Rect::from_min_size(pos2(100.0, 0.0), vec2(10.0, 10.0)),
            ),
        ];
        let m = Rect::from_min_max(pos2(5.0, 5.0), pos2(50.0, 50.0));
        assert_eq!(marquee_hits(&items, m), vec!["a".to_string()]);
        let m2 = Rect::from_min_max(pos2(50.0, 50.0), pos2(5.0, 5.0));
        assert!(
            marquee_hits(&items, Rect::from_two_pos(m2.min, m2.max)).contains(&"a".to_string())
        );
        assert!(marquee_hits(&[], m).is_empty());
    }

    #[test]
    fn fit_centres_bounds() {
        let b = Rect::from_min_size(pos2(0.0, 0.0), vec2(400.0, 200.0));
        let size = vec2(800.0, 600.0);
        let v = View::fit(b, size, 40.0);
        let c = v.to_screen(Pos2::ZERO, b.center());
        assert!((c - pos2(400.0, 300.0)).length() < 1e-3);
        assert!(v.zoom <= 1.6);
        // degenerate bounds do not panic
        let _ = View::fit(Rect::from_min_size(Pos2::ZERO, Vec2::ZERO), size, 40.0);
        let _ = View::fit(b, Vec2::ZERO, 40.0);
    }
}
