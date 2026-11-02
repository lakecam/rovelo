import { describe, expect, it } from "vitest";
import * as ops from "./ops";
import { emptySelection, type Diagram, type Node } from "./types";

const brand = {} as Diagram["brand"];
const node = (id: string, x = 0, y = 0, extra: Partial<Node> = {}): Node => ({ id, label: id, shape: "rounded", pos: { x, y }, size: null, group: null, role: "primary", icon: null, ...extra });
const base = (): Diagram => ({
  title: "t", kind: "flowchart", direction: "down", brand,
  nodes: [node("a", 0, 0), node("b", 200, 100)],
  edges: [{ id: "e1", from: "a", to: "b", label: "", line: "solid", arrow: "end", role: null }],
  groups: [],
  tasks: [{ id: "t1", title: "x", done: true, node: "b", due: null, priority: "normal", tags: [], notes: "" }],
});

describe("ops", () => {
  it("mints ids that skip used ones across kinds", () => {
    expect(ops.freshId(base(), "e")).toBe("e2");
    expect(ops.freshId(base(), "t")).toBe("t2");
    expect(ops.freshId(base(), "n", ["n1"])).toBe("n2");
  });

  it("adds a node centred on the point without mutating", () => {
    const d = base();
    const r = ops.addNodeAt(d, { x: 100, y: 100 }, "X", { w: 120, h: 48 });
    expect(d.nodes.length).toBe(2);
    expect(r.diagram.nodes[2].pos).toEqual({ x: 40, y: 76 });
    expect(r.id).toBe("n1");
  });

  it("connects distinct nodes once", () => {
    expect(ops.connect(base(), "a", "a")).toBeNull();
    expect(ops.connect(base(), "a", "zzz")).toBeNull();
    expect(ops.connect(base(), "a", "b")).toBeNull();
    const r = ops.connect(base(), "b", "a")!;
    expect(r.diagram.edges.length).toBe(2);
    expect(r.id).toBe("e2");
  });

  it("deleting a node drops its edges and unlinks (keeps) its tasks", () => {
    const d = ops.deleteSelection(base(), { nodes: ["b"], edges: [], groups: [] });
    expect(d.nodes.map((n) => n.id)).toEqual(["a"]);
    expect(d.edges).toEqual([]);
    expect(d.tasks[0].node).toBeNull();
  });

  it("deleting stale ids is harmless", () => {
    const d = base();
    expect(ops.deleteSelection(d, { nodes: ["nope"], edges: ["nope"], groups: ["nope"] })).toEqual(d);
  });

  it("duplicates with internal edges and fresh, open tasks", () => {
    const r = ops.duplicate(base(), ["a", "b"], { x: 10, y: 10 });
    expect(r.ids).toEqual(["n1", "n2"]);
    expect(r.diagram.edges.length).toBe(2);
    expect(r.diagram.edges[1]).toMatchObject({ from: "n1", to: "n2" });
    expect(r.diagram.nodes[3].pos).toEqual({ x: 210, y: 110 });
    expect(r.diagram.tasks[1]).toMatchObject({ node: "n2", done: false });
  });

  it("groups and ungroups", () => {
    const g = ops.groupNodes(base(), ["a", "b"])!;
    expect(g.diagram.nodes.every((n) => n.group === g.id)).toBe(true);
    const back = ops.removeGroup(g.diagram, g.id);
    expect(back.groups).toEqual([]);
    expect(back.nodes.every((n) => n.group === null)).toBe(true);
    expect(ops.groupNodes(base(), ["zzz"])).toBeNull();
  });

  it("nudges and aligns", () => {
    const n = ops.nudge(base(), ["a"], 8, -8);
    expect(n.nodes[0].pos).toEqual({ x: 8, y: -8 });
    const rects = new Map([["a", { x: 0, y: 0, w: 100, h: 40 }], ["b", { x: 200, y: 100, w: 60, h: 20 }]]);
    const left = ops.alignNodes(base(), ["a", "b"], "right", rects);
    expect(left.nodes[0].pos!.x).toBe(160);
    expect(left.nodes[1].pos!.x).toBe(200);
    const mid = ops.alignNodes(base(), ["a", "b"], "middle", rects);
    expect(mid.nodes[0].pos!.y).toBe(40);
    expect(mid.nodes[1].pos!.y).toBe(50);
    expect(ops.alignNodes(base(), ["a"], "left", rects)).toEqual(base());
  });

  it("adds tasks, ignoring blank titles and unknown nodes", () => {
    expect(ops.addTask(base(), "   ")).toBeNull();
    const r = ops.addTask(base(), " Ship ", "a")!;
    expect(r.diagram.tasks[1]).toMatchObject({ title: "Ship", node: "a", id: "t2" });
    expect(ops.addTask(base(), "x", "zzz")!.diagram.tasks[1].node).toBeNull();
  });

  it("merging a parse keeps brand and positions by id", () => {
    const old = { ...base(), brand: { name: "kept" } as Diagram["brand"] };
    const parsed: Diagram = { ...base(), brand: { name: "default" } as Diagram["brand"], nodes: [node("a", 0, 0, { pos: null }), node("c", 0, 0, { pos: null })], edges: [] };
    old.nodes[0].pos = { x: 50, y: 60 };
    const m = ops.mergeParsed(old, parsed);
    expect(m.brand.name).toBe("kept");
    expect(m.nodes[0].pos).toEqual({ x: 50, y: 60 });
    expect(m.nodes[1].pos).toBeNull();
  });

  it("repair and selection pruning drop dangling references", () => {
    const d = { ...base(), nodes: [node("a")] };
    const r = ops.repair(d);
    expect(r.edges).toEqual([]);
    expect(r.tasks[0].node).toBeNull();
    const sel = { nodes: ["a", "zz"], edges: ["e1"], groups: [] };
    expect(ops.pruneSelection(r, sel)).toEqual({ nodes: ["a"], edges: [], groups: [] });
    const same = emptySelection();
    expect(ops.pruneSelection(r, same)).toBe(same);
  });
});
