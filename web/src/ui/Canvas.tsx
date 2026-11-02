// Interactive canvas: an SVG layer drawn by React from the Rust scene. Selection, drag, connect,
// marquee, pan/zoom and inline editing are React; node and edge artwork is the exporter's own markup,
// so what you see is what SVG export writes.
import { memo, useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { core } from "@/wasm";
import { ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuSeparator, ContextMenuShortcut, ContextMenuTrigger } from "@/components/ui/context-menu";
import { swapNodeIcon } from "@/lib/nodeIcons";
import { mapEdge, mapGroup, mapNode, mapNodes } from "@/lib/ops";
import { CLICK_PX, HANDLE_OFFSET_PX, HANDLE_PX, dragBox, ghostRect, handlePoints, oppositeSide, sideAnchor, type Side } from "@/lib/quickCreate";
import { marqueeHits, mergeSelection, rectFromPoints, snapDelta, toWorld, zoomAt, type View } from "@/lib/geom";
import type { Id, Point, Rect, Selection } from "@/lib/types";
import { emptySelection } from "@/lib/types";
import { glass } from "@/lib/glass";
import { useTheme } from "./theme";
import * as A from "@/state/actions";
import { amend, beginGesture, cancelGesture, edit, endGesture, getState, live, sceneOf, setState, useStore } from "@/state/store";

const Frag = memo(function Frag({ html }: { html: string }) {
  return <g dangerouslySetInnerHTML={{ __html: html }} />;
});

type Gesture =
  | { kind: "pan"; sx: number; sy: number; vx: number; vy: number }
  | { kind: "drag"; id: Id; sx: number; sy: number; ids: Id[]; start: Map<Id, Point>; anchor: Point; moved: boolean; wasSelected: boolean; shift: boolean }
  | { kind: "marquee"; sx: number; sy: number; base: Selection; additive: boolean }
  | { kind: "link"; from: Id; side: Side | null; sx: number; sy: number; moved: boolean }
  | { kind: "create"; sx: number; sy: number; start: Point };

type MenuTarget = { kind: "node" | "edge" | "group" | "canvas"; at: Point };

const THRESH = 3;
const polyline = (pts: Point[]) => pts.map((p) => `${p.x},${p.y}`).join(" ");

export function Canvas() {
  const diagram = useStore((s) => s.diagram);
  const view = useStore((s) => s.view);
  const tool = useStore((s) => s.tool);
  const selection = useStore((s) => s.selection);
  const editing = useStore((s) => s.editing);
  const scene = sceneOf(diagram);

  const root = useRef<HTMLDivElement>(null);
  const gesture = useRef<Gesture | null>(null);
  const space = useRef(false);
  const [panning, setPanning] = useState(false);
  const [moving, setMoving] = useState(false);
  const [marquee, setMarquee] = useState<Rect | null>(null);
  const [link, setLink] = useState<{ from: Id; to: Point; over: Id | null } | null>(null);
  const [hover, setHover] = useState<{ id: Id; side: Side | null } | null>(null);
  const [draft, setDraft] = useState<Rect | null>(null); // Add step tool box being drawn
  const [menu, setMenu] = useState<MenuTarget>({ kind: "canvas", at: { x: 0, y: 0 } });

  const nodeIcon = useMemo(() => new Map(diagram.nodes.map((n) => [n.id, n.icon])), [diagram.nodes]);
  const rectOf = useMemo(() => new Map(scene.nodes.map((n) => [n.id, n.rect])), [scene]);

  // Viewport size -> store, so fit and centring know the window.
  useLayoutEffect(() => {
    const el = root.current!;
    const ro = new ResizeObserver(() => {
      const r = el.getBoundingClientRect();
      const first = getState().viewport.w === 1200 && getState().viewport.h === 800;
      setState({ viewport: { w: r.width, h: r.height } });
      if (first && getState().diagram.nodes.length) A.fit();
      else if (first) setState({ view: { x: r.width / 2, y: r.height / 2, k: 1 } });
    });
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  const local = useCallback((e: { clientX: number; clientY: number }) => {
    const r = root.current!.getBoundingClientRect();
    return { sx: e.clientX - r.left, sy: e.clientY - r.top };
  }, []);
  const world = useCallback((e: { clientX: number; clientY: number }): Point => {
    const { sx, sy } = local(e);
    return toWorld(getState().view, sx, sy);
  }, [local]);

  const nodeAt = useCallback((p: Point): Id | null => {
    for (let i = scene.nodes.length - 1; i >= 0; i--) {
      const r = scene.nodes[i].rect;
      if (p.x >= r.x && p.x <= r.x + r.w && p.y >= r.y && p.y <= r.y + r.h) return scene.nodes[i].id;
    }
    return null;
  }, [scene]);

  // Space = temporary hand; wheel = pan, ctrl/pinch = zoom (needs a non-passive listener).
  useEffect(() => {
    const el = root.current!;
    const wheel = (e: WheelEvent) => {
      e.preventDefault();
      const v = getState().view;
      if (e.ctrlKey || e.metaKey) {
        const { sx, sy } = local(e);
        setState({ view: zoomAt(v, Math.exp(-e.deltaY * (e.ctrlKey ? 0.01 : 0.003)), sx, sy) });
      } else {
        setState({ view: { ...v, x: v.x - (e.shiftKey && !e.deltaX ? e.deltaY : e.deltaX), y: v.y - (e.shiftKey && !e.deltaX ? 0 : e.deltaY) } });
      }
    };
    const typing = (t: EventTarget | null) => t instanceof HTMLElement && (t.isContentEditable || /^(INPUT|TEXTAREA|SELECT)$/.test(t.tagName));
    const down = (e: KeyboardEvent) => {
      if (e.code === "Space" && !typing(e.target) && !e.repeat) { space.current = true; el.style.cursor = "grab"; if (e.target === document.body) e.preventDefault(); }
    };
    const up = (e: KeyboardEvent) => {
      if (e.code === "Space") { space.current = false; el.style.cursor = ""; }
    };
    el.addEventListener("wheel", wheel, { passive: false });
    window.addEventListener("keydown", down);
    window.addEventListener("keyup", up);
    return () => {
      el.removeEventListener("wheel", wheel);
      window.removeEventListener("keydown", down);
      window.removeEventListener("keyup", up);
    };
  }, [local]);

  // ---- gestures -----------------------------------------------------------------------------
  const onMove = useCallback((e: PointerEvent) => {
    const g = gesture.current;
    if (!g) return;
    const s = getState();
    if (g.kind === "pan") {
      setState({ view: { ...s.view, x: g.vx + (e.clientX - g.sx), y: g.vy + (e.clientY - g.sy) } });
    } else if (g.kind === "drag") {
      const dist = Math.hypot(e.clientX - g.sx, e.clientY - g.sy);
      if (!g.moved) {
        if (dist < THRESH) return;
        g.moved = true;
        setMoving(true);
        beginGesture();
      }
      const dx = (e.clientX - g.sx) / s.view.k, dy = (e.clientY - g.sy) / s.view.k;
      const sd = snapDelta(g.anchor, dx, dy, e.altKey);
      live((d) => mapNodes(d, g.ids, (n) => {
        const p0 = g.start.get(n.id)!;
        return { ...n, pos: { x: p0.x + sd.x, y: p0.y + sd.y } };
      }));
    } else if (g.kind === "marquee") {
      const a = toWorld(s.view, g.sx, g.sy);
      const { sx, sy } = local(e);
      const box = rectFromPoints(a, toWorld(s.view, sx, sy));
      setMarquee(box);
      const hits = marqueeHits(sceneOf(s.diagram).nodes, box);
      setState({ selection: { nodes: mergeSelection(g.base.nodes, hits, g.additive), edges: g.additive ? g.base.edges : [], groups: g.additive ? g.base.groups : [] } });
    } else if (g.kind === "link") {
      if (Math.hypot(e.clientX - g.sx, e.clientY - g.sy) >= CLICK_PX) g.moved = true;
      if (!g.moved) return;
      const p = world(e);
      const over = nodeAtRef.current(p);
      setLink({ from: g.from, to: p, over: over && over !== g.from ? over : null });
    } else if (g.kind === "create") {
      if (Math.hypot(e.clientX - g.sx, e.clientY - g.sy) < CLICK_PX) return setDraft(null);
      setDraft(dragBox(g.start, world(e), e.shiftKey));
    }
  }, [local, world]);

  const nodeAtRef = useRef(nodeAt);
  nodeAtRef.current = nodeAt;

  const onUp = useCallback((e: PointerEvent) => {
    const g = gesture.current;
    gesture.current = null;
    window.removeEventListener("pointermove", onMove);
    window.removeEventListener("pointerup", onUp);
    setPanning(false);
    setMoving(false);
    if (!g) return;
    if (g.kind === "drag") {
      if (g.moved) endGesture();
      else if (g.wasSelected && !g.shift && getState().selection.nodes.length > 1) {
        setState({ selection: { nodes: [g.id], edges: [], groups: [] } });
      }
    } else if (g.kind === "marquee") {
      setMarquee(null);
      if (!g.additive && Math.hypot(e.clientX - g.sx, e.clientY - g.sy) < THRESH) A.clearSelection();
    } else if (g.kind === "link") {
      // finishConnect: a click on a handle makes the next step; a drag connects, or makes it where released.
      const p = world(e);
      const over = nodeAtRef.current(p);
      setLink(null);
      setHover(null);
      // The release point is authoritative: a fast flick that never painted a preview still counts.
      if (Math.hypot(e.clientX - g.sx, e.clientY - g.sy) >= CLICK_PX) g.moved = true;
      if (!g.moved) {
        if (g.side) A.quickCreate(g.from, g.side);
      } else if (over && over !== g.from) A.connectNodes(g.from, over);
      else if (!over && g.side) A.quickCreate(g.from, g.side, p);
      // Released back on the source: nothing.
    } else if (g.kind === "create") {
      // finishCreate: a click places the default size centred on it; a drag commits the box it drew.
      setDraft(null);
      const p = world(e);
      if (Math.hypot(e.clientX - g.sx, e.clientY - g.sy) < CLICK_PX) A.addNodeAt(p.x, p.y);
      else A.addNodeBox(dragBox(g.start, p, e.shiftKey));
    }
  }, [onMove, world]);

  useEffect(() => () => {
    window.removeEventListener("pointermove", onMove);
    window.removeEventListener("pointerup", onUp);
  }, [onMove, onUp]);

  const begin = (g: Gesture) => {
    gesture.current = g;
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
  };

  const onPointerDown = (e: React.PointerEvent) => {
    if (e.button === 2) return;
    if (editing) (document.activeElement as HTMLElement | null)?.blur();
    const t = e.target as Element;
    const s = getState();
    const handleEl = t.closest("[data-handle]");
    const nodeEl = t.closest("[data-node]");
    const edgeEl = t.closest("[data-edge]");
    const groupEl = t.closest("[data-group]");
    const { sx, sy } = local(e);

    if (e.button === 1 || s.tool === "hand" || space.current) {
      e.preventDefault();
      setPanning(true);
      return begin({ kind: "pan", sx: e.clientX, sy: e.clientY, vx: s.view.x, vy: s.view.y });
    }
    if (handleEl) {
      return begin({ kind: "link", from: handleEl.getAttribute("data-handle")!, side: handleEl.getAttribute("data-side") as Side, sx: e.clientX, sy: e.clientY, moved: false });
    }
    if (nodeEl) {
      const id = nodeEl.getAttribute("data-node")!;
      if (s.tool === "connect") {
        setState({ selection: { nodes: [id], edges: [], groups: [] } });
        return begin({ kind: "link", from: id, side: null, sx: e.clientX, sy: e.clientY, moved: false });
      }
      const wasSelected = s.selection.nodes.includes(id);
      let sel = s.selection;
      if (e.shiftKey) {
        sel = { nodes: wasSelected ? sel.nodes.filter((n) => n !== id) : [...sel.nodes, id], edges: sel.edges, groups: sel.groups };
      } else if (!wasSelected) {
        sel = { nodes: [id], edges: [], groups: [] };
      }
      A.select(sel);
      if (!sel.nodes.includes(id)) return;
      const ids = sel.nodes;
      const start = new Map<Id, Point>();
      for (const nid of ids) {
        const n = s.diagram.nodes.find((x) => x.id === nid);
        const r = sceneOf(s.diagram).nodes.find((x) => x.id === nid)?.rect;
        start.set(nid, n?.pos ?? (r ? { x: r.x, y: r.y } : { x: 0, y: 0 }));
      }
      return begin({ kind: "drag", id, sx: e.clientX, sy: e.clientY, ids, start, anchor: start.get(id)!, moved: false, wasSelected, shift: e.shiftKey });
    }
    if (edgeEl) {
      const id = edgeEl.getAttribute("data-edge")!;
      A.select(e.shiftKey ? { ...s.selection, edges: s.selection.edges.includes(id) ? s.selection.edges.filter((x) => x !== id) : [...s.selection.edges, id] } : { nodes: [], edges: [id], groups: [] });
      return;
    }
    if (groupEl) {
      A.select({ nodes: [], edges: [], groups: [groupEl.getAttribute("data-group")!] });
      return;
    }
    if (s.tool === "node") return begin({ kind: "create", sx: e.clientX, sy: e.clientY, start: world(e) });
    return begin({ kind: "marquee", sx, sy, base: e.shiftKey ? s.selection : emptySelection(), additive: e.shiftKey });
  };

  const onDoubleClick = (e: React.MouseEvent) => {
    const t = e.target as Element;
    const s = getState();
    if (s.tool === "hand") return;
    const nodeEl = t.closest("[data-node]");
    const edgeEl = t.closest("[data-edge]");
    const groupEl = t.closest("[data-group]");
    if (nodeEl) setState({ editing: { kind: "node", id: nodeEl.getAttribute("data-node")! } });
    else if (edgeEl) setState({ editing: { kind: "edge", id: edgeEl.getAttribute("data-edge")! } });
    else if (groupEl) setState({ editing: { kind: "group", id: groupEl.getAttribute("data-group")! } });
    else if (!t.closest("[data-handle]") && !t.closest("textarea")) {
      const p = world(e);
      A.addNodeAt(p.x, p.y);
    }
  };

  // Hover: the step under the pointer, or whose handles the pointer is near (so they stay while you reach them).
  const onPointerMoveRoot = (e: React.PointerEvent) => {
    if (gesture.current) return;
    const t = e.target as Element;
    const handle = t.closest("[data-handle]");
    let next: { id: Id; side: Side | null } | null = null;
    if (handle) next = { id: handle.getAttribute("data-handle")!, side: handle.getAttribute("data-side") as Side };
    else {
      const p = world(e);
      const reach = (HANDLE_OFFSET_PX + HANDLE_PX / 2) / getState().view.k;
      const hit = [...scene.nodes].reverse().find(({ rect: r }) => p.x >= r.x - reach && p.x <= r.x + r.w + reach && p.y >= r.y - reach && p.y <= r.y + r.h + reach);
      if (hit) next = { id: hit.id, side: null };
    }
    if (next?.id !== hover?.id || next?.side !== hover?.side) setHover(next);
  };

  const onContextMenu = (e: React.MouseEvent) => {
    const t = e.target as Element;
    const at = world(e);
    const s = getState();
    const n = t.closest("[data-node]")?.getAttribute("data-node");
    const ed = t.closest("[data-edge]")?.getAttribute("data-edge");
    const gr = t.closest("[data-group]")?.getAttribute("data-group");
    if (n) {
      if (!s.selection.nodes.includes(n)) A.select({ nodes: [n], edges: [], groups: [] });
      setMenu({ kind: "node", at });
    } else if (ed) {
      A.select({ nodes: [], edges: [ed], groups: [] });
      setMenu({ kind: "edge", at });
    } else if (gr) {
      A.select({ nodes: [], edges: [], groups: [gr] });
      setMenu({ kind: "group", at });
    } else {
      setMenu({ kind: "canvas", at });
    }
  };

  // Esc while a gesture is live puts everything back.
  useEffect(() => {
    const k = (e: KeyboardEvent) => {
      if (e.key !== "Escape" || !gesture.current) return;
      const g = gesture.current;
      if (g.kind === "drag" && g.moved) cancelGesture();
      gesture.current = null;
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
      setMarquee(null); setLink(null); setDraft(null); setPanning(false); setMoving(false);
    };
    window.addEventListener("keydown", k);
    return () => window.removeEventListener("keydown", k);
  }, [onMove, onUp]);

  const cursor = panning ? "grabbing" : tool === "hand" ? "grab" : tool === "connect" || tool === "node" ? "crosshair" : "default";
  const k = view.k;
  const sel = selection;
  const single = sel.nodes.length === 1 && !sel.edges.length && !sel.groups.length ? sel.nodes[0] : null;
  // Plus handles on the selected step and on the step under the pointer (drawHandles).
  const handleIds = tool === "select" && !editing && !moving && !link ? [...new Set([single, hover?.id].filter((x): x is Id => !!x))] : [];
  const ghostFor = handleIds.length && hover?.side && rectOf.get(hover.id) ? { r: rectOf.get(hover.id)!, side: hover.side } : null;
  const dotStep = 24 * k;
  const dots = k < 0.35 ? 0 : 1;

  return (
    <ContextMenu>
      <ContextMenuTrigger asChild>
        <div
          ref={root}
          className="absolute inset-0 select-none overflow-hidden"
          style={{
            cursor,
            touchAction: "none",
            backgroundImage: dots ? "radial-gradient(circle, var(--rv-dot) 1.2px, transparent 1.4px)" : "none",
            backgroundSize: `${dotStep}px ${dotStep}px`,
            backgroundPosition: `${view.x - dotStep / 2}px ${view.y - dotStep / 2}px`,
          }}
          onPointerDown={onPointerDown}
          onPointerMove={onPointerMoveRoot}
          onDoubleClick={onDoubleClick}
          onContextMenu={onContextMenu}
          data-testid="canvas"
        >
          <GlassGroups view={view} />
          <svg className="absolute inset-0 h-full w-full" role="application" aria-label="Diagram canvas">
            <defs dangerouslySetInnerHTML={{ __html: scene.defs }} />
            <g transform={`translate(${view.x} ${view.y}) scale(${k})`}>
              {scene.groups.map((g) => {
                const on = sel.groups.includes(g.id);
                return (
                  <g key={g.id}>
                    <rect data-group={g.id} x={g.rect.x} y={g.rect.y} width={g.rect.w} height={28} fill="transparent" style={{ cursor: "pointer" }} />
                    {on && <rect x={g.rect.x - 3} y={g.rect.y - 3} width={g.rect.w + 6} height={g.rect.h + 6} rx={23} fill="none" stroke="var(--rv-accent)" strokeWidth={1.5 / k} pointerEvents="none" />}
                  </g>
                );
              })}
              {scene.edges.map((e) => {
                if (e.points.length < 2) return null;
                const on = sel.edges.includes(e.id);
                return (
                  <g key={e.id}>
                    {on && <polyline points={polyline(e.points)} fill="none" stroke="var(--rv-accent)" strokeOpacity={0.35} strokeWidth={7 / k + 2} strokeLinejoin="round" strokeLinecap="round" pointerEvents="none" />}
                    <Frag html={e.svg} />
                    <polyline data-edge={e.id} points={polyline(e.points)} fill="none" stroke="transparent" strokeWidth={14 / k} pointerEvents="stroke" style={{ cursor: "pointer" }} />
                  </g>
                );
              })}
              {scene.nodes.map((n) => (
                <g key={n.id} className={editing?.kind === "node" && editing.id === n.id ? "rv-editing" : undefined}>
                  <Frag html={swapNodeIcon(n.svg, nodeIcon.get(n.id))} />
                  <rect data-node={n.id} x={n.rect.x} y={n.rect.y} width={n.rect.w} height={n.rect.h} fill="transparent" style={{ cursor: tool === "select" ? "move" : undefined }} />
                </g>
              ))}
              {scene.edges.map((e) =>
                e.label_svg && e.label_rect ? (
                  <g key={e.id} className={editing?.kind === "edge" && editing.id === e.id ? "rv-editing" : undefined}>
                    <Frag html={e.label_svg} />
                    <rect data-edge={e.id} x={e.label_rect.x} y={e.label_rect.y} width={e.label_rect.w} height={e.label_rect.h} fill="transparent" style={{ cursor: "pointer" }} />
                  </g>
                ) : null,
              )}
              {scene.badges.map((b) => <Frag key={b.id} html={b.svg} />)}

              {sel.nodes.map((id) => {
                const r = rectOf.get(id);
                return r ? <rect key={id} x={r.x - 4} y={r.y - 4} width={r.w + 8} height={r.h + 8} rx={8} fill="none" stroke="var(--rv-accent)" strokeWidth={1.5 / k} pointerEvents="none" /> : null;
              })}
              {ghostFor && <Ghost r={ghostFor.r} side={ghostFor.side} k={k} />}
              {handleIds.map((id) => rectOf.get(id) && <PlusHandles key={id} id={id} r={rectOf.get(id)!} k={k} active={hover?.id === id ? hover.side : null} />)}
              {draft && <rect x={draft.x} y={draft.y} width={draft.w} height={draft.h} rx={8} fill="var(--rv-accent)" fillOpacity={0.05} stroke="var(--rv-accent)" strokeOpacity={0.6} strokeWidth={1.5 / k} pointerEvents="none" />}
              {link && rectOf.get(link.from) && (
                <g pointerEvents="none">
                  <line x1={rectOf.get(link.from)!.x + rectOf.get(link.from)!.w / 2} y1={rectOf.get(link.from)!.y + rectOf.get(link.from)!.h / 2} x2={link.to.x} y2={link.to.y} stroke="var(--rv-accent)" strokeWidth={2 / k} strokeDasharray={`${6 / k} ${4 / k}`} />
                  {link.over && rectOf.get(link.over) && <rect x={rectOf.get(link.over)!.x - 4} y={rectOf.get(link.over)!.y - 4} width={rectOf.get(link.over)!.w + 8} height={rectOf.get(link.over)!.h + 8} rx={8} fill="var(--rv-accent-soft)" fillOpacity={0.5} stroke="var(--rv-accent)" strokeWidth={2 / k} />}
                  <circle cx={link.to.x} cy={link.to.y} r={4 / k} fill="var(--rv-accent)" />
                </g>
              )}
              {marquee && <rect x={marquee.x} y={marquee.y} width={marquee.w} height={marquee.h} fill="var(--rv-accent)" fillOpacity={0.08} stroke="var(--rv-accent)" strokeWidth={1 / k} pointerEvents="none" />}
            </g>
          </svg>
          <InlineEditor view={view} />
        </div>
      </ContextMenuTrigger>
      <CanvasMenu target={menu} />
    </ContextMenu>
  );
}

/**
 * Groups as tinted glass: HTML layers under the SVG so they get a real backdrop blur (16px on screen), the role
 * tint at 10% (light 8%), a 1.5px border in the role's vibrant step and a white 8% highlight along the top.
 * The exported SVG keeps the core's own stand-in.
 */
function GlassGroups({ view }: { view: View }) {
  const diagram = useStore((s) => s.diagram);
  const dark = useTheme() === "dark";
  const scene = sceneOf(diagram);
  const editingId = useStore((s) => (s.editing?.kind === "group" ? s.editing.id : null));
  if (!scene.groups.length) return null;
  return (
    <div aria-hidden className="pointer-events-none absolute left-0 top-0 origin-top-left" style={{ transform: `translate(${view.x}px, ${view.y}px) scale(${view.k})` }}>
      {scene.groups.map((sg) => {
        const g = diagram.groups.find((x) => x.id === sg.id);
        if (!g) return null;
        const c = glass(diagram.brand.palette[g.role], dark);
        const r = sg.rect;
        return (
          <div
            key={sg.id}
            className="absolute"
            style={{
              left: r.x, top: r.y, width: r.w, height: r.h,
              borderRadius: 20,
              background: c.fill,
              border: `1.5px solid ${c.border}`,
              boxShadow: "inset 0 1px 0 rgba(255, 255, 255, 0.08)",
              backdropFilter: `blur(${16 / view.k}px)`,
              WebkitBackdropFilter: `blur(${16 / view.k}px)`,
            }}
          >
            {editingId !== sg.id && (
              <div className="absolute left-3 top-3 flex items-center gap-2 text-xs font-semibold" style={{ color: c.title }}>
                <span className="size-2 rounded-full" style={{ background: c.title }} />
                {g.label}
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}

/** drawHandles: four 16 px (screen) circles 22 px out from each side, surface fill, accent stroke, a plus inside;
 * the hovered one fills accent with the plus in surface. */
function PlusHandles({ id, r, k, active }: { id: Id; r: Rect; k: number; active: Side | null }) {
  const rad = HANDLE_PX / 2 / k;
  const sw = 1.25 / k;
  return (
    <g>
      {handlePoints(r, k).map(({ side, point: p }) => {
        const on = side === active;
        return (
          <g key={side} data-handle={id} data-side={side} style={{ cursor: "pointer" }}>
            <circle cx={p.x} cy={p.y} r={rad + 4 / k} fill="transparent" />
            <circle cx={p.x} cy={p.y} r={rad} fill={on ? "var(--rv-accent)" : "var(--rv-surface)"} stroke="var(--rv-accent)" strokeWidth={sw} />
            <path d={`M${p.x - rad / 2} ${p.y}H${p.x + rad / 2}M${p.x} ${p.y - rad / 2}V${p.y + rad / 2}`} stroke={on ? "var(--rv-surface)" : "var(--rv-accent)"} strokeWidth={sw} strokeLinecap="round" />
          </g>
        );
      })}
    </g>
  );
}

/** drawQuickCreateGhost: the same-size step one gap away and the arrow into it. */
function Ghost({ r, side, k }: { r: Rect; side: Side; k: number }) {
  const g = ghostRect(r, side);
  const from = sideAnchor(r, side);
  const to = sideAnchor(g, oppositeSide(side));
  const back = 8 / k;
  const dx = Math.sign(to.x - from.x), dy = Math.sign(to.y - from.y);
  const head = `M${to.x - dx * back - dy * back * 0.6} ${to.y - dy * back - dx * back * 0.6}L${to.x} ${to.y}L${to.x - dx * back + dy * back * 0.6} ${to.y - dy * back + dx * back * 0.6}`;
  return (
    <g pointerEvents="none">
      <rect x={g.x} y={g.y} width={g.w} height={g.h} rx={6} fill="var(--rv-accent)" fillOpacity={0.05} stroke="var(--rv-accent)" strokeOpacity={0.45} strokeWidth={1 / k} />
      <path d={`M${from.x} ${from.y}L${to.x} ${to.y}`} stroke="var(--rv-accent)" strokeOpacity={0.45} strokeWidth={1.5 / k} fill="none" />
      <path d={head} stroke="var(--rv-accent)" strokeOpacity={0.45} strokeWidth={1.5 / k} fill="none" />
    </g>
  );
}

// ---- inline editor ---------------------------------------------------------------------------
function InlineEditor({ view }: { view: View }) {
  const editing = useStore((s) => s.editing);
  const diagram = useStore((s) => s.diagram);
  if (!editing) return null;
  const scene = sceneOf(diagram);
  let rect: Rect | undefined;
  let value = "";
  let bg = "var(--rv-surface)";
  let fg = "var(--rv-text)";
  if (editing.kind === "node") {
    const n = diagram.nodes.find((x) => x.id === editing.id);
    rect = scene.nodes.find((x) => x.id === editing.id)?.rect;
    if (!n || !rect) return null;
    value = n.label;
    const c = core.nodeColors(diagram.brand, n.role);
    bg = c.fill; fg = c.text;
  } else if (editing.kind === "edge") {
    const e = diagram.edges.find((x) => x.id === editing.id);
    const es = scene.edges.find((x) => x.id === editing.id);
    if (!e || !es || es.points.length < 2) return null;
    rect = es.label_rect ?? { x: es.label_at.x - 50, y: es.label_at.y - 12, w: 100, h: 24 };
    if (rect.w < 100) rect = { ...rect, x: rect.x - (100 - rect.w) / 2, w: 100 };
    value = e.label;
    const b = diagram.brand.background;
    bg = `rgb(${b[0]} ${b[1]} ${b[2]})`;
    const t = diagram.brand.text;
    fg = `rgb(${t[0]} ${t[1]} ${t[2]})`;
  } else {
    const g = diagram.groups.find((x) => x.id === editing.id);
    const gr = scene.groups.find((x) => x.id === editing.id)?.rect;
    if (!g || !gr) return null;
    rect = { x: gr.x + 8, y: gr.y + 2, w: Math.max(120, gr.w - 16), h: 26 };
    value = g.label;
  }
  return <EditorBox key={`${editing.kind}:${editing.id}`} editing={editing} rect={rect} value={value} view={view} bg={bg} fg={fg} fontSize={diagram.brand.font_size * (editing.kind === "node" ? 1 : 0.86)} multiline={editing.kind === "node"} />;
}

function EditorBox({ editing, rect, value, view, bg, fg, fontSize, multiline }: {
  editing: NonNullable<ReturnType<typeof getState>["editing"]>; rect: Rect; value: string; view: View; bg: string; fg: string; fontSize: number; multiline: boolean;
}) {
  const ta = useRef<HTMLTextAreaElement>(null);
  const done = useRef(false);
  useEffect(() => { ta.current?.focus(); ta.current?.select(); }, []);
  const finish = (commitIt: boolean) => {
    if (done.current) return;
    done.current = true;
    const text = ta.current?.value ?? value;
    if (commitIt && text !== value) {
      if (editing.kind === "node" && editing.isNew) amend((d) => mapNode(d, editing.id, (n) => ({ ...n, label: text })));
      else if (editing.kind === "node") edit((d) => mapNode(d, editing.id, (n) => ({ ...n, label: text })));
      else if (editing.kind === "edge") edit((d) => mapEdge(d, editing.id, (e) => ({ ...e, label: text })));
      else edit((d) => mapGroup(d, editing.id, (g) => ({ ...g, label: text })));
    }
    setState({ editing: null });
  };
  const k = view.k;
  return (
    <textarea
      ref={ta}
      defaultValue={value}
      aria-label="Edit label"
      rows={1}
      spellCheck={false}
      onPointerDown={(e) => e.stopPropagation()}
      onDoubleClick={(e) => e.stopPropagation()}
      onContextMenu={(e) => e.stopPropagation()}
      onBlur={() => finish(true)}
      onKeyDown={(e) => {
        e.stopPropagation();
        if (e.key === "Escape" || e.key === "Tab") { e.preventDefault(); finish(true); }
        else if (e.key === "Enter" && (!multiline || !e.shiftKey)) { e.preventDefault(); finish(true); }
      }}
      className="absolute resize-none overflow-hidden rounded-md border-0 p-0 text-center font-medium outline-none"
      style={{
        left: rect.x * k + view.x,
        top: rect.y * k + view.y,
        width: rect.w * k,
        height: rect.h * k,
        fontSize: fontSize * k,
        lineHeight: 1.35,
        color: fg,
        background: bg,
        boxShadow: "0 0 0 2px var(--rv-accent)",
        paddingTop: Math.max(0, (rect.h * k - fontSize * k * 1.35 * Math.max(1, value.split("\n").length)) / 2),
        fontFamily: "inherit",
      }}
    />
  );
}

// ---- context menu ----------------------------------------------------------------------------
function CanvasMenu({ target }: { target: MenuTarget }) {
  const sel = useStore((s) => s.selection);
  const item = "text-sm";
  if (target.kind === "canvas") {
    return (
      <ContextMenuContent className="w-52">
        <ContextMenuItem className={item} onSelect={() => A.addNodeAt(target.at.x, target.at.y)}>Add step here</ContextMenuItem>
        <ContextMenuItem className={item} onSelect={A.selectAll}>Select all<ContextMenuShortcut>⌘A</ContextMenuShortcut></ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuItem className={item} onSelect={A.autoLayout}>Auto-layout<ContextMenuShortcut>⇧⌘L</ContextMenuShortcut></ContextMenuItem>
        <ContextMenuItem className={item} onSelect={() => A.fit()}>Fit to screen<ContextMenuShortcut>⌘0</ContextMenuShortcut></ContextMenuItem>
      </ContextMenuContent>
    );
  }
  const many = sel.nodes.length > 1;
  return (
    <ContextMenuContent className="w-52">
      {target.kind === "node" && !many && (
        <ContextMenuItem className={item} onSelect={() => setState({ editing: { kind: "node", id: sel.nodes[0] } })}>Edit label<ContextMenuShortcut>↵</ContextMenuShortcut></ContextMenuItem>
      )}
      {target.kind === "edge" && (
        <ContextMenuItem className={item} onSelect={() => setState({ editing: { kind: "edge", id: sel.edges[0] } })}>Edit label<ContextMenuShortcut>↵</ContextMenuShortcut></ContextMenuItem>
      )}
      {target.kind === "node" && (
        <>
          <ContextMenuItem className={item} onSelect={A.duplicateSelection}>Duplicate<ContextMenuShortcut>⌘D</ContextMenuShortcut></ContextMenuItem>
          <ContextMenuItem className={item} onSelect={A.groupSelection}>Group<ContextMenuShortcut>⌘G</ContextMenuShortcut></ContextMenuItem>
          {!many && <ContextMenuItem className={item} onSelect={() => { A.addTask("New task", sel.nodes[0]); A.togglePanel("tasks"); }}>Add task</ContextMenuItem>}
        </>
      )}
      {target.kind === "group" && <ContextMenuItem className={item} onSelect={() => A.ungroup(sel.groups[0])}>Ungroup</ContextMenuItem>}
      <ContextMenuSeparator />
      <ContextMenuItem variant="destructive" className={item} onSelect={A.deleteSelection}>Delete<ContextMenuShortcut>⌫</ContextMenuShortcut></ContextMenuItem>
    </ContextMenuContent>
  );
}
