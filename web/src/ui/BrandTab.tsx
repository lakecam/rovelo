// Brand tab: presets, role palette, surfaces, type, shape, routing, fill, logo, with a live preview
// rendered by the core's own SVG exporter. Edits are batched so one drag is one undo step.
import { useMemo, useState } from "react";
import { core } from "@/wasm";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { InsetSlider } from "@/components/InsetSlider";
import { Switch } from "@/components/ui/switch";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { ROLES, type BrandKit, type Diagram, type EdgeRouting, type NodeFill, type Role, type Rgb } from "@/lib/types";
import { swapExportIcons } from "@/lib/nodeIcons";
import { openText } from "@/lib/files";
import * as A from "@/state/actions";
import { useBurst } from "@/state/burst";
import { edit, toast, useStore } from "@/state/store";
import { Segmented, SectionLabel, Swatch, hexOf, parseHex, rgbCss } from "./kit";
import { Icon } from "./icons";
import { cn } from "@/lib/utils";

const FONTS: { label: string; stack: string }[] = [
  { label: "Inter", stack: "Inter, -apple-system, 'Segoe UI', sans-serif" },
  { label: "System UI", stack: "system-ui, -apple-system, 'Segoe UI', sans-serif" },
  { label: "Georgia", stack: "Georgia, 'Times New Roman', serif" },
  { label: "Helvetica Neue", stack: "'Helvetica Neue', Helvetica, Arial, sans-serif" },
  { label: "Avenir Next", stack: "'Avenir Next', Avenir, sans-serif" },
  { label: "Menlo", stack: "Menlo, 'SF Mono', Consolas, monospace" },
];

/** A three-step sample drawn with the kit, small enough to preview live. */
export function previewDiagram(brand: BrandKit): Diagram {
  const n = (id: string, label: string, x: number, y: number, role: Role, shape: "rounded" | "pill" | "diamond" = "rounded") =>
    ({ id, label, shape, pos: { x, y }, size: null, group: null, role, icon: null });
  const e = (id: string, from: string, to: string, label = "") => ({ id, from, to, label, line: "solid" as const, arrow: "end" as const, role: null });
  return {
    title: "", kind: "flowchart", direction: "right", brand,
    nodes: [n("a", "Start", 0, 20, "primary", "pill"), n("b", "Review", 150, 20, "secondary"), n("c", "Ship", 300, 20, "success", "pill"), n("d", "Fix", 150, 110, "danger")],
    edges: [e("e1", "a", "b"), e("e2", "b", "c", "ok"), e("e3", "b", "d")],
    groups: [], tasks: [],
  };
}

