// Code tab: the diagram as Rovelo text. Typing re-parses after a pause; the canvas updates and
// diagnostics show beside their lines. Canvas edits regenerate the text unless you are mid-edit.
import { useEffect, useMemo, useRef, useState } from "react";
import { core } from "@/wasm";
import { Button } from "@/components/ui/button";
import { mergeParsed } from "@/lib/ops";
import type { Diag, Diagram } from "@/lib/types";
import { edit, getState, setState, toast, useStore } from "@/state/store";
import { Icon } from "./icons";
import { cn } from "@/lib/utils";

const LH = 20;

export function CodeTab() {
  const diagram = useStore((s) => s.diagram);
  const [text, setText] = useState(() => core.serialize(diagram));
  const [diags, setDiags] = useState<Diag[]>([]);
  const own = useRef<Diagram | null>(null); // the diagram our last parse produced
  const dirty = useRef(false); // typed since the last sync
  const ta = useRef<HTMLTextAreaElement>(null);
  const gutter = useRef<HTMLDivElement>(null);
  const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

  // Canvas -> text, unless the change came from this editor.
  useEffect(() => {
    if (own.current === diagram) return;
    if (document.activeElement === ta.current && dirty.current) return;
    setText(core.serialize(diagram));
    setDiags([]);
    dirty.current = false;
  }, [diagram]);

  useEffect(() => () => clearTimeout(timer.current), []);

  const reparse = (src: string) => {
    const r = core.parse(src);
    setDiags(r.diagnostics);
    if (r.diagnostics.some((d) => d.severity === "error") && !r.diagram.nodes.length && src.trim()) return;
    const old = getState().diagram;
    const merged = core.layoutUnplaced(mergeParsed(old, r.diagram));
    const same = JSON.stringify({ ...merged, brand: 0 }) === JSON.stringify({ ...old, brand: 0 });
    if (same) return;
    // Keep the parsed diagram object so the sync effect can tell the change is ours.
    edit(() => (own.current = merged));
    setState({ started: true });
  };

  const onChange = (v: string) => {
    setText(v);
    dirty.current = true;
    clearTimeout(timer.current);
    timer.current = setTimeout(() => { reparse(v); dirty.current = false; }, 350);
  };

  const lines = text.split("\n").length;
  const byLine = useMemo(() => {
    const m = new Map<number, Diag>();
    for (const d of diags) if (!m.has(d.line) || d.severity === "error") m.set(d.line, d);
    return m;
  }, [diags]);
  const errors = diags.filter((d) => d.severity === "error").length;
  const warns = diags.filter((d) => d.severity === "warning").length;

  const jump = (line: number) => {
    const el = ta.current;
    if (!el) return;
    const parts = text.split("\n");
    let start = 0;
    for (let i = 0; i < line - 1 && i < parts.length; i++) start += parts[i].length + 1;
    el.focus();
    el.setSelectionRange(start, start + (parts[line - 1]?.length ?? 0));
    el.scrollTop = Math.max(0, (line - 3) * LH);
  };

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-4 p-4 pt-0">
      <div className="flex min-h-0 flex-1 overflow-hidden rounded-lg border border-hair bg-sunk focus-within:border-brand focus-within:ring-[3px] focus-within:ring-brand-soft">
        <div ref={gutter} aria-hidden className="w-8 shrink-0 select-none overflow-hidden border-r border-hair py-2 pr-2 text-right font-mono text-[13px] text-ink-muted" style={{ lineHeight: `${LH}px` }}>
          {Array.from({ length: lines }, (_, i) => {
            const d = byLine.get(i + 1);
            return (
              <div key={i} className={cn(d?.severity === "error" && "font-semibold text-bad", d?.severity === "warning" && "font-semibold text-warn")}>
                {d ? (d.severity === "error" ? "!" : "·") : ""}{i + 1}
              </div>
            );
          })}
        </div>
        <textarea
          ref={ta}
          value={text}
          onChange={(e) => onChange(e.target.value)}
          onScroll={(e) => { if (gutter.current) gutter.current.scrollTop = e.currentTarget.scrollTop; }}
          wrap="off"
          spellCheck={false}
          aria-label="Rovelo code"
          className="min-w-0 flex-1 resize-none bg-transparent px-2 py-2 font-mono text-[13px] outline-none"
          style={{ lineHeight: `${LH}px`, tabSize: 2 }}
        />
      </div>
      <div className="max-h-24 min-h-6 shrink-0 overflow-y-auto text-sm" aria-live="polite">
        {diags.length === 0 ? (
          <span className="flex items-center gap-2 text-ink-muted"><Icon name="done" size={14} className="text-ok" />No problems</span>
        ) : (
          <>
            <div className="mb-1 text-ink-muted">{errors ? `${errors} error${errors > 1 ? "s" : ""}` : ""}{errors && warns ? ", " : ""}{warns ? `${warns} warning${warns > 1 ? "s" : ""}` : ""}</div>
            <ul className="flex flex-col gap-1">
              {diags.map((d, i) => (
                <li key={i}>
                  <button onClick={() => jump(d.line)} className="flex w-full gap-2 rounded-lg px-2 py-1 text-left hover:bg-accent">
                    <span className={cn("w-16 shrink-0 font-mono text-[13px]", d.severity === "error" ? "text-bad" : "text-warn")}>{d.severity === "error" ? "Error" : d.severity === "warning" ? "Warn" : "Info"} {d.line}</span>
                    <span className="min-w-0 flex-1">{d.message}</span>
                  </button>
                </li>
              ))}
            </ul>
          </>
        )}
      </div>
      <div className="flex gap-2">
        <Button variant="secondary" size="sm" className="flex-1" onClick={() => setState({ mermaidOpen: true })}><Icon name="code" />Paste Mermaid…</Button>
        <Button variant="secondary" size="sm" onClick={() => { navigator.clipboard?.writeText(text).then(() => toast("Copied")).catch(() => toast("Copy failed")); }}><Icon name="duplicate" />Copy</Button>
      </div>
    </div>
  );
}
