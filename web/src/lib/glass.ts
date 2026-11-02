// Tinted-glass group colours (DESIGN.md > Diagram nodes and groups). The "vibrant step" of a role colour is its
// Tailwind 400 (dark) or 500 (light) step when the kit uses a Tailwind colour, else a mix toward white.
import type { Rgb } from "./types";

const hex = (c: Rgb) => "#" + c.map((v) => v.toString(16).padStart(2, "0")).join("");
const mix = (c: Rgb, t: number): Rgb => c.map((v) => Math.round(v + (255 - v) * t)) as Rgb;

/** Tailwind 600 (or 500) value -> [500, 400]. */
const STEPS: Record<string, [string, string]> = {
  "#c026d3": ["#d946ef", "#e879f9"], // fuchsia
  "#7c3aed": ["#8b5cf6", "#a78bfa"], // violet
  "#f59e0b": ["#f59e0b", "#fbbf24"], // amber-500
  "#d97706": ["#f59e0b", "#fbbf24"], // amber-600
  "#71717a": ["#71717a", "#a1a1aa"], // zinc-500
  "#16a34a": ["#22c55e", "#4ade80"], // green
  "#dc2626": ["#ef4444", "#f87171"], // red
  "#2563eb": ["#3b82f6", "#60a5fa"], // blue
  "#0891b2": ["#06b6d4", "#22d3ee"], // cyan
};

export function vibrant(c: Rgb, dark: boolean): string {
  const s = STEPS[hex(c)];
  if (s) return dark ? s[1] : s[0];
  return hex(mix(c, dark ? 0.25 : 0.12));
}

export const rgba = (c: Rgb, a: number) => `rgba(${c[0]}, ${c[1]}, ${c[2]}, ${a})`;

export interface GlassStyle { fill: string; border: string; title: string }

export function glass(c: Rgb, dark: boolean): GlassStyle {
  const v = vibrant(c, dark);
  return { fill: rgba(c, dark ? 0.1 : 0.08), border: v, title: v };
}
