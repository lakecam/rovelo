// Making a step, ported from the original app (openflowkit src/opencanvas: domain/connectors/quickCreate.ts,
// connectHandles.ts, presentation/v2/v2PointerCore.ts dragBox, map/mapMode.ts nudgeInto). Pure: no core calls.
import { freshId } from "./ops";
import { SNAP, snap } from "./geom";
import type { View } from "./geom";
import type { Diagram, Edge, Id, Node, Point, Rect, Size } from "./types";

export type Side = "top" | "right" | "bottom" | "left";
export const SIDES: Side[] = ["top", "right", "bottom", "left"];

/** Plus handles: 16 px on screen, 22 px out from the edge, picked within 9 px. */
export const HANDLE_PX = 16;
export const HANDLE_OFFSET_PX = 22;
export const HANDLE_PICK_PX = 9;
/** Below this a press is a click, in screen pixels. */
export const CLICK_PX = 4;
export const MIN_CREATE = 8;

export const oppositeSide = (s: Side): Side => ({ top: "bottom", right: "left", bottom: "top", left: "right" } as const)[s];

/** Side midpoint of a rect, on its edge. */
export function sideAnchor(r: Rect, s: Side): Point {
  const cx = r.x + r.w / 2, cy = r.y + r.h / 2;
  return s === "top" ? { x: cx, y: r.y } : s === "right" ? { x: r.x + r.w, y: cy } : s === "bottom" ? { x: cx, y: r.y + r.h } : { x: r.x, y: cy };
}

/** The four handle centres in world units at zoom `k`. */
export function handlePoints(r: Rect, k: number): { side: Side; point: Point }[] {
  const o = HANDLE_OFFSET_PX / k;
  return SIDES.map((side) => {
    const a = sideAnchor(r, side);
    const point = side === "top" ? { x: a.x, y: a.y - o } : side === "right" ? { x: a.x + o, y: a.y } : side === "bottom" ? { x: a.x, y: a.y + o } : { x: a.x - o, y: a.y };
    return { side, point };
  });
}

export function pickHandle(r: Rect, p: Point, k: number): Side | null {
  const reach = HANDLE_PICK_PX / k;
  return handlePoints(r, k).find(({ point }) => Math.hypot(point.x - p.x, point.y - p.y) <= reach)?.side ?? null;
}

/** Where a same-size copy sits: one full node size past the source edge. */
export function quickCreateOrigin(r: Rect, s: Side): Point {
  return s === "right" ? { x: r.x + r.w * 2, y: r.y } : s === "left" ? { x: r.x - r.w * 2, y: r.y }
    : s === "bottom" ? { x: r.x, y: r.y + r.h * 2 } : { x: r.x, y: r.y - r.h * 2 };
}

/** The ghost a hover previews: the source's size at the origin a click would use. */
export const ghostRect = (r: Rect, s: Side): Rect => ({ ...quickCreateOrigin(r, s), w: r.w, h: r.h });

const isDecision = (n: Node) => n.shape === "diamond";

export interface QuickCreate { diagram: Diagram; nodeId: Id; edgeId: Id }

/**
 * Same kind, size and look as the source with a blank label (a decision leads to a plain step), centred where a
 * same-size copy would sit, or on `dropAt` when a handle drag was released on empty canvas; plus the arrow into it.
 * `sizeOf` measures a node the way the core does (its manual size, else from its label).
 */
export function planQuickCreate(d: Diagram, sourceId: Id, sourceRect: Rect, side: Side, sizeOf: (n: Node) => Size, dropAt?: Point): QuickCreate | null {
  const src = d.nodes.find((n) => n.id === sourceId);
  if (!src) return null;
  const decision = isDecision(src);
  const nodeId = freshId(d, "n");
  const draft: Node = decision
    ? { id: nodeId, label: "", shape: "rounded", pos: null, size: null, group: src.group, role: src.role, icon: null }
    : { ...src, id: nodeId, label: "", pos: null };
  const size = sizeOf(draft);
  const o = quickCreateOrigin(sourceRect, side);
  const c = dropAt ?? { x: o.x + sourceRect.w / 2, y: o.y + sourceRect.h / 2 };
  const node: Node = { ...draft, pos: { x: c.x - size.w / 2, y: c.y - size.h / 2 } };
  const edgeId = freshId({ ...d, nodes: [...d.nodes, node] }, "e");
  const edge: Edge = { id: edgeId, from: src.id, to: nodeId, label: "", line: "solid", arrow: "end", role: null };
  return { diagram: { ...d, nodes: [...d.nodes, node], edges: [...d.edges, edge] }, nodeId, edgeId };
}

/** The Add step tool's drag: the box drawn from `start` to `p`, snapped to the grid, Shift for a square. */
export function dragBox(start: Point, p: Point, square: boolean): Rect {
  const dx = Math.abs(p.x - start.x), dy = Math.abs(p.y - start.y);
  const w = Math.max(MIN_CREATE, snap(square ? Math.max(dx, dy) : dx, SNAP));
  const h = Math.max(MIN_CREATE, snap(square ? Math.max(dx, dy) : dy, SNAP));
  return { x: p.x < start.x ? start.x - w : start.x, y: p.y < start.y ? start.y - h : start.y, w, h };
}

export interface FreeArea { left: number; top: number; width: number; height: number }

/** The view that puts `box` inside the free area, panning the least and keeping the zoom; null when it already is. */
export function nudgeInto(box: Rect, v: View, free: FreeArea, margin = 16): View | null {
  const axis = (at: number, size: number, lo: number, hi: number) =>
    size > hi - lo ? lo - at : at < lo ? lo - at : at + size > hi ? hi - at - size : 0;
  const dx = axis(box.x * v.k + v.x, box.w * v.k, free.left + margin, free.left + free.width - margin);
  const dy = axis(box.y * v.k + v.y, box.h * v.k, free.top + margin, free.top + free.height - margin);
  return Math.abs(dx) < 0.5 && Math.abs(dy) < 0.5 ? null : { k: v.k, x: v.x + dx, y: v.y + dy };
}
