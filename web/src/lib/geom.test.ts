import { describe, expect, it } from "vitest";
import { SNAP, centerOn, clampK, fitView, marqueeHits, mergeSelection, rectFromPoints, snap, snapDelta, toScreen, toWorld, zoomAt } from "./geom";

describe("snapping", () => {
  it("rounds to the 8px grid", () => {
    expect(SNAP).toBe(8);
    expect(snap(0)).toBe(0);
    expect(snap(3.9)).toBe(0);
    expect(snap(4)).toBe(8);
    expect(snap(-5)).toBe(-8);
    expect(snap(23, 24)).toBe(24);
  });

  it("snaps the dragged anchor, so every node moves by the same delta", () => {
    const d = snapDelta({ x: 13, y: 5 }, 10, 10);
    expect({ x: 13 + d.x, y: 5 + d.y }).toEqual({ x: 24, y: 16 });
    expect(d).toEqual({ x: 11, y: 11 });
  });

  it("Alt disables snapping", () => {
    expect(snapDelta({ x: 13, y: 5 }, 10.5, 3, true)).toEqual({ x: 10.5, y: 3 });
  });

  it("a zero drag of an off-grid node snaps it onto the grid", () => {
    expect(snapDelta({ x: 13, y: 5 }, 0, 0)).toEqual({ x: 3, y: 3 });
  });
});

describe("marquee", () => {
  const items = [
    { id: "a", rect: { x: 0, y: 0, w: 100, h: 50 } },
    { id: "b", rect: { x: 200, y: 0, w: 100, h: 50 } },
    { id: "c", rect: { x: 0, y: 200, w: 100, h: 50 } },
  ];

  it("builds a rect from any two corners", () => {
    expect(rectFromPoints({ x: 10, y: 40 }, { x: 4, y: 8 })).toEqual({ x: 4, y: 8, w: 6, h: 32 });
  });

  it("selects every node it touches, not only those it contains", () => {
    expect(marqueeHits(items, { x: 90, y: 40, w: 120, h: 20 })).toEqual(["a", "b"]);
    expect(marqueeHits(items, { x: -10, y: -10, w: 400, h: 400 })).toEqual(["a", "b", "c"]);
    expect(marqueeHits(items, { x: 150, y: 100, w: 20, h: 20 })).toEqual([]);
  });

  it("treats a tiny box as a click", () => {
    expect(marqueeHits(items, { x: 10, y: 10, w: 1, h: 1 })).toEqual([]);
  });

  it("shift adds to the selection without duplicates", () => {
    expect(mergeSelection(["a"], ["a", "b"], true)).toEqual(["a", "b"]);
    expect(mergeSelection(["a"], ["b"], false)).toEqual(["b"]);
  });
});

describe("viewport", () => {
  it("round-trips screen and world", () => {
    const v = { x: 40, y: -20, k: 2 };
    const p = toWorld(v, 140, 80);
    expect(p).toEqual({ x: 50, y: 50 });
    expect(toScreen(v, p)).toEqual({ x: 140, y: 80 });
  });

  it("zooms around the cursor", () => {
    const v = { x: 10, y: 10, k: 1 };
    const before = toWorld(v, 300, 200);
    const z = zoomAt(v, 2, 300, 200);
    expect(z.k).toBe(2);
    const after = toWorld(z, 300, 200);
    expect(after.x).toBeCloseTo(before.x);
    expect(after.y).toBeCloseTo(before.y);
  });

  it("clamps zoom", () => {
    expect(clampK(100)).toBe(4);
    expect(clampK(0.001)).toBe(0.1);
    expect(zoomAt({ x: 0, y: 0, k: 4 }, 2, 0, 0).k).toBe(4);
  });

  it("fits bounds inside the free area and centres them", () => {
    const inset = { left: 100, top: 50, right: 100, bottom: 50 };
    const v = fitView({ x: 0, y: 0, w: 400, h: 200 }, 1000, 600, inset);
    expect(v.k).toBe(1.25); // capped; the free area (800x500) is bigger than the diagram
    const centre = toScreen(v, { x: 200, y: 100 });
    expect(centre.x).toBeCloseTo(500);
    expect(centre.y).toBeCloseTo(300);
    expect(fitView({ x: 0, y: 0, w: 4000, h: 200 }, 1000, 600, inset).k).toBeCloseTo(0.2);
  });

  it("centres on a point", () => {
    const v = centerOn({ x: 0, y: 0, k: 2 }, { x: 50, y: 50 }, 1000, 600, { left: 0, top: 0, right: 200, bottom: 0 });
    expect(toScreen(v, { x: 50, y: 50 })).toEqual({ x: 400, y: 300 });
  });
});
