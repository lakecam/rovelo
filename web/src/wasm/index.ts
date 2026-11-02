// Typed bridge to the Rust core (crates/rovelo-wasm). Diagrams cross as JSON strings.
import init, * as w from "./pkg/rovelo_wasm.js";
import wasmUrl from "./pkg/rovelo_wasm_bg.wasm?url";
import type { BrandKit, Diagram, Parsed, Point, Rect, Scene, Size } from "@/lib/types";

let ready: Promise<void> | null = null;

/** Load the module once; call before first use (main.tsx awaits it). */
export function loadCore(bytes?: BufferSource): Promise<void> {
  ready ??= init({ module_or_path: bytes ?? wasmUrl }).then(() => undefined);
  return ready;
}

const j = <T>(s: string) => JSON.parse(s) as T;
const s = (d: Diagram) => JSON.stringify(d);

export const core = {
  parse: (text: string) => j<Parsed>(w.parse(text)),
  fromMermaid: (text: string) => j<Parsed>(w.from_mermaid(text)),
  serialize: (d: Diagram) => w.serialize(s(d)),
  repair: (d: Diagram) => j<Diagram>(w.repair(s(d))),
  autoLayout: (d: Diagram) => j<Diagram>(w.auto_layout(s(d))),
  layoutUnplaced: (d: Diagram) => j<Diagram>(w.layout_unplaced(s(d))),
  toSvg: (d: Diagram) => w.to_svg(s(d)),
  scene: (d: Diagram) => j<Scene>(w.scene(s(d))),
  nodeRect: (d: Diagram, id: string) => j<Rect>(w.node_rect(s(d), id)),
  nodeSize: (d: Diagram, id: string) => j<Size>(w.node_size(s(d), id)),
  edgePath: (d: Diagram, id: string) => j<{ points: Point[]; label_at: Point }>(w.edge_path(s(d), id)),
  groupRect: (d: Diagram, id: string) => j<Rect | null>(w.group_rect(s(d), id)),
  diagramBounds: (d: Diagram, margin: number) => j<Rect>(w.diagram_bounds(s(d), margin)),
  brandPresets: () => j<BrandKit[]>(w.brand_presets()),
  validateBrand: (text: string) => j<BrandKit>(w.validate_brand(text)),
  nodeColors: (b: BrandKit, role: string) => j<{ fill: string; stroke: string; text: string }>(w.node_colors(JSON.stringify(b), role)),
  iconNames: () => j<string[]>(w.icon_names()),
  defaultDiagram: () => j<Diagram>(w.default_diagram()),
  /** The house kit: the core's "Rovelo" preset when it has one, else its first preset. */
  defaultBrand: (): BrandKit => {
    const all = j<BrandKit[]>(w.brand_presets());
    return all.find((b) => b.name === "Rovelo") ?? all[0];
  },
  /** An empty diagram wearing the house kit. */
  freshDiagram: (): Diagram => ({ ...j<Diagram>(w.default_diagram()), brand: core.defaultBrand() }),
  template: (name: "userflow" | "taskflow" | "blank", today: string, brand?: BrandKit) =>
    j<Diagram>(w.template(name, today, brand ? JSON.stringify(brand) : "")),
};
