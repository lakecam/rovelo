// Floating chrome around the canvas: title, tools, file actions, zoom + history.
import { useEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuShortcut, DropdownMenuTrigger } from "@/components/ui/dropdown-menu";
import { canRedo, canUndo } from "@/lib/history";
import { docName } from "@/lib/files";
import * as A from "@/state/actions";
import { edit, isDirty, setState, useStore, type Tool } from "@/state/store";
import { Divider, IconBtn, Tip } from "./kit";
import { Icon } from "./icons";
import { toggleTheme, useTheme } from "./theme";
import { cn } from "@/lib/utils";

const MARK = (
  <svg viewBox="0 0 32 32" width="22" height="22" aria-hidden>
    <rect width="32" height="32" rx="8" fill="var(--rv-accent)" />
    <circle cx="9" cy="10" r="3" fill="#fff" />
    <circle cx="23" cy="22" r="3" fill="#fff" />
    <path d="M9 13v4a4 4 0 0 0 4 4h7" fill="none" stroke="#fff" strokeWidth="2" strokeLinecap="round" />
  </svg>
);

export function TitlePill() {
  const title = useStore((s) => s.diagram.title);
  const dirty = useStore(isDirty);
  const file = useStore((s) => s.file);
  const panelOpen = useStore((s) => s.panelOpen);
  const [renaming, setRenaming] = useState(false);
  const input = useRef<HTMLInputElement>(null);
  useEffect(() => { if (renaming) input.current?.select(); }, [renaming]);
  const commit = (v: string) => {
    setRenaming(false);
    const t = v.trim() || "Untitled";
    if (t !== title) edit((d) => ({ ...d, title: t }));
  };
  return (
    <div className="rv-pill pointer-events-auto flex h-10 items-center gap-1 rounded-xl p-1">
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <button className="grid size-8 place-items-center rounded-lg transition-[background-color] duration-150 hover:bg-accent" aria-label="Rovelo menu">{MARK}</button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="start" className="w-56">
          <DropdownMenuItem onSelect={A.newDiagram}>New diagram<DropdownMenuShortcut>⌘N</DropdownMenuShortcut></DropdownMenuItem>
          <DropdownMenuItem onSelect={A.openDiagram}>Open…<DropdownMenuShortcut>⌘O</DropdownMenuShortcut></DropdownMenuItem>
          <DropdownMenuItem onSelect={() => A.saveDiagram(true)}>Save as…<DropdownMenuShortcut>⇧⌘S</DropdownMenuShortcut></DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem onSelect={() => setState({ mermaidOpen: true })}>Paste Mermaid…</DropdownMenuItem>
          <DropdownMenuItem onSelect={A.importBrand}>Load brand kit…</DropdownMenuItem>
          <DropdownMenuItem onSelect={A.exportBrand}>Save brand kit…</DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
      {renaming ? (
        <input
          ref={input}
          defaultValue={title}
          aria-label="Diagram title"
          className="h-8 w-48 rounded-lg border border-brand bg-sunk px-2 text-sm font-medium outline-none ring-[3px] ring-brand-soft"
          onBlur={(e) => commit(e.currentTarget.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") commit(e.currentTarget.value);
            if (e.key === "Escape") setRenaming(false);
          }}
        />
      ) : (
        <Tip label={file ? docName(file.name) : "Rename"} side="bottom">
          <button className="h-8 max-w-48 truncate rounded-lg px-2 text-sm font-medium transition-[background-color] duration-150 hover:bg-accent" onClick={() => setRenaming(true)}>
            {title}
          </button>
        </Tip>
      )}
      {dirty && <span className="mx-1 size-2 shrink-0 rounded-full bg-warn" role="img" aria-label="Unsaved changes" />}
      <Divider />
      <IconBtn label={panelOpen ? "Hide panel" : "Show panel"} keys="⌘\" active={panelOpen} onClick={() => A.togglePanel()}>
        <Icon name="sidebar" />
      </IconBtn>
    </div>
  );
}

