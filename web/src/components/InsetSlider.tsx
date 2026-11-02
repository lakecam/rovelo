// WoopKIT's LabelSlider, Inset look, in our tokens (DESIGN.md > Components > Slider). One 32-tall row is the
// track; label inside left, value inside right; a 16% ink wash to a 4x20 handle that is always shown.
import { useEffect, useRef, useState } from "react";
import { cn } from "@/lib/utils";
import { FINE, formatValue, fineValue, handleX, keyDelta, onValueArea, parseTyped, ratio, scrubValue, snapValue, valueFromX, widestForm, zeros } from "@/lib/slider";

export interface InsetSliderProps {
  label: string;
  value: number;
  min: number;
  max: number;
  step?: number;
  unit?: string;
  defaultValue?: number;
  /** False for a value with no natural span: no wash, no handle, drags scrub a step a point. */
  filled?: boolean;
  onChange: (v: number) => void;
  /** Called when a drag, key press or typed entry ends: one undo step. */
  onCommit?: () => void;
  className?: string;
}

const EASE = "cubic-bezier(0.25,0,0,1)";

export function InsetSlider({ label, value, min, max, step = 1, unit = "", defaultValue, filled = true, onChange, onCommit, className }: InsetSliderProps) {
  const row = useRef<HTMLDivElement>(null);
  const numRef = useRef<HTMLSpanElement>(null);
  const drag = useRef<{ start: number; x0: number } | null>(null);
  const lastDown = useRef(0);
  const before = useRef<number | null>(null); // the value before the press that began a double-click
  const [w, setW] = useState(240);
  const [hover, setHover] = useState(false);
  const [dragging, setDragging] = useState(false);
  const [editing, setEditing] = useState(false);
  const [valueW, setValueW] = useState(28);

  useEffect(() => {
    const el = row.current;
    if (!el) return;
    const ro = new ResizeObserver(() => setW(el.getBoundingClientRect().width));
    ro.observe(el);
    setW(el.getBoundingClientRect().width);
    return () => ro.disconnect();
  }, []);
  useEffect(() => {
    if (numRef.current) setValueW(numRef.current.getBoundingClientRect().width);
  }, [value, editing, unit]);

  const text = formatValue(value, step, unit);
  const widest = widestForm(min, max, step, unit);
  const k = ratio(value, min, max);
  const x = handleX(k, w);
  const changed = defaultValue !== undefined && value !== defaultValue;
  const lit = hover || dragging;
  const localX = (e: { clientX: number }) => e.clientX - row.current!.getBoundingClientRect().left;

  const move = (e: React.PointerEvent) => {
    const d = drag.current;
    if (!d) return;
    const fine = e.altKey;
    let v: number;
    if (!filled) v = scrubValue(d.start, e.clientX - d.x0, min, max, step, fine ? FINE : 1);
    else if (fine && e.clientX !== d.x0) v = fineValue(d.start, e.clientX - d.x0, w, min, max, step);
    else v = valueFromX(localX(e), w, min, max, step);
    if (v !== value) onChange(v);
  };

  const down = (e: React.PointerEvent) => {
    if (editing || e.button !== 0) return;
    try { e.currentTarget.setPointerCapture(e.pointerId); } catch { /* synthetic or already released */ }
    const now = Date.now();
    if (now - lastDown.current > 400) before.current = value;
    lastDown.current = now;
    drag.current = { start: value, x0: e.clientX };
    setDragging(true);
    row.current?.focus();
    if (filled) {
      const v = valueFromX(localX(e), w, min, max, step); // a press anywhere sets the value at once
      if (v !== value) onChange(v);
    }
  };

  const up = () => {
    if (!drag.current) return;
    drag.current = null;
    setDragging(false);
    onCommit?.();
  };

  const dbl = (e: React.MouseEvent) => {
    if (editing) return;
    if (onValueArea(localX(e), w, valueW)) {
      // Only a press pair that just happened moved the value; put back what was there before it.
      if (filled && before.current !== null && before.current !== value && Date.now() - lastDown.current < 600) onChange(before.current);
      setEditing(true);
    } else if (defaultValue !== undefined) {
      onChange(defaultValue);
      onCommit?.();
    }
  };

  const keys = (e: React.KeyboardEvent) => {
    if (editing) return;
    const d = keyDelta(e.key, e.shiftKey, step);
    if (d === null) return;
    e.preventDefault();
    const v = d === "min" ? min : d === "max" ? max : snapValue(value + d, min, max, step);
    if (v !== value) onChange(v);
    onCommit?.();
  };

  return (
    <div
      ref={row}
      role="slider"
      tabIndex={0}
      aria-label={label}
      aria-valuemin={min}
      aria-valuemax={max}
      aria-valuenow={value}
      aria-valuetext={text}
      onPointerDown={down}
      onPointerMove={move}
      onPointerUp={up}
      onPointerCancel={up}
      onDoubleClick={dbl}
      onKeyDown={keys}
      onPointerEnter={() => setHover(true)}
      onPointerLeave={() => setHover(false)}
      className={cn("relative h-8 w-full cursor-ew-resize touch-none overflow-hidden rounded-lg bg-sunk text-sm select-none", className)}
    >
      {filled && <div aria-hidden className="pointer-events-none absolute inset-y-0 left-0 bg-ink/[0.16]" style={{ width: x }} />}
      {filled && (
        <div aria-hidden className="pointer-events-none absolute top-1/2 -mt-2.5 h-5 w-1" style={{ left: x - 2 }}>
          <div className="size-full rounded-[2px] bg-ink" style={{ transform: `scaleY(${dragging ? 1.25 : 1})`, transition: `transform 150ms ${EASE}` }} />
        </div>
      )}
      <div className="pointer-events-none absolute inset-0 flex items-center justify-between px-4">
        <span className={cn("truncate", changed ? "font-medium text-ink" : "text-ink-muted")}>{label}</span>
        {editing ? (
          <NumberField initial={value} width={valueW} min={min} max={max} step={step} onDone={(v) => { setEditing(false); if (v !== null && v !== value) onChange(v); onCommit?.(); }} />
        ) : (
          // Fixed-width box sized by the widest form (digits as zeros), right-aligned: it never moves a neighbour.
          <span ref={numRef} className={cn("relative inline-block text-right", lit ? "text-ink" : "text-ink-muted")}>
            <span aria-hidden className="invisible">{zeros(widest)}</span>
            <span className="absolute inset-y-0 right-0">{text}</span>
          </span>
        )}
      </div>
    </div>
  );
}