export function BrandTab() {
  const brand = useStore((s) => s.diagram.brand);
  const burst = useBurst();
  const drag = useBurst(null); // sliders: one drag, one undo step
  const [role, setRole] = useState<Role>("primary");
  const set = (f: (b: BrandKit) => BrandKit) => burst.run((d) => ({ ...d, brand: f(d.brand) }));
  const slide = (f: (b: BrandKit) => BrandKit) => drag.run((d) => ({ ...d, brand: f(d.brand) }));
  const presets = useMemo(() => core.brandPresets(), []);
  const preview = useMemo(() => {
    const d = previewDiagram(brand);
    return swapExportIcons(core.toSvg(d), d);
  }, [brand]);
  const house = useMemo(() => core.defaultBrand(), []);
  const fontKnown = (s: string) => FONTS.some((f) => f.stack === s);

  const setRgb = (key: "background" | "surface" | "text" | "muted_text", c: Rgb) => set((b) => ({ ...b, [key]: c }));

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto p-4 pt-0">
      <div
        className="shrink-0 overflow-hidden rounded-lg border border-hair [&>svg]:block [&>svg]:h-auto [&>svg]:w-full"
        aria-label="Brand preview"
        role="img"
        dangerouslySetInnerHTML={{ __html: preview }}
      />

      <Section label="Presets">
        <div className="grid grid-cols-2 gap-2">
          {presets.map((p) => (
            <button
              key={p.name}
              onClick={() => A.setBrand(p)}
              aria-pressed={brand.name === p.name}
              className={cn("rounded-lg border bg-surface p-2 text-left transition-[border-color,background-color] duration-150 hover:bg-accent", brand.name === p.name ? "border-hair-strong bg-accent" : "border-hair")}
            >
              <div className="flex -space-x-1">
                {(["primary", "secondary", "accent", "danger"] as const).map((r) => (
                  <span key={r} className="size-4 rounded-full border border-surface" style={{ background: rgbCss(p.palette[r]) }} />
                ))}
              </div>
              <div className="mt-2 text-sm font-medium">{p.name}</div>
            </button>
          ))}
        </div>
      </Section>

      <Section label="Name">
        <Input aria-label="Brand name" value={brand.name} onChange={(e) => set((b) => ({ ...b, name: e.target.value }))} />
      </Section>

      <Section label="Palette">
        <div className="flex flex-wrap gap-2">
          {ROLES.map((r) => (
            <Swatch key={r} label={r[0].toUpperCase() + r.slice(1)} color={rgbCss(brand.palette[r])} selected={role === r} onClick={() => setRole(r)} />
          ))}
        </div>
        <ColorField label={`${role[0].toUpperCase()}${role.slice(1)} colour`} value={brand.palette[role]} onChange={(c) => set((b) => ({ ...b, palette: { ...b.palette, [role]: c } }))} />
      </Section>

      <Section label="Surfaces">
        <div className="flex flex-col gap-2">
          <ColorField compact label="Background" value={brand.background} onChange={(c) => setRgb("background", c)} />
          <ColorField compact label="Surface" value={brand.surface} onChange={(c) => setRgb("surface", c)} />
          <ColorField compact label="Text" value={brand.text} onChange={(c) => setRgb("text", c)} />
          <ColorField compact label="Muted text" value={brand.muted_text} onChange={(c) => setRgb("muted_text", c)} />
        </div>
      </Section>

      <Section label="Type">
        <FontSelect label="Body font" value={brand.font_family} known={fontKnown(brand.font_family)} onChange={(v) => set((b) => ({ ...b, font_family: v }))} />
        <FontSelect label="Heading font" value={brand.heading_family} known={fontKnown(brand.heading_family)} onChange={(v) => set((b) => ({ ...b, heading_family: v }))} />
        <Range label="Text size" defaultValue={house.font_size} value={brand.font_size} min={10} max={24} step={1} unit="px" onChange={(v) => slide((b) => ({ ...b, font_size: v }))} onCommit={drag.end} />
      </Section>

      <Section label="Shape">
        <Range label="Corner radius" defaultValue={house.corner_radius} value={brand.corner_radius} min={0} max={24} step={1} unit="px" onChange={(v) => slide((b) => ({ ...b, corner_radius: v }))} onCommit={drag.end} />
        <Range label="Line weight" defaultValue={house.stroke_width} value={brand.stroke_width} min={0.5} max={4} step={0.5} unit="px" onChange={(v) => slide((b) => ({ ...b, stroke_width: v }))} onCommit={drag.end} />
        <Segmented<EdgeRouting> label="Edge routing" value={brand.edge_routing} onChange={(v) => set((b) => ({ ...b, edge_routing: v }))} options={[
          { value: "straight", label: "Straight" }, { value: "orthogonal", label: "Elbow" }, { value: "curved", label: "Curved" },
        ]} />
        <Segmented<NodeFill> label="Node fill" value={brand.node_fill} onChange={(v) => set((b) => ({ ...b, node_fill: v }))} options={[
          { value: "solid", label: "Solid" }, { value: "soft", label: "Soft" }, { value: "outline", label: "Outline" },
        ]} />
        <Toggle label="Drop shadow" checked={brand.shadow} onChange={(v) => edit((d) => ({ ...d, brand: { ...d.brand, shadow: v } }))} />
        <Toggle label="Dot grid in exports" checked={brand.show_grid} onChange={(v) => edit((d) => ({ ...d, brand: { ...d.brand, show_grid: v } }))} />
      </Section>

      <Section label="Logo">
        <div className="flex items-center gap-2">
          <Button variant="secondary" size="sm" onClick={async () => {
            const f = await openText("svg");
            if (!f) return;
            if (!/<svg[\s>]/i.test(f.text)) return toast("That file is not an SVG");
            edit((d) => ({ ...d, brand: { ...d.brand, logo_svg: f.text } }));
          }}>{brand.logo_svg ? "Replace SVG…" : "Choose SVG…"}</Button>
          {brand.logo_svg && <Button variant="ghost" size="sm" onClick={() => edit((d) => ({ ...d, brand: { ...d.brand, logo_svg: null } }))}>Remove</Button>}
        </div>
        <p className="text-xs font-medium text-ink-muted">Drawn top-right of exports.</p>
      </Section>

      <div className="flex shrink-0 gap-2">
        <Button variant="secondary" size="sm" className="flex-1" onClick={A.importBrand}><Icon name="open" />Load .brand.json</Button>
        <Button variant="secondary" size="sm" className="flex-1" onClick={A.exportBrand}><Icon name="save" />Save</Button>
      </div>
    </div>
  );
}

