// Command palette (React Bits Pro "command-menu-1", wired to the app's own actions).
import { useCallback, useMemo } from "react";
import CommandMenu, { type Command } from "@/components/blocks/command-menu-1";
import * as A from "@/state/actions";
import { setState, useStore } from "@/state/store";
import { toggleTheme } from "./theme";

export function CommandPalette() {
  const open = useStore((s) => s.paletteOpen);
  const nodes = useStore((s) => s.diagram.nodes);
  const hasNodes = nodes.length > 0;
  const onClose = useCallback(() => setState({ paletteOpen: false }), []);

  const commands = useMemo<Command[]>(() => {
    const c = (group: string, id: string, title: string, shortcut: string, icon: string, run: () => void, detail?: string): Command => ({ id, group, title, shortcut, icon, run, detail });
    return [
      c("Tools", "tool-select", "Select", "V", "select", () => A.setTool("select")),
      c("Tools", "tool-hand", "Hand", "H", "pan", () => A.setTool("hand")),
      c("Tools", "tool-node", "Add step", "N", "add-node", () => A.setTool("node"), "Then click the canvas"),
      c("Tools", "tool-connect", "Connect", "C", "connect", () => A.setTool("connect"), "Drag from one step to another"),
      c("Diagram", "add-here", "Add a step in the middle of the view", "", "plus", () => A.addNodeInView()),
      ...(hasNodes ? [c("Diagram", "layout", "Auto-layout", "⇧⌘L", "auto-layout", A.autoLayout)] : []),
      c("Diagram", "fit", "Fit to screen", "⌘0", "fit", () => A.fit()),
      c("Diagram", "undo", "Undo", "⌘Z", "undo", A.undo),
      c("Diagram", "redo", "Redo", "⇧⌘Z", "redo", A.redo),
      c("Diagram", "dup", "Duplicate selection", "⌘D", "duplicate", A.duplicateSelection),
      c("Diagram", "group", "Group selection", "⌘G", "shapes", A.groupSelection),
      c("Diagram", "del", "Delete selection", "⌫", "trash", A.deleteSelection),
      c("Panels", "p-tasks", "Show tasks", "", "tasks", () => A.togglePanel("tasks")),
      c("Panels", "p-brand", "Show brand kit", "", "brand", () => A.togglePanel("brand")),
      c("Panels", "p-code", "Show code", "", "code", () => A.togglePanel("code")),
      c("Panels", "p-mermaid", "Paste Mermaid…", "", "code", () => setState({ mermaidOpen: true })),
      c("File", "new", "New diagram", "⌘N", "add-node", A.newDiagram),
      c("File", "open", "Open…", "⌘O", "open", () => void A.openDiagram()),
      c("File", "save", "Save", "⌘S", "save", () => void A.saveDiagram()),
      c("File", "svg", "Export SVG", "", "export", () => void A.exportSvg()),
      c("File", "png", "Export PNG", "", "export", () => void A.exportPng()),
      c("File", "theme", "Switch light or dark", "", "sun", toggleTheme),
      ...nodes.slice(0, 40).map((n) => c("Go to step", `go-${n.id}`, n.label.replace(/\n/g, " "), "", "select", () => A.focusNode(n.id))),
    ];
  }, [nodes, hasNodes]);

  return <CommandMenu commands={commands} open={open} onClose={onClose} />;
}
