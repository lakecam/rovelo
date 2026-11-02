// Small shared controls built on shadcn / Radix primitives. Styling follows DESIGN.md.
import { forwardRef, type ComponentProps, type ReactNode } from "react";
import { ToggleGroup as TG } from "radix-ui";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

export function Tip({ label, keys, children, side = "bottom" }: { label: string; keys?: string; children: ReactNode; side?: "top" | "bottom" | "left" | "right" }) {
  return (
    <Tooltip>
      <TooltipTrigger asChild>{children}</TooltipTrigger>
      <TooltipContent side={side} sideOffset={8} className="flex items-center gap-2 text-xs">
        {label}
        {keys && <kbd className="rounded bg-white/15 px-1 font-sans text-xs opacity-80">{keys}</kbd>}
      </TooltipContent>
    </Tooltip>
  );
}

type IconBtnProps = Omit<ComponentProps<typeof Button>, "size" | "variant"> & { label: string; keys?: string; active?: boolean; side?: "top" | "bottom" | "left" | "right" };

/** 32px square icon button with a tooltip; accent icon when active. */
export const IconBtn = forwardRef<HTMLButtonElement, IconBtnProps>(function IconBtn({ label, keys, active, side, className, children, ...p }, ref) {
  return (
    <Tip label={label} keys={keys} side={side}>
      <Button ref={ref} variant="ghost" size="icon-sm" aria-label={label} aria-pressed={active} data-active={active || undefined} className={cn("text-ink-muted hover:text-ink", active && "bg-accent text-ink", className)} {...p}>
        {children}
      </Button>
    </Tip>
  );
});

export const Divider = ({ className }: { className?: string }) => <span aria-hidden className={cn("mx-1 h-4 w-px shrink-0 bg-hair", className)} />;

export interface SegOption<T extends string> { value: T; label: ReactNode; title?: string }

/** Segmented control: sunk track, white active segment with hairline and shadow. */
export function Segmented<T extends string>({ value, onChange, options, className, label }: {
  value: T | null; onChange: (v: T) => void; options: SegOption<T>[]; className?: string; label: string;
}) {
  return (
    <TG.Root
      type="single"
      value={value ?? ""}
      onValueChange={(v) => v && onChange(v as T)}
      aria-label={label}
      className={cn("inline-flex h-8 w-full items-center rounded-[10px] bg-sunk p-0.5", className)}
    >
      {options.map((o) => (
        <TG.Item
          key={o.value}
          value={o.value}
          title={o.title}
          className={cn(
            "flex h-7 flex-1 items-center justify-center gap-2 rounded-lg border border-transparent px-2 text-sm font-medium text-ink-muted transition-[background-color,color] duration-150 ease-out hover:text-ink",
            "data-[state=on]:border-hair data-[state=on]:bg-surface data-[state=on]:text-ink data-[state=on]:shadow-hair dark:data-[state=on]:bg-zinc-700 dark:data-[state=on]:border-transparent",
          )}
        >
          {o.label}
        </TG.Item>
      ))}
    </TG.Root>
  );
}

export const rgbCss = (c: number[]) => `rgb(${c[0]} ${c[1]} ${c[2]})`;
export const hexOf = (c: number[]) => "#" + c.map((v) => v.toString(16).padStart(2, "0")).join("");
export const parseHex = (s: string): [number, number, number] | null => {
  const m = /^#?([0-9a-f]{3}|[0-9a-f]{6})$/i.exec(s.trim());
  if (!m) return null;
  let h = m[1];
  if (h.length === 3) h = h.split("").map((c) => c + c).join("");
  return [parseInt(h.slice(0, 2), 16), parseInt(h.slice(2, 4), 16), parseInt(h.slice(4, 6), 16)];
};

/** 20px swatch, 1px border; selected gets a 2px accent ring. */
export function Swatch({ color, selected, onClick, label, size = 24 }: { color: string; selected?: boolean; onClick?: () => void; label: string; size?: number }) {
  return (
    <Tip label={label} side="top">
      <button
        type="button"
        onClick={onClick}
        aria-label={label}
        aria-pressed={selected}
        className={cn("shrink-0 rounded-full border border-black/15 transition-transform duration-150 hover:scale-110 dark:border-white/20", selected && "ring-2 ring-ink ring-offset-2 ring-offset-surface")}
        style={{ width: size, height: size, background: color }}
      />
    </Tip>
  );
}

export const SectionLabel = ({ children, className }: { children: ReactNode; className?: string }) => (
  <div className={cn("text-xs font-medium text-ink-muted", className)}>{children}</div>
);

export function Chip({ tone = "plain", children, className, ...p }: ComponentProps<"span"> & { tone?: "plain" | "late" | "today" | "accent" | "ok" }) {
  return (
    <span
      className={cn(
        "inline-flex h-6 shrink-0 items-center gap-1 rounded-md border px-2 text-xs font-medium leading-none",
        tone === "plain" && "border-hair text-ink-muted",
        tone === "late" && "border-bad/30 bg-bad-soft text-bad",
        tone === "today" && "border-warn/30 bg-warn-soft text-warn",
        tone === "accent" && "border-hair-strong bg-accent text-ink",
        tone === "ok" && "border-ok/30 text-ok",
        className,
      )}
      {...p}
    >
      {children}
    </span>
  );
}
