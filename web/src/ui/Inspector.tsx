// Floating inspector at the right edge. Only exists while something is selected; no heavy header.
import { useMemo } from "react";
import { core } from "@/wasm";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { mapEdge, mapGroup, mapNode, mapNodes, setRoleOnSelection } from "@/lib/ops";
import { ROLES, SHAPES, type Arrow, type Diagram, type LineStyle, type Role, type Shape } from "@/lib/types";
import * as A from "@/state/actions";
import { edit, setState, useStore } from "@/state/store";
import { IconBtn, Segmented, SectionLabel, Swatch, rgbCss } from "./kit";
import { Icon } from "./icons";
import { SlotReveal } from "./slots";
import { canonicalIcon } from "@/lib/nodeIcons";

const cap = (s: string) => s[0].toUpperCase() + s.slice(1);
const SIZES = [{ k: "S", f: 1 }, { k: "M", f: 1.25 }, { k: "L", f: 1.6 }, { k: "XL", f: 2 }] as const;
const ICON_FILE: Record<string, string> = { check: "done" };

export function Inspector() {
  const selection = useStore((s) => s.selection);
  const count = selection.nodes.length + selection.edges.length + selection.groups.length;
  if (!count) return null;
  const kind = selection.nodes.length ? "node" : selection.edges.length ? "edge" : "group";
  const title = count > 1 ? `${count} selected` : cap(kind === "node" ? "step" : kind);
  return (
    <SlotReveal className="pointer-events-none absolute bottom-[72px] right-4 top-[72px] flex w-[280px] flex-col justify-start">
      <aside aria-label="Inspector" className="rv-panel pointer-events-auto flex max-h-full flex-col rounded-2xl">
        <div className="flex shrink-0 items-center justify-between px-4 pt-4">
          <h2 className="text-base font-semibold leading-8">{title}</h2>
          <IconBtn label="Close inspector" keys="Esc" side="left" onClick={A.clearSelection}><Icon name="close" /></IconBtn>
        </div>
        <div className="flex min-h-0 flex-col gap-4 overflow-y-auto p-4">
          {kind === "node" && <NodeInspector />}
          {kind === "edge" && <EdgeInspector />}
          {kind === "group" && <GroupInspector />}
        </div>
      </aside>
    </SlotReveal>
  );
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex flex-col gap-2">
      <SectionLabel>{label}</SectionLabel>
      {children}
    </div>
  );
}

function RoleSwatches({ value, onPick, neutralDefault }: { value: Role | null; onPick: (r: Role | null) => void; neutralDefault?: boolean }) {
  const brand = useStore((s) => s.diagram.brand);
  return (
    <div className="flex flex-wrap gap-2" role="group" aria-label="Colour">
      {ROLES.map((r) => (
        <Swatch key={r} label={cap(r)} color={rgbCss(brand.palette[r])} selected={(value ?? (neutralDefault ? "neutral" : null)) === r} onClick={() => onPick(neutralDefault && r === "neutral" ? null : r)} size={24} />
      ))}
    </div>
  );
}