/** The number turned into a field, same font: Enter or a click away commits, Esc keeps the old value. */
function NumberField({ initial, width, min, max, step, onDone }: { initial: number; width: number; min: number; max: number; step: number; onDone: (v: number | null) => void }) {
  const ref = useRef<HTMLInputElement>(null);
  const done = useRef(false);
  useEffect(() => { ref.current?.focus(); ref.current?.select(); }, []);
  const finish = (commit: boolean) => {
    if (done.current) return;
    done.current = true;
    onDone(commit ? parseTyped(ref.current?.value ?? "", min, max, step) : null);
  };
  return (
    <input
      ref={ref}
      defaultValue={String(initial)}
      aria-label="Value"
      inputMode="decimal"
      onPointerDown={(e) => e.stopPropagation()}
      onDoubleClick={(e) => e.stopPropagation()}
      onBlur={() => finish(true)}
      onKeyDown={(e) => {
        e.stopPropagation();
        if (e.key === "Enter") finish(true);
        if (e.key === "Escape") finish(false);
      }}
      style={{ width: Math.max(width, 28) + 6 }}
      className="pointer-events-auto -mx-[3px] -my-px rounded-[4px] border border-ink/35 bg-ink/[0.08] px-[2px] py-0 text-right text-sm text-ink outline-none"
    />
  );
}
