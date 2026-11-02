import { describe, expect, it } from "vitest";
import { dragBox, ghostRect, handlePoints, nudgeInto, oppositeSide, pickHandle, planQuickCreate, quickCreateOrigin, sideAnchor } from "./quickCreate";
import type { Diagram, Node } from "./types";

const r = { x: 100, y: 50, w: 120, h: 40 };
const node = (id: string, extra: Partial<Node> = {}): Node => ({ id, label: id, shape: "rounded", pos: { x: 100, y: 50 }, size: null, group: "g1", role: "accent", icon: "user", ...extra });
const diagram = (n: Node): Diagram => ({ title: "", kind: "flowchart", direction: "down", brand: {} as Diagram["brand"], nodes: [n], edges: [], groups: [{ id: "g1", label: "G", role: "neutral" }], tasks: [] });
const measure = (n: Node) => (n.size ? n.size : n.shape === "rounded" && n.label === "" ? { w: 100, h: 48 } : { w: 120, h: 40 });

describe("quick create (ported)", () => {
  it("anchors and opposite sides", () => {
    expect(sideAnchor(r, "top")).toEqual({ x: 160, y: 50 });
    expect(sideAnchor(r, "right")).toEqual({ x: 220, y: 70 });
    expect(oppositeSide("left")).toBe("right");
  });

  it("puts handles 22 screen px outside each side, at any zoom", () => {
    const pts = handlePoints(r, 2);
    expect(pts.find((p) => p.side === "top")!.point).toEqual({ x: 160, y: 39 });
    expect(pts.find((p) => p.side === "left")!.point).toEqual({ x: 89, y: 70 });
    expect(pickHandle(r, { x: 160, y: 40 }, 2)).toBe("top");
    expect(pickHandle(r, { x: 160, y: 70 }, 2)).toBeNull();
  });

  it("places the next node one full size away", () => {
    expect(quickCreateOrigin(r, "right")).toEqual({ x: 340, y: 50 });
    expect(quickCreateOrigin(r, "left")).toEqual({ x: -140, y: 50 });
    expect(quickCreateOrigin(r, "bottom")).toEqual({ x: 100, y: 130 });
    expect(quickCreateOrigin(r, "top")).toEqual({ x: 100, y: -30 });
    expect(ghostRect(r, "bottom")).toEqual({ x: 100, y: 130, w: 120, h: 40 });
  });

  it("copies kind and look with a blank label, plus the arrow, centred on the ghost", () => {
    const q = planQuickCreate(diagram(node("a", { size: { w: 120, h: 40 } })), "a", r, "right", measure)!;
    const made = q.diagram.nodes[1];
    expect(made).toMatchObject({ id: q.nodeId, label: "", shape: "rounded", role: "accent", icon: "user", group: "g1", size: { w: 120, h: 40 } });
    expect(made.pos).toEqual({ x: 340, y: 50 });
    expect(q.diagram.edges).toEqual([{ id: q.edgeId, from: "a", to: q.nodeId, label: "", line: "solid", arrow: "end", role: null }]);
    expect(q.edgeId).not.toBe(q.nodeId);
  });

  it("a decision leads to a plain step of default size, still centred on the axis", () => {
    const q = planQuickCreate(diagram(node("d", { shape: "diamond", icon: null })), "d", r, "bottom", measure)!;
    const made = q.diagram.nodes[1];
    expect(made.shape).toBe("rounded");
    expect(made.size).toBeNull();
    // centre of the same-size ghost: (160, 150); a 100x48 box centred there
    expect(made.pos).toEqual({ x: 110, y: 126 });
  });

  it("a drag released on empty canvas centres the new node on the drop", () => {
    const q = planQuickCreate(diagram(node("a")), "a", r, "right", measure, { x: 500, y: 300 })!;
    expect(q.diagram.nodes[1].pos).toEqual({ x: 450, y: 276 });
  });

  it("a missing source makes nothing", () => {
    expect(planQuickCreate(diagram(node("a")), "zz", r, "right", measure)).toBeNull();
  });

  it("the Add step drag box snaps and squares with Shift, from any corner", () => {
    expect(dragBox({ x: 0, y: 0 }, { x: 101, y: 45 }, false)).toEqual({ x: 0, y: 0, w: 104, h: 48 });
    expect(dragBox({ x: 0, y: 0 }, { x: -101, y: -45 }, false)).toEqual({ x: -104, y: -48, w: 104, h: 48 });
    expect(dragBox({ x: 0, y: 0 }, { x: 101, y: 45 }, true)).toEqual({ x: 0, y: 0, w: 104, h: 104 });
    expect(dragBox({ x: 0, y: 0 }, { x: 1, y: 1 }, false)).toEqual({ x: 0, y: 0, w: 8, h: 8 });
  });

  it("pans the least to clear the chrome, keeping zoom", () => {
    const free = { left: 316, top: 72, width: 700, height: 600 };
    expect(nudgeInto({ x: 400, y: 200, w: 100, h: 50 }, { x: 0, y: 0, k: 1 }, free)).toBeNull();
    expect(nudgeInto({ x: 100, y: 200, w: 100, h: 50 }, { x: 0, y: 0, k: 1 }, free)).toEqual({ k: 1, x: 232, y: 0 });
    expect(nudgeInto({ x: 400, y: 700, w: 100, h: 50 }, { x: 0, y: 0, k: 2 }, free)).toEqual({ k: 2, x: 0, y: -844 });
  });
});
