// The core draws its own line glyphs for node icons. The app swaps them for the Nucleo duotone files
// (same twelve roles) both on the canvas and in exports, so the diagram and the chrome share one set.
import { iconInner } from "@/ui/icons";
import type { Diagram } from "./types";

const ALIASES: Record<string, string> = {
  person: "user", account: "user", profile: "user",
  email: "mail", envelope: "mail", inbox: "mail",
  house: "home",
  basket: "cart", "shopping-cart": "cart", checkout: "cart",
  tick: "check", done: "check", ok: "check",
  settings: "gear", cog: "gear",
  magnifier: "search", find: "search",
  notification: "bell", alert: "bell",
  "credit-card": "card", payment: "card",
  mobile: "phone", call: "phone",
};
const FILE: Record<string, string> = { check: "done" };

export const canonicalIcon = (name: string | null | undefined): string | null => {
  if (!name) return null;
  const n = name.trim().toLowerCase();
  const c = ALIASES[n] ?? n;
  return ["user", "lock", "mail", "home", "cart", "check", "star", "gear", "search", "bell", "card", "phone"].includes(c) ? c : null;
};

const GLYPH = /<g transform="translate\(([^ ]+) ([^)]+)\) scale\(([^)]+)\)" fill="none" stroke="([^"]+)"[^>]*>[\s\S]*?<\/g>/;

/** Replace the core's glyph inside one node's markup. Unknown icons are left as drawn. */
export function swapNodeIcon(markup: string, icon: string | null | undefined): string {
  const c = canonicalIcon(icon);
  const inner = c && iconInner(FILE[c] ?? c);
  if (!inner) return markup;
  return markup.replace(GLYPH, (_m, x, y, s, color) => `<g transform="translate(${x} ${y}) scale(${(Number(s) * 24) / 18})" color="${color}">${inner}</g>`);
}

const esc = (s: string) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;").replace(/'/g, "&apos;");

/** Apply the swap to a whole exported SVG document. */
export function swapExportIcons(svg: string, d: Diagram): string {
  let out = svg;
  for (const n of d.nodes) {
    if (!canonicalIcon(n.icon)) continue;
    const open = `<g data-node="${esc(n.id)}">`;
    const a = out.indexOf(open);
    if (a < 0) continue;
    const next = out.indexOf("<g data-", a + open.length);
    const end = next < 0 ? out.lastIndexOf("</svg>") : next;
    out = out.slice(0, a) + swapNodeIcon(out.slice(a, end), n.icon) + out.slice(end);
  }
  return out;
}
