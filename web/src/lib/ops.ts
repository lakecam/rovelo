// Plain immutable edits on the Diagram JSON. Rust is only asked for layout, geometry, text and SVG.
import type { Diagram, Edge, Group, Id, Node, Point, Rect, Role, Selection, Task } from "./types";

const usedIds = (d: Diagram) =>
  new Set<string>([...d.nodes.map((n) => n.id), ...d.edges.map((e) => e.id), ...d.groups.map((g) => g.id), ...d.tasks.map((t) => t.id)]);

/** A fresh id with the prefix that no node, edge, group or task uses (same rule as the Rust model). */
export function freshId(d: Diagram, prefix: string, extra: Iterable<string> = []): Id {
  const used = usedIds(d);
  for (const e of extra) used.add(e);
  for (let i = 1; ; i++) if (!used.has(`${prefix}${i}`)) return `${prefix}${i}`;
}

export const findNode = (d: Diagram, id: Id) => d.nodes.find((n) => n.id === id);

export function mapNode(d: Diagram, id: Id, f: (n: Node) => Node): Diagram {
  return { ...d, nodes: d.nodes.map((n) => (n.id === id ? f(n) : n)) };
}
export function mapNodes(d: Diagram, ids: Id[], f: (n: Node) => Node): Diagram {
  const set = new Set(ids);
  return { ...d, nodes: d.nodes.map((n) => (set.has(n.id) ? f(n) : n)) };
}
export function mapEdge(d: Diagram, id: Id, f: (e: Edge) => Edge): Diagram {
  return { ...d, edges: d.edges.map((e) => (e.id === id ? f(e) : e)) };
}
export function mapTask(d: Diagram, id: Id, f: (t: Task) => Task): Diagram {
  return { ...d, tasks: d.tasks.map((t) => (t.id === id ? f(t) : t)) };
}
export function mapGroup(d: Diagram, id: Id, f: (g: Group) => Group): Diagram {
  return { ...d, groups: d.groups.map((g) => (g.id === id ? f(g) : g)) };
}

/** Add a node whose centre is `center`. `size` is the measured size (from the core) or a guess. */
export function addNodeAt(d: Diagram, center: Point, label = "New step", size = { w: 120, h: 48 }): { diagram: Diagram; id: Id } {
  const id = freshId(d, "n");
  const node: Node = {
    id, label, shape: "rounded", role: "primary", icon: null, group: null, size: null,
    pos: { x: center.x - size.w / 2, y: center.y - size.h / 2 },
  };
  return { diagram: { ...d, nodes: [...d.nodes, node] }, id };
}

/** Connect two distinct existing nodes unless that exact edge exists. */
export function connect(d: Diagram, from: Id, to: Id): { diagram: Diagram; id: Id } | null {
  if (from === to || !findNode(d, from) || !findNode(d, to)) return null;
  if (d.edges.some((e) => e.from === from && e.to === to)) return null;
  const id = freshId(d, "e");
  const edge: Edge = { id, from, to, label: "", line: "solid", arrow: "end", role: null };
  return { diagram: { ...d, edges: [...d.edges, edge] }, id };
}

export function removeNodes(d: Diagram, ids: Id[]): Diagram {
  const gone = new Set(ids);
  return {
    ...d,
    nodes: d.nodes.filter((n) => !gone.has(n.id)),
    edges: d.edges.filter((e) => !gone.has(e.from) && !gone.has(e.to)),
    tasks: d.tasks.map((t) => (t.node && gone.has(t.node) ? { ...t, node: null } : t)),
  };
}

export function removeGroup(d: Diagram, id: Id): Diagram {
  return {
    ...d,
    groups: d.groups.filter((g) => g.id !== id),
    nodes: d.nodes.map((n) => (n.group === id ? { ...n, group: null } : n)),
  };
}

/** Delete the selected nodes (with their edges), edges and groups. */
export function deleteSelection(d: Diagram, sel: Selection): Diagram {
  let out = removeNodes(d, sel.nodes);
  const edges = new Set(sel.edges);
  out = { ...out, edges: out.edges.filter((e) => !edges.has(e.id)) };
  for (const g of sel.groups) out = removeGroup(out, g);
  return out;
}

/** Clone nodes (with the edges between them and their open tasks) shifted by `offset`. */
export function duplicate(d: Diagram, ids: Id[], offset: Point = { x: 24, y: 24 }): { diagram: Diagram; ids: Id[] } {
  const map = new Map<Id, Id>();
  const minted: string[] = [];
  const nodes: Node[] = [];
  for (const id of ids) {
    const src = findNode(d, id);
    if (!src) continue;
    const nid = freshId(d, "n", minted);
    minted.push(nid);
    map.set(id, nid);
    nodes.push({ ...src, id: nid, pos: src.pos && { x: src.pos.x + offset.x, y: src.pos.y + offset.y } });
  }
  const edges: Edge[] = [];
  for (const e of d.edges) {
    const f = map.get(e.from), t = map.get(e.to);
    if (!f || !t) continue;
    const eid = freshId(d, "e", minted);
    minted.push(eid);
    edges.push({ ...e, id: eid, from: f, to: t });
  }
  const tasks: Task[] = [];
  for (const t of d.tasks) {
    const n = t.node && map.get(t.node);
    if (!n) continue;
    const tid = freshId(d, "t", minted);
    minted.push(tid);
    tasks.push({ ...t, id: tid, node: n, done: false });
  }
  return {
    diagram: { ...d, nodes: [...d.nodes, ...nodes], edges: [...d.edges, ...edges], tasks: [...d.tasks, ...tasks] },
    ids: nodes.map((n) => n.id),
  };
}

