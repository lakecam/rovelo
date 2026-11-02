// Left panel (Tasks / Brand / Code), empty-state block and the Mermaid dialog.
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Textarea } from "@/components/ui/textarea";
import * as A from "@/state/actions";
import { setState, useStore, type PanelTab } from "@/state/store";
import type { Diag } from "@/lib/types";
import { BrandTab } from "./BrandTab";
import { CodeTab } from "./CodeTab";
import { TasksTab } from "./TasksTab";
import { IconBtn, Segmented } from "./kit";
import { Icon } from "./icons";
import { SlotReveal } from "./slots";
import EmptyStateBlock, { type Starter } from "@/components/blocks/empty-state-1";

export function LeftPanel() {
  const open = useStore((s) => s.panelOpen);
  const tab = useStore((s) => s.tab);
  if (!open) return null;
  return (
    <SlotReveal className="pointer-events-none absolute bottom-[72px] left-4 top-[72px] flex w-[300px] flex-col justify-start">
      <aside aria-label="Panel" className="rv-panel pointer-events-auto flex max-h-full min-h-0 flex-1 flex-col rounded-2xl pt-4">
        <div className="flex items-center gap-2 px-4 pb-4">
          <Segmented<PanelTab> label="Panel" value={tab} onChange={(t) => setState({ tab: t })} options={[
            { value: "tasks", label: <><Icon name="tasks" />Tasks</> },
            { value: "brand", label: <><Icon name="brand" />Brand</> },
            { value: "code", label: <><Icon name="code" />Code</> },
          ]} />
          <IconBtn label="Collapse panel" keys="⌘\" side="right" onClick={() => A.togglePanel()}><Icon name="close" /></IconBtn>
        </div>
        {tab === "tasks" && <TasksTab />}
        {tab === "brand" && <BrandTab />}
        {tab === "code" && <CodeTab />}
      </aside>
    </SlotReveal>
  );
}

// ---- empty state (React Bits Pro "empty-state-1", starting paths = the three templates) -----
const STARTERS: Starter[] = [
  { id: "userflow", icon: "select", title: "User flow", body: "Screens and the paths between them.", meta: "Sign-up example" },
  { id: "taskflow", icon: "tasks", title: "Task flow", body: "Steps and decisions with to-dos attached.", meta: "Release checklist" },
  { id: "blank", icon: "add-node", title: "Blank", body: "Start from an empty canvas.", meta: "No setup" },
];

export function EmptyState() {
  const empty = useStore((s) => s.diagram.nodes.length === 0);
  const started = useStore((s) => s.started);
  const panelOpen = useStore((s) => s.panelOpen);
  if (!empty) return null;
  const offset = { paddingLeft: panelOpen ? 316 : 0 }; // centre in the free canvas, clear of the panel
  if (started) {
    return (
      <div className="pointer-events-none absolute inset-0 grid place-items-center" style={offset}>
        <p className="rounded-lg border border-hair bg-surface px-4 py-2 text-sm text-ink-muted shadow-hair">Double-click anywhere to add a step</p>
      </div>
    );
  }
  return (
    <div className="absolute inset-y-0 right-0" style={{ left: panelOpen ? 316 : 0 }} data-testid="empty-state">
      <EmptyStateBlock starters={STARTERS} onPick={(id) => A.newFromTemplate(id as "userflow" | "taskflow" | "blank")} onAddStep={() => A.addNodeInView()} onPasteMermaid={() => setState({ mermaidOpen: true })} />
    </div>
  );
}

// ---- mermaid ---------------------------------------------------------------------------------
export function MermaidDialog() {
  const open = useStore((s) => s.mermaidOpen);
  const [src, setSrc] = useState("");
  const [diags, setDiags] = useState<Diag[] | null>(null);
  const close = () => { setState({ mermaidOpen: false }); setDiags(null); };
  return (
    <Dialog open={open} onOpenChange={(o) => (o ? setState({ mermaidOpen: true }) : close())}>
      <DialogContent className="max-w-[520px] gap-4 rounded-2xl border-hair bg-surface p-4">
        <DialogHeader>
          <DialogTitle className="text-base font-semibold">Paste Mermaid</DialogTitle>
          <DialogDescription className="text-sm">A flowchart or graph. It replaces the steps and connections; your brand kit stays.</DialogDescription>
        </DialogHeader>
        <Textarea
          value={src}
          onChange={(e) => { setSrc(e.target.value); setDiags(null); }}
          rows={9}
          autoFocus
          spellCheck={false}
          aria-label="Mermaid source"
          placeholder={"flowchart LR\n  A[Start] --> B{Ready?}\n  B -->|yes| C[Ship]"}
          className="resize-none border-hair bg-sunk font-mono text-[13px] leading-5"
        />
        {diags && diags.length > 0 && (
          <ul className="max-h-24 overflow-y-auto text-sm text-ink-muted">
            {diags.map((d, i) => <li key={i}>Line {d.line}: {d.message}</li>)}
          </ul>
        )}
        <DialogFooter className="gap-2 sm:justify-between">
          <Button variant="ghost" size="sm" onClick={async () => { try { setSrc(await navigator.clipboard.readText()); setDiags(null); } catch { /* clipboard blocked */ } }}>
            <Icon name="duplicate" />Paste from clipboard
          </Button>
          <div className="flex gap-2">
            <Button variant="secondary" size="sm" onClick={close}>Cancel</Button>
            <Button size="sm" disabled={!src.trim()} onClick={() => {
              const d = A.importMermaid(src);
              if (d.length) setDiags(d);
              const ok = useStoreNodes();
              if (ok) close();
            }}>Import</Button>
          </div>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
import { getState } from "@/state/store";
const useStoreNodes = () => getState().diagram.nodes.length > 0 && getState().started;
