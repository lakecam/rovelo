// Pure maths of the Inset slider (WoopKIT LabelSlider, Theme.swift `insetBody`).
export const PAD = 8; // handle travel is inset 8 from each end
export const FINE = 0.15; // Alt drags at 0.15x

export const clamp = (v: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, v));

/** Digits after the point that a step needs: 1 -> 0, 0.5 -> 1, 0.05 -> 2. */
export function decimals(step: number): number {
  const s = String(step);
  if (s.includes("e-")) return Number(s.split("e-")[1]);
  return s.includes(".") ? s.split(".")[1].length : 0;
}

/** Round to the step (from zero, as the original does), clamp, and drop float noise. */
export function snapValue(raw: number, min: number, max: number, step: number): number {
  const v = clamp(Math.round(raw / step) * step, min, max);
  return Number(v.toFixed(decimals(step)));
}

/** 0..1 position of a value in its range. */
export const ratio = (value: number, min: number, max: number) => clamp((value - min) / Math.max(max - min, 1e-9), 0, 1);

/** Handle centre x in a row `w` wide: 8 + k * (w - 16). */
export const handleX = (k: number, w: number) => PAD + k * (w - 2 * PAD);

/** A press or drag at row-x sets the value directly: the pointer carries the handle. */
export function valueFromX(x: number, w: number, min: number, max: number, step: number): number {
  const inner = Math.max(1, w - 2 * PAD);
  return snapValue(min + ((x - PAD) / inner) * (max - min), min, max, step);
}

/** Alt drag: move from where the drag started by `dx` pixels at 0.15x. */
export function fineValue(start: number, dx: number, w: number, min: number, max: number, step: number, fine = FINE): number {
  const inner = Math.max(1, w - 2 * PAD);
  return snapValue(start + (dx / inner) * (max - min) * fine, min, max, step);
}

/** Scrub (a value with no natural span): one step per point, from where the drag began. */
export const scrubValue = (start: number, dx: number, min: number, max: number, step: number, fine = 1) =>
  snapValue(start + dx * step * fine, min, max, step);

export const formatValue = (v: number, step: number, unit = "") => `${v.toFixed(decimals(step))}${unit}`;

/** Any digits will do for sizing: the widest digit is the zero. */
export const zeros = (s: string) => s.replace(/\d/g, "0");

/** The longest the number reads, so its box never changes width. */
export function widestForm(min: number, max: number, step: number, unit = ""): string {
  const forms = [formatValue(min, step, unit), formatValue(max, step, unit)].map(zeros);
  return forms.reduce((a, b) => (b.length > a.length ? b : a));
}

/** Arrow keys move one step, Shift ten; null for any other key. */
export function keyDelta(key: string, shift: boolean, step: number): number | "min" | "max" | null {
  const n = shift ? 10 : 1;
  if (key === "ArrowRight" || key === "ArrowUp") return step * n;
  if (key === "ArrowLeft" || key === "ArrowDown") return -step * n;
  if (key === "Home") return "min";
  if (key === "End") return "max";
  return null;
}

/** Text typed into the number: clamped and rounded to the step; null if it is not a number. */
export function parseTyped(text: string, min: number, max: number, step: number): number | null {
  const n = Number.parseFloat(text.replace(/[^\d.+-]/g, ""));
  return Number.isFinite(n) ? snapValue(n, min, max, step) : null;
}

/** The number's own area at the row's right, where a double-click types instead of resets. */
export const onValueArea = (x: number, w: number, valueWidth: number, inset = 16) => x > w - inset - valueWidth - 6;
