// App state in one small external store (no state library). Edits are immutable Diagram updates;
// the core is called only for layout, geometry, text and SVG.
import { useSyncExternalStore } from "react";
import { core } from "@/wasm";
import * as H from "@/lib/history";
import { fitView, type View, type Inset } from "@/lib/geom";
import { pruneSelection } from "@/lib/ops";
import { emptySelection, type Diagram, type Id, type Scene, type Selection } from "@/lib/types";

export type Tool = "hand" | "select" | "node" | "connect";
export type PanelTab = "tasks" | "brand" | "code";
/** `isNew`: the label typed into a step just made joins its creation step in the history. */
export type Editing = { kind: "node" | "edge" | "group"; id: Id; isNew?: boolean } | null;
export interface FileRef { name: string; handle?: unknown; path?: string }

export interface State {
  diagram: Diagram;
  history: H.History<Diagram>;
  saved: Diagram;
  selection: Selection;
  tool: Tool;
  view: View;
  viewport: { w: number; h: number };
  panelOpen: boolean;
  tab: PanelTab;
  editing: Editing;
  started: boolean;
  taskSel: Id | null;
  file: FileRef | null;
  mermaidOpen: boolean;
  paletteOpen: boolean;
  toast: string | null;
}

type Listener = () => void;
let state: State;
const listeners = new Set<Listener>();
let gestureBase: Diagram | null = null;

export const getState = () => state;

export function setState(patch: Partial<State> | ((s: State) => Partial<State>)) {
  const p = typeof patch === "function" ? patch(state) : patch;
  state = { ...state, ...p };
  listeners.forEach((l) => l());
}

export function subscribe(l: Listener) {
  listeners.add(l);
  return () => listeners.delete(l);
}

export function useStore<T>(sel: (s: State) => T): T {
  return useSyncExternalStore(subscribe, () => sel(state));
}

// ---- scene cache: one Rust call per diagram object, shared by every consumer ----------------
const sceneCache = new WeakMap<Diagram, Scene>();
export function sceneOf(d: Diagram): Scene {
  let s = sceneCache.get(d);
  if (!s) sceneCache.set(d, (s = core.scene(d)));
  return s;
}
export const useScene = () => sceneOf(useStore((s) => s.diagram));

// ---- init ---------------------------------------------------------------------------------
const STORE_KEY = "rovelo:doc";

export function initStore(initial?: Diagram) {
  let diagram = initial;
  if (!diagram) {
    try {
      const raw = localStorage.getItem(STORE_KEY);
      if (raw) diagram = core.layoutUnplaced(core.repair(JSON.parse(raw)));
    } catch { /* ignore a corrupt or unavailable store */ }
  }
  diagram ??= core.freshDiagram();
  state = {
    diagram,
    history: H.emptyHistory(),
    saved: diagram,
    selection: emptySelection(),
    tool: "select",
    view: { x: 0, y: 0, k: 1 },
    viewport: { w: 1200, h: 800 },
    panelOpen: true,
    tab: "tasks",
    editing: null,
    started: diagram.nodes.length > 0,
    taskSel: null,
    file: null,
    mermaidOpen: false,
    paletteOpen: false,
    toast: null,
  };
  let timer: ReturnType<typeof setTimeout> | undefined;
  subscribe(() => {
    clearTimeout(timer);
    timer = setTimeout(() => {
      try { localStorage.setItem(STORE_KEY, JSON.stringify(state.diagram)); } catch { /* quota or private mode */ }
    }, 400);
  });
}

// ---- edits --------------------------------------------------------------------------------

/** One undoable step. Returning the same object is a no-op. */
export function edit(f: (d: Diagram) => Diagram) {
  const before = state.diagram;
  const next = f(before);
  if (next === before) return;
  setState((s) => ({
    diagram: next,
    history: H.commit(s.history, before),
    selection: pruneSelection(next, s.selection),
  }));
}

/** Edit and also hand back what the edit produced (e.g. a new id). */
export function editWith<T>(f: (d: Diagram) => { diagram: Diagram; result: T }): T {
  let out!: T;
  edit((d) => {
    const r = f(d);
    out = r.result;
    return r.diagram;
  });
  return out;
}

/** Change the document without a new undo step: the change joins the last one (a new step's first label). */
export function amend(f: (d: Diagram) => Diagram) {
  const next = f(state.diagram);
  if (next !== state.diagram) setState({ diagram: next });
}

/** Gesture: many live updates, one history step. */
export function beginGesture() {
  gestureBase = state.diagram;
}
export function live(f: (d: Diagram) => Diagram) {
  setState((s) => ({ diagram: f(s.diagram) }));
}
export function endGesture() {
  const base = gestureBase;
  gestureBase = null;
  if (base && base !== state.diagram) setState((s) => ({ history: H.commit(s.history, base) }));
}
/** Throw the gesture away (Esc while dragging). */
export function cancelGesture() {
  if (gestureBase) setState({ diagram: gestureBase });
  gestureBase = null;
}

export function undo() {
  const r = H.undo(state.history, state.diagram);
  if (r) setState((s) => ({ diagram: r.value, history: r.history, selection: pruneSelection(r.value, s.selection), editing: null }));
}
export function redo() {
  const r = H.redo(state.history, state.diagram);
  if (r) setState((s) => ({ diagram: r.value, history: r.history, selection: pruneSelection(r.value, s.selection), editing: null }));
}

/** Replace the whole document (open, new, template). Clears history. */
export function loadDiagram(d: Diagram, file: FileRef | null = null, started = true) {
  const laid = core.layoutUnplaced(d);
  setState({
    diagram: laid, saved: laid, history: H.emptyHistory(), selection: emptySelection(),
    file, started, editing: null, taskSel: null,
  });
}

export const isDirty = (s: State) => s.diagram !== s.saved;
export const markSaved = (file?: FileRef | null) =>
  setState((s) => ({ saved: s.diagram, file: file === undefined ? s.file : file }));

export function toast(msg: string) {
  setState({ toast: msg });
  setTimeout(() => state.toast === msg && setState({ toast: null }), 2600);
}

// ---- view ---------------------------------------------------------------------------------

/** Space the floating chrome leaves free around the canvas. */
export function chromeInset(s: State): Inset {
  return { left: s.panelOpen ? 12 + 300 + 24 : 24, top: 72, right: s.selection.nodes.length + s.selection.edges.length + s.selection.groups.length ? 12 + 280 + 24 : 24, bottom: 72 };
}

export function fit(animateInset?: Inset) {
  const s = state;
  const b = sceneOf(s.diagram).bounds;
  if (!s.diagram.nodes.length) return setState({ view: { x: s.viewport.w / 2, y: s.viewport.h / 2, k: 1 } });
  setState({ view: fitView(b, s.viewport.w, s.viewport.h, animateInset ?? chromeInset({ ...s, selection: emptySelection() })) });
}
