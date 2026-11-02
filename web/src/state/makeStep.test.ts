// Making a step end to end through the store and the real core: plus-handle click, type, commit = one undo step.
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { beforeAll, describe, expect, it } from "vitest";

const pkg = path.resolve(import.meta.dirname, "../wasm/pkg/rovelo_wasm_bg.wasm");

describe.skipIf(!existsSync(pkg))("making a step", () => {
  let S: typeof import("./store");
  let A: typeof import("./actions");
  beforeAll(async () => {
    const w = await import("@/wasm");
    await w.loadCore(readFileSync(pkg));
    S = await import("./store");
    A = await import("./actions");
    S.initStore(w.core.template("taskflow", "2026-11-02"));
  });

  it("click a handle, type Hello, Enter: a connected step labelled Hello, one undo step", () => {
    const before = S.getState().diagram;
    const id = A.quickCreate("ship", "right")!;
    const st = S.getState();
    expect(st.editing).toEqual({ kind: "node", id, isNew: true });
    expect(st.selection.nodes).toEqual([id]);
    expect(st.tool).toBe("select");
    // what the inline editor does on Enter for a new step
    S.amend((d) => ({ ...d, nodes: d.nodes.map((n) => (n.id === id ? { ...n, label: "Hello" } : n)) }));
    const after = S.getState().diagram;
    expect(after.nodes.find((n) => n.id === id)!.label).toBe("Hello");
    expect(after.edges.some((e) => e.from === "ship" && e.to === id)).toBe(true);
    expect(S.getState().history.past.length).toBe(1);
    A.undo();
    expect(S.getState().diagram).toBe(before);
  });

  it("from a decision it makes a plain step", () => {
    const id = A.quickCreate("approved", "bottom")!;
    expect(S.getState().diagram.nodes.find((n) => n.id === id)!.shape).toBe("rounded");
  });
});