const TOOLS: { tool: Tool; icon: string; label: string; key: string }[] = [
  { tool: "hand", icon: "pan", label: "Hand", key: "H" },
  { tool: "select", icon: "select", label: "Select", key: "V" },
  { tool: "node", icon: "add-node", label: "Add step", key: "N" },
  { tool: "connect", icon: "connect", label: "Connect", key: "C" },
];

export function ToolPill() {
  const tool = useStore((s) => s.tool);
  const hasNodes = useStore((s) => s.diagram.nodes.length > 0);
  return (
    <div className="rv-pill pointer-events-auto flex h-10 items-center gap-1 rounded-xl p-1" role="toolbar" aria-label="Tools">
      {TOOLS.slice(0, 2).map((t) => (
        <IconBtn key={t.tool} label={t.label} keys={t.key} active={tool === t.tool} onClick={() => A.setTool(t.tool)}><Icon name={t.icon} /></IconBtn>
      ))}
      <Divider />
      {TOOLS.slice(2).map((t) => (
        <IconBtn key={t.tool} label={t.label} keys={t.key} active={tool === t.tool} onClick={() => A.setTool(t.tool)}><Icon name={t.icon} /></IconBtn>
      ))}
      <Divider />
      <IconBtn label="Auto-layout" keys="⇧⌘L" disabled={!hasNodes} onClick={A.autoLayout}><Icon name="auto-layout" /></IconBtn>
    </div>
  );
}

export function ActionsPill() {
  const theme = useTheme();
  return (
    <div className="rv-pill pointer-events-auto flex h-10 items-center gap-1 rounded-xl p-1">
      <IconBtn label="Open" keys="⌘O" onClick={A.openDiagram}><Icon name="open" /></IconBtn>
      <IconBtn label="Save" keys="⌘S" onClick={() => A.saveDiagram()}><Icon name="save" /></IconBtn>
      <IconBtn label={theme === "dark" ? "Switch to light" : "Switch to dark"} onClick={toggleTheme}><Icon name={theme === "dark" ? "sun" : "moon"} /></IconBtn>
      <Divider />
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button size="sm"><Icon name="export" />Export</Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="w-44">
          <DropdownMenuItem onSelect={A.exportSvg}>SVG image</DropdownMenuItem>
          <DropdownMenuItem onSelect={A.exportPng}>PNG image (2x)</DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
}

export function ZoomPill() {
  const k = useStore((s) => s.view.k);
  const history = useStore((s) => s.history);
  return (
    <div className="rv-pill pointer-events-auto flex h-10 items-center gap-1 rounded-xl p-1">
      <IconBtn label="Undo" keys="⌘Z" side="top" disabled={!canUndo(history)} onClick={A.undo}><Icon name="undo" /></IconBtn>
      <IconBtn label="Redo" keys="⇧⌘Z" side="top" disabled={!canRedo(history)} onClick={A.redo}><Icon name="redo" /></IconBtn>
      <Divider />
      <IconBtn label="Zoom out" keys="⌘−" side="top" onClick={() => A.zoomBy(1 / 1.25)}><Icon name="minus" /></IconBtn>
      <Tip label="Reset to 100%" side="top">
        <button onClick={() => A.zoomTo(1)} className={cn("h-8 min-w-12 rounded-lg px-2 text-sm font-medium tabular-nums transition-[background-color] duration-150 hover:bg-accent")}>
          {Math.round(k * 100)}%
        </button>
      </Tip>
      <IconBtn label="Zoom in" keys="⌘+" side="top" onClick={() => A.zoomBy(1.25)}><Icon name="plus" /></IconBtn>
      <Divider />
      <IconBtn label="Fit to screen" keys="⌘0" side="top" onClick={() => A.fit()}><Icon name="fit" /></IconBtn>
    </div>
  );
}

export function Toast() {
  const msg = useStore((s) => s.toast);
  if (!msg) return null;
  return (
    <div role="status" className="rv-panel pointer-events-none absolute bottom-[72px] left-1/2 -translate-x-1/2 rounded-xl px-4 py-2 text-sm">
      {msg}
    </div>
  );
}
