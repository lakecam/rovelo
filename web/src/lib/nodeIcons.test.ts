import { describe, expect, it } from "vitest";
import { canonicalIcon, swapExportIcons, swapNodeIcon } from "./nodeIcons";
import type { Diagram } from "./types";

const glyph = `<g transform="translate(10 20) scale(0.75)" fill="none" stroke="#123456" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="8" r="4"/></g>`;

describe("node icons", () => {
  it("canonicalises names and aliases like the core", () => {
    expect(canonicalIcon("Person")).toBe("user");
    expect(canonicalIcon("shopping-cart")).toBe("cart");
    expect(canonicalIcon("done")).toBe("check");
    expect(canonicalIcon("dragon")).toBeNull();
    expect(canonicalIcon(null)).toBeNull();
  });

  it("swaps the core glyph for the Nucleo file, keeping place and colour", () => {
    const out = swapNodeIcon(`<rect/>${glyph}<text>x</text>`, "user");
    expect(out).toContain(`translate(10 20) scale(1)`);
    expect(out).toContain(`color="#123456"`);
    expect(out).toContain("opacity=\"0.4\"");
    expect(out).not.toContain("stroke-width");
    expect(out.endsWith("<text>x</text>")).toBe(true);
  });

  it("leaves unknown icons alone", () => {
    expect(swapNodeIcon(glyph, "dragon")).toBe(glyph);
    expect(swapNodeIcon(glyph, null)).toBe(glyph);
  });

  it("swaps inside the right node of an exported document", () => {
    const svg = `<svg><g data-node="a"><rect/>${glyph}</g><g data-node="b"><rect/>${glyph}</g></svg>`;
    const d = { nodes: [{ id: "b", icon: "lock" }] } as unknown as Diagram;
    const out = swapExportIcons(svg, d);
    expect(out.split("scale(1)").length - 1).toBe(1);
    expect(out.indexOf("scale(1)")).toBeGreaterThan(out.indexOf('data-node="b"'));
  });
});
