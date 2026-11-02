// Calls the real compiled core (needs `pnpm wasm` to have run once). Skipped when the build is absent.
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it, beforeAll } from "vitest";

const pkg = path.resolve(import.meta.dirname, "pkg/rovelo_wasm_bg.wasm");
const built = existsSync(pkg);

describe.skipIf(!built)("rust core through wasm", () => {
  let core: typeof import("./index").core;
  beforeAll(async () => {
    const m = await import("./index");
    await m.loadCore(readFileSync(pkg));
    core = m.core;
  });

  it("parses, lays out and serialises", () => {
    const p = core.parse("taskflow right\nA -> B -> C");
    expect(p.diagram.nodes.length).toBe(3);
    const laid = core.layoutUnplaced(p.diagram);
    expect(laid.nodes.every((n) => n.pos)).toBe(true);
    expect(core.serialize(laid)).toContain("A");
  });

  it("builds a scene whose parts match the diagram", () => {
    const d = core.template("taskflow", "2026-11-02");
    const s = core.scene(d);
    expect(s.nodes.length).toBe(d.nodes.length);
    expect(s.edges.length).toBe(d.edges.length);
    expect(s.nodes[0].svg).toContain("<");
    expect(core.toSvg(d).startsWith("<svg")).toBe(true);
  });

  it("reports parse problems as data and bad JSON as an error", () => {
    expect(core.parse("A [shape: blob]").diagnostics.length).toBeGreaterThan(0);
    expect(() => core.serialize({} as never)).toThrow();
  });

  it("serves brand presets", () => {
    const names = core.brandPresets().map((b) => b.name);
    expect(names).toEqual(expect.arrayContaining(["Slate", "Paper", "Midnight"]));
    expect(core.defaultBrand().name).toBeTruthy();
  });
});