function NodeInspector() {
  const diagram = useStore((s) => s.diagram);
  const selection = useStore((s) => s.selection);
  const ids = selection.nodes;
  const nodes = ids.map((id) => diagram.nodes.find((n) => n.id === id)).filter((n) => !!n);
  const one = nodes.length === 1 ? nodes[0] : null;
  const names = useMemo(() => core.iconNames(), []);
  const role = nodes.every((n) => n.role === nodes[0].role) ? nodes[0].role : null;

  // Size presets multiply the measured size, S being the measured size itself.
  const base = useMemo(() => {
    if (!one) return null;
    return core.nodeSize(mapNode(diagram, one.id, (n) => ({ ...n, size: null })), one.id);
  }, [diagram, one]);
  const sizeKey = one && base ? (one.size ? SIZES.find((s) => Math.abs(one.size!.w / base.w - s.f) < 0.03)?.k ?? null : "S") : null;

  const setAll = (f: (n: (typeof nodes)[number]) => (typeof nodes)[number]) => edit((d) => mapNodes(d, ids, f));
  const iconName = (n: string) => ICON_FILE[n] ?? n;

  return (
    <>
      {one && (
        <Field label="Label">
          <Textarea key={one.id + one.label} aria-label="Label" defaultValue={one.label} rows={2} className="min-h-0 resize-none"
            onBlur={(e) => e.currentTarget.value !== one.label && edit((d) => mapNode(d, one.id, (n) => ({ ...n, label: e.currentTarget.value })))}
            onKeyDown={(e) => { if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); e.currentTarget.blur(); } }}
          />
        </Field>
      )}
      <Field label="Colour"><RoleSwatches value={role} onPick={(r) => r && edit((d) => setRoleOnSelection(d, selection, r))} /></Field>
      {one && base && (
        <Field label="Size">
          <Segmented label="Size" value={sizeKey} onChange={(k) => {
            const f = SIZES.find((s) => s.k === k)!.f;
            edit((d) => mapNode(d, one.id, (n) => ({ ...n, size: f === 1 ? null : { w: Math.round(base.w * f), h: Math.round(base.h * f) } })));
          }} options={SIZES.map((s) => ({ value: s.k, label: s.k }))} />
        </Field>
      )}
      <Field label="Shape">
        <Select value={nodes.every((n) => n.shape === nodes[0].shape) ? nodes[0].shape : ""} onValueChange={(v) => setAll((n) => ({ ...n, shape: v as Shape }))}>
          <SelectTrigger size="sm" aria-label="Shape" className="w-full"><SelectValue placeholder="Mixed" /></SelectTrigger>
          <SelectContent>{SHAPES.map((s) => <SelectItem key={s} value={s}>{cap(s)}</SelectItem>)}</SelectContent>
        </Select>
      </Field>
      <Field label="Icon">
        <Select value={nodes.every((n) => n.icon === nodes[0].icon) ? (canonicalIcon(nodes[0].icon) ?? "none") : ""} onValueChange={(v) => setAll((n) => ({ ...n, icon: v === "none" ? null : v }))}>
          <SelectTrigger size="sm" aria-label="Icon" className="w-full"><SelectValue placeholder="Mixed" /></SelectTrigger>
          <SelectContent>
            <SelectItem value="none">None</SelectItem>
            {names.map((n) => <SelectItem key={n} value={n}><span className="flex items-center gap-2"><Icon name={iconName(n)} size={14} />{cap(n)}</span></SelectItem>)}
          </SelectContent>
        </Select>
      </Field>
      {nodes.length > 1 && (
        <Field label="Align">
          <div className="flex items-center gap-1" role="group" aria-label="Align">
            {(["left", "center", "right"] as const).map((m) => (
              <IconBtn key={m} label={`Align ${m === "center" ? "centres" : m + " edges"}`} side="bottom" onClick={() => A.alignSelection(m)}><Icon name={`align-${m}`} /></IconBtn>
            ))}
            <span aria-hidden className="mx-1 h-4 w-px bg-hair" />
            {(["top", "middle", "bottom"] as const).map((m) => (
              <IconBtn key={m} label={`Align ${m === "middle" ? "middles" : m + " edges"}`} side="bottom" onClick={() => A.alignSelection(m)}><Icon name={`align-${m}`} /></IconBtn>
            ))}
          </div>
        </Field>
      )}
      {one && (
        <Field label="Group">
          <Select value={one.group ?? "none"} onValueChange={(v) => {
            if (v === "new") A.groupSelection();
            else edit((d) => mapNode(d, one.id, (n) => ({ ...n, group: v === "none" ? null : v })));
          }}>
            <SelectTrigger size="sm" aria-label="Group" className="w-full"><SelectValue /></SelectTrigger>
            <SelectContent>
              <SelectItem value="none">No group</SelectItem>
              {diagram.groups.map((g) => <SelectItem key={g.id} value={g.id}>{g.label}</SelectItem>)}
              <SelectItem value="new">New group…</SelectItem>
            </SelectContent>
          </Select>
        </Field>
      )}
      {one && (
        <Field label="Tasks">
          <div className="flex items-center justify-between text-sm">
            <span>{diagram.tasks.filter((t) => t.node === one.id && t.done).length} of {diagram.tasks.filter((t) => t.node === one.id).length} done</span>
            <Button variant="secondary" size="sm" onClick={() => {
              const id = A.addTask("New task", one.id);
              if (id) { A.togglePanel("tasks"); setState({ taskSel: id }); }
            }}><Icon name="plus" />Add</Button>
          </div>
        </Field>
      )}
      <div className="flex gap-2">
        <Button variant="secondary" size="sm" className="flex-1" onClick={A.duplicateSelection}><Icon name="duplicate" />Duplicate</Button>
        <Button variant="secondary" size="sm" className="flex-1 text-bad hover:bg-bad-soft" onClick={A.deleteSelection}><Icon name="trash" />Delete</Button>
      </div>
    </>
  );
}

