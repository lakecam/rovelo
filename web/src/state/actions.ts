// Everything a button, menu or shortcut can do. UI components call these; none hold logic.
import { core } from "@/wasm";
import * as ops from "@/lib/ops";
import type { Align } from "@/lib/ops";
import { centerOn, zoomAt } from "@/lib/geom";
import { nudgeInto, planQuickCreate, type Side } from "@/lib/quickCreate";
import { swapExportIcons } from "@/lib/nodeIcons";
import { BRAND_EXT, DIAGRAM_EXT, docName, openText, saveData, slug, svgToPng } from "@/lib/files";
import { todayISO } from "@/lib/tasks";
import type { BrandKit, Diagram, Id, Selection } from "@/lib/types";
import { emptySelection } from "@/lib/types";
import {
  chromeInset, edit, editWith, fit, getState, isDirty, loadDiagram, markSaved, redo, sceneOf, setState, toast, undo,
  type FileRef, type Tool,
} from "./store";

export { undo, redo, fit };

/** The core's SVG with Nucleo node icons. */
export const exportedSvg = (d: Diagram) => swapExportIcons(core.toSvg(d), d);

const guard = async (f: () => Promise<void>) => {
  try { await f(); } catch (e) { toast(e instanceof Error ? e.message : String(e)); }
};

// ---- selection ----------------------------------------------------------------------------
export const select = (selection: Selection) => setState({ selection, taskSel: null });
export const clearSelection = () => setState({ selection: emptySelection(), editing: null });
export const selectAll = () => {
  const d = getState().diagram;
  select({ nodes: d.nodes.map((n) => n.id), edges: [], groups: [] });
};
export const setTool = (tool: Tool) => setState({ tool });

// ---- nodes --------------------------------------------------------------------------------
/** After making a step: select it, back to Select, open its editor, and pan the least it takes to show it. */
function afterCreate(id: Id) {
  setState({ selection: { nodes: [id], edges: [], groups: [] }, started: true, editing: { kind: "node", id, isNew: true }, tool: "select", taskSel: null });
  uncover(id);
}

/** Pan the least it takes to bring a step out from under the panels or the window edge, zoom kept. */
export function uncover(id: Id) {
  const s = getState();
  const r = sceneOf(s.diagram).nodes.find((n) => n.id === id)?.rect;
  if (!r) return;
  const inset = chromeInset(s);
  const free = { left: inset.left, top: inset.top, width: s.viewport.w - inset.left - inset.right, height: s.viewport.h - inset.top - inset.bottom };
  const v = nudgeInto(r, s.view, free, 0);
  if (v) setState({ view: v });
}

/** A default-size step centred on (x, y), blank, editor open (double-click, Add step click). */
export function addNodeAt(x: number, y: number, label = "") {
  const id = editWith((d) => {
    const probe = ops.addNodeAt(d, { x, y }, label);
    // Measure with the core so the node is centred on the click.
    const measured = core.nodeSize(probe.diagram, probe.id);
    const r = ops.addNodeAt(d, { x, y }, label, measured);
    return { diagram: r.diagram, result: r.id };
  });
  afterCreate(id);
  return id;
}

/** Add step tool drag: the step fills the box drawn. */
export function addNodeBox(box: { x: number; y: number; w: number; h: number }) {
  const id = editWith((d) => {
    const r = ops.addNodeAt(d, { x: box.x + box.w / 2, y: box.y + box.h / 2 }, "", { w: box.w, h: box.h });
    return { diagram: ops.mapNode(r.diagram, r.id, (n) => ({ ...n, size: { w: box.w, h: box.h } })), result: r.id };
  });
  afterCreate(id);
  return id;
}

