import { describe, expect, it } from "vitest";
import { clamp, decimals, fineValue, formatValue, handleX, keyDelta, onValueArea, parseTyped, ratio, scrubValue, snapValue, valueFromX, widestForm, zeros } from "./slider";

describe("inset slider maths", () => {
  it("reads decimals from the step", () => {
    expect(decimals(1)).toBe(0);
    expect(decimals(0.5)).toBe(1);
    expect(decimals(0.05)).toBe(2);
    expect(decimals(1e-7)).toBe(7);
  });

  it("snaps to the step, clamps, and drops float noise", () => {
    expect(snapValue(7.4, 0, 24, 1)).toBe(7);
    expect(snapValue(1.26, 0.5, 4, 0.5)).toBe(1.5);
    expect(snapValue(0.1 + 0.2, 0, 1, 0.1)).toBe(0.3);
    expect(snapValue(99, 0, 24, 1)).toBe(24);
    expect(snapValue(-3, 0, 24, 1)).toBe(0);
    expect(clamp(5, 0, 3)).toBe(3);
  });

  it("places the handle 8 in from each end", () => {
    expect(handleX(0, 200)).toBe(8);
    expect(handleX(1, 200)).toBe(192);
    expect(handleX(0.5, 200)).toBe(100);
    expect(ratio(12, 0, 24)).toBe(0.5);
    expect(ratio(50, 0, 24)).toBe(1);
  });

  it("a press at row-x sets the value, inverse of handleX", () => {
    expect(valueFromX(8, 200, 0, 24, 1)).toBe(0);
    expect(valueFromX(192, 200, 0, 24, 1)).toBe(24);
    expect(valueFromX(100, 200, 0, 24, 1)).toBe(12);
    expect(valueFromX(-50, 200, 0, 24, 1)).toBe(0);
    expect(valueFromX(900, 200, 0, 24, 1)).toBe(24);
    for (const v of [3, 10, 17]) expect(valueFromX(handleX(ratio(v, 0, 24), 300), 300, 0, 24, 1)).toBe(v);
  });

  it("Alt moves at 0.15x from where the drag began", () => {
    // 100px across a 216 inner span of 0..216 is 100 units; fine = 15
    expect(fineValue(50, 100, 232, 0, 216, 1)).toBe(65);
    expect(fineValue(50, -1000, 232, 0, 216, 1)).toBe(0);
    expect(fineValue(50, 0, 232, 0, 216, 1)).toBe(50);
  });

  it("a value with no span scrubs a step a point", () => {
    expect(scrubValue(10, 5, 0, 100, 2)).toBe(20);
    expect(scrubValue(10, 5, 0, 100, 2, 0.15)).toBe(12);
    expect(scrubValue(10, 1000, 0, 100, 2)).toBe(100);
  });

  it("formats with the step's precision and sizes the number box by its widest form", () => {
    expect(formatValue(14, 1, "px")).toBe("14px");
    expect(formatValue(1.5, 0.5, "px")).toBe("1.5px");
    expect(zeros("12.5px")).toBe("00.0px");
    expect(widestForm(0.5, 4, 0.5, "px")).toBe("0.0px");
    expect(widestForm(0, 24, 1, "px")).toBe("00px");
    expect(widestForm(-10, 100, 1, "%").length).toBe(4);
  });

  it("arrow keys move one step, Shift ten", () => {
    expect(keyDelta("ArrowRight", false, 2)).toBe(2);
    expect(keyDelta("ArrowUp", true, 2)).toBe(20);
    expect(keyDelta("ArrowLeft", false, 0.5)).toBe(-0.5);
    expect(keyDelta("ArrowDown", true, 1)).toBe(-10);
    expect(keyDelta("Home", false, 1)).toBe("min");
    expect(keyDelta("End", false, 1)).toBe("max");
    expect(keyDelta("a", false, 1)).toBeNull();
  });

  it("typed values are clamped and stepped; junk is refused", () => {
    expect(parseTyped("13.6", 0, 24, 1)).toBe(14);
    expect(parseTyped("  99px ", 0, 24, 1)).toBe(24);
    expect(parseTyped("1.3", 0.5, 4, 0.5)).toBe(1.5);
    expect(parseTyped("abc", 0, 24, 1)).toBeNull();
    expect(parseTyped("", 0, 24, 1)).toBeNull();
  });

  it("tells the number's area from the rest of the row", () => {
    expect(onValueArea(190, 200, 28)).toBe(true);
    expect(onValueArea(100, 200, 28)).toBe(false);
    expect(onValueArea(150, 200, 28)).toBe(false);
    expect(onValueArea(151, 200, 28)).toBe(true);
  });
});
