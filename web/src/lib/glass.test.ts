import { describe, expect, it } from "vitest";
import { glass, rgba, vibrant } from "./glass";

describe("glass groups", () => {
  it("uses the Tailwind 400 step in dark and 500 in light", () => {
    expect(vibrant([0x7c, 0x3a, 0xed], true)).toBe("#a78bfa");
    expect(vibrant([0x7c, 0x3a, 0xed], false)).toBe("#8b5cf6");
  });
  it("lightens an unknown colour instead", () => {
    expect(vibrant([0, 0, 0], true)).toBe("#404040");
    expect(vibrant([100, 100, 100], false)).toBe("#777777");
  });
  it("tints the fill at 10% dark and 8% light", () => {
    expect(glass([10, 20, 30], true).fill).toBe("rgba(10, 20, 30, 0.1)");
    expect(glass([10, 20, 30], false).fill).toBe(rgba([10, 20, 30], 0.08));
  });
});