/** Click (or drag to empty canvas from) a plus handle: the next step and its arrow, one undo step. */
export function quickCreate(sourceId: Id, side: Side, dropAt?: { x: number; y: number }) {
  const s = getState();
  const rect = sceneOf(s.diagram).nodes.find((n) => n.id === sourceId)?.rect;
  if (!rect) return null;
  const id = editWith((d) => {
    const q = planQuickCreate(d, sourceId, rect, side, (n) => core.nodeSize({ ...d, nodes: [...d.nodes, n] }, n.id), dropAt);
    return { diagram: q ? q.diagram : d, result: q?.nodeId ?? null };
  });
  if (id) afterCreate(id);
  return id;
}

export function addNodeInView() {
  const s = getState();
  const c = { x: (s.viewport.w / 2 - s.view.x) / s.view.k, y: (s.viewport.h / 2 - s.view.y) / s.view.k };
  return addNodeAt(c.x, c.y);
}

export function connectNodes(from: Id, to: Id) {
  const id = editWith((d) => {
    const r = ops.connect(d, from, to);
    return { diagram: r ? r.diagram : d, result: r?.id ?? null };
  });
  if (id) setState({ selection: { nodes: [], edges: [id], groups: [] } });
  return id;
}

export function deleteSelection() {
  const s = getState();
  if (!s.selection.nodes.length && !s.selection.edges.length && !s.selection.groups.length) return;
  edit((d) => ops.deleteSelection(d, s.selection));
  setState({ selection: emptySelection(), editing: null });
}

export function duplicateSelection() {
  const s = getState();
  if (!s.selection.nodes.length) return;
  const ids = editWith((d) => {
    const r = ops.duplicate(d, s.selection.nodes);
    return { diagram: r.diagram, result: r.ids };
  });
  setState({ selection: { nodes: ids, edges: [], groups: [] } });
}

export function groupSelection() {
  const s = getState();
  if (s.selection.nodes.length < 1) return;
  const id = editWith((d) => {
    const r = ops.groupNodes(d, s.selection.nodes);
    return { diagram: r ? r.diagram : d, result: r?.id ?? null };
  });
  if (id) setState({ selection: { nodes: [], edges: [], groups: [id] }, editing: { kind: "group", id } });
}

export function ungroup(id: Id) {
  edit((d) => ops.removeGroup(d, id));
}

export function nudgeSelection(dx: number, dy: number) {
  const s = getState();
  if (s.selection.nodes.length) edit((d) => ops.nudge(d, s.selection.nodes, dx, dy));
}

export function alignSelection(mode: Align) {
  const s = getState();
  const rects = new Map(sceneOf(s.diagram).nodes.map((n) => [n.id, n.rect]));
  edit((d) => ops.alignNodes(d, s.selection.nodes, mode, rects));
}

export function autoLayout() {
  edit((d) => (d.nodes.length ? core.autoLayout(d) : d));
  fit();
}

// ---- tasks --------------------------------------------------------------------------------
export function addTask(title: string, node: Id | null) {
  const id = editWith((d) => {
    const r = ops.addTask(d, title, node);
    return { diagram: r ? r.diagram : d, result: r?.id ?? null };
  });
  return id;
}

// ---- view ---------------------------------------------------------------------------------
export function zoomBy(factor: number) {
  const s = getState();
  setState({ view: zoomAt(s.view, factor, s.viewport.w / 2, s.viewport.h / 2) });
}
export function zoomTo(k: number) {
  const s = getState();
  setState({ view: zoomAt(s.view, k / s.view.k, s.viewport.w / 2, s.viewport.h / 2) });
}

/** Select a node and bring it to the middle of the free canvas. */
export function focusNode(id: Id) {
  const s = getState();
  const r = sceneOf(s.diagram).nodes.find((n) => n.id === id)?.rect;
  if (!r) return;
  setState({
    selection: { nodes: [id], edges: [], groups: [] },
    view: centerOn(s.view, { x: r.x + r.w / 2, y: r.y + r.h / 2 }, s.viewport.w, s.viewport.h, chromeInset({ ...s, selection: { nodes: [id], edges: [], groups: [] } })),
  });
}

