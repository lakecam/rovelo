// Pure canvas math: snapping, marquee, viewport, rect helpers.
import type { Point, Rect } from "./types";

export const SNAP = 8;

export const snap = (v: number, step = SNAP) => Math.round(v / step) * step;

/**
 * Snap a drag so the anchor's new top-left lands on the grid; every dragged node moves by the same
 * delta, so relative layout is kept. `disabled` (Alt held) returns the raw delta.
 */
export function snapDelta(anchor: Point, dx: number, dy: number, disabled = false, step = SNAP): Point {
  if (disabled) return { x: dx, y: dy };
  return { x: snap(anchor.x + dx, step) - anchor.x, y: snap(anchor.y + dy, step) - anchor.y };
}

export const rectFromPoints = (a: Point, b: Point): Rect => ({
  x: Math.min(a.x, b.x),
  y: Math.min(a.y, b.y),
  w: Math.abs(a.x - b.x),
  h: Math.abs(a.y - b.y),
});

export const intersects = (a: Rect, b: Rect) =>
  a.x <= b.x + b.w && a.x + a.w >= b.x && a.y <= b.y + b.h && a.y + a.h >= b.y;

export const contains = (outer: Rect, inner: Rect) =>
  inner.x >= outer.x && inner.y >= outer.y && inner.x + inner.w <= outer.x + outer.w && inner.y + inner.h <= outer.y + outer.h;

export const union = (a: Rect, b: Rect): Rect => {
  const x = Math.min(a.x, b.x), y = Math.min(a.y, b.y);
  return { x, y, w: Math.max(a.x + a.w, b.x + b.w) - x, h: Math.max(a.y + a.h, b.y + b.h) - y };
};

export const center = (r: Rect): Point => ({ x: r.x + r.w / 2, y: r.y + r.h / 2 });

/** Ids whose rect the marquee touches. A box smaller than 3 units selects nothing (it was a click). */
export function marqueeHits(items: { id: string; rect: Rect }[], box: Rect): string[] {
  if (box.w < 3 && box.h < 3) return [];
  return items.filter((i) => intersects(box, i.rect)).map((i) => i.id);
}

/** Add `hits` to `base` (shift held) or replace it. Order stable, no duplicates. */
export function mergeSelection(base: string[], hits: string[], additive: boolean): string[] {
  if (!additive) return hits;
  return [...base, ...hits.filter((h) => !base.includes(h))];
}

export interface View { x: number; y: number; k: number }
export const MIN_K = 0.1;
export const MAX_K = 4;
export const clampK = (k: number) => Math.min(MAX_K, Math.max(MIN_K, k));

export const toWorld = (v: View, sx: number, sy: number): Point => ({ x: (sx - v.x) / v.k, y: (sy - v.y) / v.k });
export const toScreen = (v: View, p: Point): Point => ({ x: p.x * v.k + v.x, y: p.y * v.k + v.y });

/** Zoom by `factor`, keeping the world point under screen (cx, cy) fixed. */
export function zoomAt(v: View, factor: number, cx: number, cy: number): View {
  const k = clampK(v.k * factor);
  const f = k / v.k;
  return { k, x: cx - (cx - v.x) * f, y: cy - (cy - v.y) * f };
}

export interface Inset { left: number; top: number; right: number; bottom: number }

/** View that fits `bounds` inside the viewport minus the floating chrome insets. */
export function fitView(bounds: Rect, vw: number, vh: number, inset: Inset, maxK = 1.25): View {
  const aw = Math.max(50, vw - inset.left - inset.right);
  const ah = Math.max(50, vh - inset.top - inset.bottom);
  const k = clampK(Math.min(aw / Math.max(bounds.w, 1), ah / Math.max(bounds.h, 1), maxK));
  return {
    k,
    x: inset.left + (aw - bounds.w * k) / 2 - bounds.x * k,
    y: inset.top + (ah - bounds.h * k) / 2 - bounds.y * k,
  };
}

/** Centre the view on a world point, keeping zoom. */
export function centerOn(v: View, p: Point, vw: number, vh: number, inset: Inset): View {
  const cx = inset.left + (vw - inset.left - inset.right) / 2;
  const cy = inset.top + (vh - inset.top - inset.bottom) / 2;
  return { k: v.k, x: cx - p.x * v.k, y: cy - p.y * v.k };
}