function Section({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <section className="flex shrink-0 flex-col gap-2">
      <SectionLabel>{label}</SectionLabel>
      {children}
    </section>
  );
}

function ColorField({ label, value, onChange, compact }: { label: string; value: Rgb; onChange: (c: Rgb) => void; compact?: boolean }) {
  const hex = hexOf(value);
  const [draft, setDraft] = useState<string | null>(null);
  return (
    <label className={cn("flex items-center gap-2", compact ? "h-8 rounded-lg border border-hair bg-sunk px-2" : "")}>
      <span className="relative size-6 shrink-0 overflow-hidden rounded-full border border-black/15 dark:border-white/20" style={{ background: hex }}>
        <input type="color" aria-label={label} value={hex} onChange={(e) => { const c = parseHex(e.target.value); if (c) onChange(c); }} className="absolute inset-0 size-full cursor-pointer opacity-0" />
      </span>
      {compact && <span className="min-w-0 flex-1 truncate text-xs text-ink-muted">{label}</span>}
      <input
        aria-label={`${label} hex`}
        value={draft ?? hex}
        spellCheck={false}
        onChange={(e) => { setDraft(e.target.value); const c = parseHex(e.target.value); if (c) onChange(c); }}
        onBlur={() => setDraft(null)}
        className="h-6 w-24 rounded-md border border-transparent bg-transparent px-1 font-mono text-[13px] outline-none hover:border-hair focus-visible:border-brand"
      />
    </label>
  );
}

function FontSelect({ label, value, known, onChange }: { label: string; value: string; known: boolean; onChange: (v: string) => void }) {
  return (
    <div className="flex items-center gap-2">
      <span className="w-24 shrink-0 text-sm text-ink-muted">{label}</span>
      <Select value={value} onValueChange={onChange}>
        <SelectTrigger size="sm" aria-label={label} className="min-w-0 flex-1"><SelectValue /></SelectTrigger>
        <SelectContent>
          {FONTS.map((f) => <SelectItem key={f.stack} value={f.stack}>{f.label}</SelectItem>)}
          {!known && <SelectItem value={value}>Custom</SelectItem>}
        </SelectContent>
      </Select>
    </div>
  );
}

function Range({ label, value, min, max, step, unit, defaultValue, onChange, onCommit }: { label: string; value: number; min: number; max: number; step: number; unit: string; defaultValue: number; onChange: (v: number) => void; onCommit: () => void }) {
  return <InsetSlider label={label} value={value} min={min} max={max} step={step} unit={unit} defaultValue={defaultValue} onChange={onChange} onCommit={onCommit} />;
}

function Toggle({ label, checked, onChange }: { label: string; checked: boolean; onChange: (v: boolean) => void }) {
  return (
    <label className="flex h-8 items-center justify-between text-sm">
      {label}
      <Switch checked={checked} onCheckedChange={onChange} aria-label={label} />
    </label>
  );
}