export function togglePanel(tab?: "tasks" | "brand" | "code") {
  const s = getState();
  if (tab && (!s.panelOpen || s.tab !== tab)) setState({ panelOpen: true, tab });
  else if (tab) setState({ tab });
  else setState({ panelOpen: !s.panelOpen });
}

// ---- documents ----------------------------------------------------------------------------
export function newFromTemplate(name: "userflow" | "taskflow" | "blank") {
  const s = getState();
  const d = core.template(name, todayISO(), s.diagram.brand);
  loadDiagram(d, null, true);
  requestAnimationFrame(() => fit());
}

export function newDiagram() {
  if (isDirty(getState()) && !confirm("Discard unsaved changes?")) return;
  loadDiagram(core.freshDiagram(), null, false);
  requestAnimationFrame(() => fit());
}

export const openDiagram = () =>
  guard(async () => {
    if (isDirty(getState()) && !confirm("Discard unsaved changes?")) return;
    const f = await openText("diagram");
    if (!f) return;
    let d: Diagram;
    try {
      d = core.repair(JSON.parse(f.text));
    } catch (e) {
      throw new Error(`That is not a Rovelo diagram (${e instanceof Error ? e.message : e})`);
    }
    loadDiagram(d, f.ref, true);
    requestAnimationFrame(() => fit());
    toast(`Opened ${docName(f.ref.name)}`);
  });

export const saveDiagram = (asNew = false) =>
  guard(async () => {
    const s = getState();
    const d = core.layoutUnplaced(s.diagram);
    const ref = await saveData(JSON.stringify(d, null, 2), {
      kind: "diagram",
      suggestedName: `${slug(d.title)}${DIAGRAM_EXT}`,
      ref: asNew ? null : s.file,
    });
    if (!ref) return;
    markSaved(ref as FileRef);
    toast(`Saved ${docName(ref.name)}`);
  });

export const exportSvg = () =>
  guard(async () => {
    const d = getState().diagram;
    const ref = await saveData(exportedSvg(d), { kind: "svg", suggestedName: `${slug(d.title)}.svg` });
    if (ref) toast("Exported SVG");
  });

export const exportPng = () =>
  guard(async () => {
    const d = getState().diagram;
    const bytes = await svgToPng(exportedSvg(d), 2);
    const ref = await saveData(bytes, { kind: "png", suggestedName: `${slug(d.title)}.png` });
    if (ref) toast("Exported PNG");
  });

// ---- brand --------------------------------------------------------------------------------
export function setBrand(b: BrandKit) {
  edit((d) => ({ ...d, brand: b }));
}

export const importBrand = () =>
  guard(async () => {
    const f = await openText("brand");
    if (!f) return;
    let kit: BrandKit;
    try {
      kit = core.validateBrand(f.text);
    } catch (e) {
      throw new Error(`That is not a brand kit (${e instanceof Error ? e.message : e})`);
    }
    setBrand(kit);
    toast(`Loaded ${kit.name}`);
  });

export const exportBrand = () =>
  guard(async () => {
    const b = getState().diagram.brand;
    const ref = await saveData(JSON.stringify(b, null, 2), { kind: "brand", suggestedName: `${slug(b.name)}${BRAND_EXT}` });
    if (ref) toast("Saved brand kit");
  });

// ---- mermaid ------------------------------------------------------------------------------
/** Replace the diagram with an imported Mermaid flowchart (kept brand and title). */
export function importMermaid(src: string) {
  const r = core.fromMermaid(src);
  if (!r.diagram.nodes.length) return r.diagnostics;
  edit((old) => core.autoLayout({ ...r.diagram, brand: old.brand, tasks: old.tasks.filter((t) => !t.node), title: old.title === "Untitled" ? r.diagram.title : old.title }));
  setState({ started: true, selection: emptySelection() });
  requestAnimationFrame(() => fit());
  return r.diagnostics;
}