function EdgeInspector() {
  const diagram = useStore((s) => s.diagram);
  const ids = useStore((s) => s.selection.edges);
  const edges = ids.map((id) => diagram.edges.find((e) => e.id === id)).filter((e) => !!e);
  const one = edges.length === 1 ? edges[0] : null;
  const patch = (p: Partial<(typeof edges)[number]>) => edit((d) => ({ ...d, edges: d.edges.map((e) => (ids.includes(e.id) ? { ...e, ...p } : e)) }));
  return (
    <>
      {one && (
        <Field label="Label">
          <Input key={one.id + one.label} aria-label="Edge label" defaultValue={one.label} placeholder="No label"
            onBlur={(e) => e.currentTarget.value !== one.label && edit((d) => mapEdge(d, one.id, (x) => ({ ...x, label: e.currentTarget.value })))}
            onKeyDown={(e) => e.key === "Enter" && e.currentTarget.blur()} />
        </Field>
      )}
      <Field label="Colour"><RoleSwatches value={edges.every((e) => e.role === edges[0].role) ? edges[0].role : null} neutralDefault onPick={(r) => patch({ role: r })} /></Field>
      <Field label="Line">
        <Segmented<LineStyle> label="Line style" value={edges.every((e) => e.line === edges[0].line) ? edges[0].line : null} onChange={(v) => patch({ line: v })} options={[
          { value: "solid", label: "Solid" }, { value: "dashed", label: "Dashed" }, { value: "dotted", label: "Dotted" },
        ]} />
      </Field>
      <Field label="Arrow">
        <Segmented<Arrow> label="Arrowheads" value={edges.every((e) => e.arrow === edges[0].arrow) ? edges[0].arrow : null} onChange={(v) => patch({ arrow: v })} options={[
          { value: "end", label: "End" }, { value: "both", label: "Both" }, { value: "none", label: "None" },
        ]} />
      </Field>
      <div className="flex gap-2">
        {one && <Button variant="secondary" size="sm" className="flex-1" onClick={() => edit((d) => mapEdge(d, one.id, (e) => ({ ...e, from: e.to, to: e.from })))}>Reverse</Button>}
        <Button variant="secondary" size="sm" className="flex-1 text-bad hover:bg-bad-soft" onClick={A.deleteSelection}><Icon name="trash" />Delete</Button>
      </div>
    </>
  );
}

function GroupInspector() {
  const diagram = useStore((s) => s.diagram);
  const id = useStore((s) => s.selection.groups[0]);
  const g = diagram.groups.find((x) => x.id === id);
  if (!g) return null;
  const members = diagram.nodes.filter((n) => n.group === id).length;
  return (
    <>
      <Field label="Label">
        <Input key={g.id + g.label} aria-label="Group label" defaultValue={g.label}
          onBlur={(e) => e.currentTarget.value !== g.label && edit((d) => mapGroup(d, g.id, (x) => ({ ...x, label: e.currentTarget.value })))}
          onKeyDown={(e) => e.key === "Enter" && e.currentTarget.blur()} />
      </Field>
      <Field label="Colour"><RoleSwatches value={g.role} onPick={(r) => r && edit((d: Diagram) => mapGroup(d, g.id, (x) => ({ ...x, role: r })))} /></Field>
      <p className="text-xs text-ink-muted">{members} step{members === 1 ? "" : "s"} in this group.</p>
      <div className="flex gap-2">
        <Button variant="secondary" size="sm" className="flex-1" onClick={() => A.ungroup(g.id)}>Ungroup</Button>
        <Button variant="secondary" size="sm" className="flex-1 text-bad hover:bg-bad-soft" onClick={A.deleteSelection}><Icon name="trash" />Delete</Button>
      </div>
    </>
  );
}
