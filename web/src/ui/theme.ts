// Dark by default; light is a choice that is remembered. The system appearance is never followed.
import { useSyncExternalStore } from "react";

export type Theme = "dark" | "light";
const KEY = "rovelo:theme";
const listeners = new Set<() => void>();

function read(): Theme {
  const forced = new URLSearchParams(location.search).get("theme"); // ?theme= is for screenshots
  if (forced === "light" || forced === "dark") return forced;
  try {
    return localStorage.getItem(KEY) === "light" ? "light" : "dark";
  } catch {
    return "dark";
  }
}

let current: Theme = typeof window === "undefined" ? "dark" : read();

export function applyTheme(t: Theme = current) {
  document.documentElement.classList.toggle("dark", t === "dark");
}

export function setTheme(t: Theme) {
  current = t;
  try {
    localStorage.setItem(KEY, t);
  } catch { /* private mode: the choice lasts for this session only */ }
  applyTheme(t);
  listeners.forEach((l) => l());
}

export const toggleTheme = () => setTheme(current === "dark" ? "light" : "dark");

export function useTheme(): Theme {
  return useSyncExternalStore((l) => (listeners.add(l), () => listeners.delete(l)), () => current);
}