/** Put the nodes in a new group. */
export function groupNodes(d: Diagram, ids: Id[]): { diagram: Diagram; id: Id } | null {
  const members = ids.filter((i) => findNode(d, i));
  if (!members.length) return null;
  const id = freshId(d, "g");
  const g: Group = { id, label: "Group", role: "neutral" };
  return { diagram: mapNodes({ ...d, groups: [...d.groups, g] }, members, (n) => ({ ...n, group: id })), id };
}

export function nudge(d: Diagram, ids: Id[], dx: number, dy: number): Diagram {
  return mapNodes(d, ids, (n) => {
    const p = n.pos ?? { x: 0, y: 0 };
    return { ...n, pos: { x: p.x + dx, y: p.y + dy } };
  });
}

export type Align = "left" | "center" | "right" | "top" | "middle" | "bottom";

/** Align nodes to the union of their rectangles. `rects` come from the core's scene. */
export function alignNodes(d: Diagram, ids: Id[], mode: Align, rects: Map<Id, Rect>): Diagram {
  const rs = ids.map((i) => rects.get(i)).filter((r): r is Rect => !!r);
  if (rs.length < 2) return d;
  const l = Math.min(...rs.map((r) => r.x)), t = Math.min(...rs.map((r) => r.y));
  const r = Math.max(...rs.map((r) => r.x + r.w)), b = Math.max(...rs.map((r) => r.y + r.h));
  return mapNodes(d, ids, (n) => {
    const rc = rects.get(n.id);
    if (!rc) return n;
    let { x, y } = rc;
    if (mode === "left") x = l;
    else if (mode === "right") x = r - rc.w;
    else if (mode === "center") x = (l + r) / 2 - rc.w / 2;
    else if (mode === "top") y = t;
    else if (mode === "bottom") y = b - rc.h;
    else y = (t + b) / 2 - rc.h / 2;
    return { ...n, pos: { x, y } };
  });
}

/** Add a to-do, optionally hanging off a node. Blank titles are ignored. */
export function addTask(d: Diagram, title: string, node: Id | null = null): { diagram: Diagram; id: Id } | null {
  const t = title.trim();
  if (!t) return null;
  const id = freshId(d, "t");
  const task: Task = {
    id, title: t, done: false, node: node && findNode(d, node) ? node : null,
    due: null, priority: "normal", tags: [], notes: "",
  };
  return { diagram: { ...d, tasks: [...d.tasks, task] }, id };
}

export function removeTask(d: Diagram, id: Id): Diagram {
  return { ...d, tasks: d.tasks.filter((t) => t.id !== id) };
}

/** Drop edges and links that point at things that no longer exist. */
export function repair(d: Diagram): Diagram {
  const ids = new Set(d.nodes.map((n) => n.id));
  const gids = new Set(d.groups.map((g) => g.id));
  return {
    ...d,
    edges: d.edges.filter((e) => ids.has(e.from) && ids.has(e.to)),
    nodes: d.nodes.map((n) => (n.group && !gids.has(n.group) ? { ...n, group: null } : n)),
    tasks: d.tasks.map((t) => (t.node && !ids.has(t.node) ? { ...t, node: null } : t)),
  };
}

/** Rebuild a parsed diagram on top of the current one: brand and positions are kept by id. */
export function mergeParsed(old: Diagram, parsed: Diagram): Diagram {
  const nodes = parsed.nodes.map((n) => {
    const o = findNode(old, n.id);
    return o ? { ...n, pos: n.pos ?? o.pos, size: n.size ?? o.size } : n;
  });
  return repair({ ...parsed, brand: old.brand, nodes });
}

export function setRoleOnSelection(d: Diagram, sel: Selection, role: Role): Diagram {
  let out = mapNodes(d, sel.nodes, (n) => ({ ...n, role }));
  const es = new Set(sel.edges);
  out = { ...out, edges: out.edges.map((e) => (es.has(e.id) ? { ...e, role } : e)) };
  const gs = new Set(sel.groups);
  return { ...out, groups: out.groups.map((g) => (gs.has(g.id) ? { ...g, role } : g)) };
}

export function pruneSelection(d: Diagram, sel: Selection): Selection {
  const nodes = sel.nodes.filter((i) => findNode(d, i));
  const edges = sel.edges.filter((i) => d.edges.some((e) => e.id === i));
  const groups = sel.groups.filter((i) => d.groups.some((g) => g.id === i));
  return nodes.length === sel.nodes.length && edges.length === sel.edges.length && groups.length === sel.groups.length
    ? sel
    : { nodes, edges, groups };
}
