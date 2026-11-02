import { useEffect } from "react";
import { TooltipProvider } from "@/components/ui/tooltip";
import * as A from "@/state/actions";
import { getState, setState, useStore } from "@/state/store";
import { applyTheme } from "./ui/theme";
import { CommandPalette } from "./ui/CommandPalette";
import { Canvas } from "./ui/Canvas";
import { ActionsPill, TitlePill, Toast, ToolPill, ZoomPill } from "./ui/Chrome";
import { Inspector } from "./ui/Inspector";
import { EmptyState, LeftPanel, MermaidDialog } from "./ui/Panels";

const typing = (t: EventTarget | null) =>
  t instanceof HTMLElement && (t.isContentEditable || /^(INPUT|TEXTAREA|SELECT)$/.test(t.tagName));

function useShortcuts() {
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const mod = e.metaKey || e.ctrlKey;
      const k = e.key.toLowerCase();
      if (mod && k === "k") { e.preventDefault(); return setState({ paletteOpen: !getState().paletteOpen }); }
      if (mod && k === "z") { e.preventDefault(); return e.shiftKey ? A.redo() : A.undo(); }
      if (mod && k === "y") { e.preventDefault(); return A.redo(); }
      if (mod && k === "s") { e.preventDefault(); return void A.saveDiagram(e.shiftKey); }
      if (mod && k === "o") { e.preventDefault(); return void A.openDiagram(); }
      if (mod && k === "n") { e.preventDefault(); return A.newDiagram(); }
      if (mod && k === "\\") { e.preventDefault(); return A.togglePanel(); }
      if (mod && k === "0") { e.preventDefault(); return A.fit(); }
      if (mod && (k === "=" || k === "+")) { e.preventDefault(); return A.zoomBy(1.25); }
      if (mod && k === "-") { e.preventDefault(); return A.zoomBy(1 / 1.25); }
      if (mod && e.shiftKey && k === "l") { e.preventDefault(); return A.autoLayout(); }
      if (typing(e.target)) return;
      if (mod && k === "a") { e.preventDefault(); return A.selectAll(); }
      if (mod && k === "d") { e.preventDefault(); return A.duplicateSelection(); }
      if (mod && k === "g") { e.preventDefault(); return A.groupSelection(); }
      if (mod) return;
      const s = getState();
      switch (e.key) {
        case "Delete": case "Backspace": e.preventDefault(); return A.deleteSelection();
        case "Escape":
          if (s.selection.nodes.length || s.selection.edges.length || s.selection.groups.length) return A.clearSelection();
          if (s.tool !== "select") return A.setTool("select");
          return;
        case "Enter":
          if (s.selection.nodes.length === 1) { e.preventDefault(); setState({ editing: { kind: "node", id: s.selection.nodes[0] } }); }
          else if (s.selection.edges.length === 1) { e.preventDefault(); setState({ editing: { kind: "edge", id: s.selection.edges[0] } }); }
          return;
        case "ArrowLeft": case "ArrowRight": case "ArrowUp": case "ArrowDown": {
          if (!s.selection.nodes.length) return;
          e.preventDefault();
          const step = e.shiftKey ? 1 : 8;
          return A.nudgeSelection(e.key === "ArrowLeft" ? -step : e.key === "ArrowRight" ? step : 0, e.key === "ArrowUp" ? -step : e.key === "ArrowDown" ? step : 0);
        }
      }
      const tools: Record<string, "select" | "hand" | "node" | "connect"> = { v: "select", h: "hand", n: "node", c: "connect" };
      if (tools[k]) A.setTool(tools[k]);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);
}

export default function App() {
  useShortcuts();
  useEffect(() => applyTheme(), []);
  const panelOpen = useStore((s) => s.panelOpen);
  return (
    <TooltipProvider delayDuration={400} skipDelayDuration={150}>
      <main className="relative h-full w-full overflow-hidden bg-paper">
        <Canvas />
        <EmptyState />
        <div className="pointer-events-none absolute inset-x-4 top-4 flex items-start justify-between gap-4">
          <TitlePill />
          <div className="absolute left-1/2 -translate-x-1/2"><ToolPill /></div>
          <ActionsPill />
        </div>
        <LeftPanel />
        <Inspector />
        <div className="pointer-events-none absolute bottom-4 left-4"><ZoomPill /></div>
        {!panelOpen && null}
        <Toast />
        <MermaidDialog />
        <CommandPalette />
      </main>
    </TooltipProvider>
  );
}
