import { describe, expect, it } from "vitest";
import { LIMIT, canRedo, canUndo, commit, emptyHistory, redo, undo } from "./history";

describe("history", () => {
  it("starts empty", () => {
    const h = emptyHistory<number>();
    expect(canUndo(h)).toBe(false);
    expect(canRedo(h)).toBe(false);
    expect(undo(h, 1)).toBeNull();
    expect(redo(h, 1)).toBeNull();
  });

  it("undo and redo walk the snapshots", () => {
    let h = commit(emptyHistory<number>(), 1); // 1 -> 2
    h = commit(h, 2); // 2 -> 3
    const u1 = undo(h, 3)!;
    expect(u1.value).toBe(2);
    const u2 = undo(u1.history, u1.value)!;
    expect(u2.value).toBe(1);
    expect(undo(u2.history, u2.value)).toBeNull();
    const r1 = redo(u2.history, u2.value)!;
    expect(r1.value).toBe(2);
    expect(redo(r1.history, r1.value)!.value).toBe(3);
  });

  it("a new edit clears redo", () => {
    let h = commit(emptyHistory<string>(), "a");
    const u = undo(h, "b")!;
    expect(canRedo(u.history)).toBe(true);
    h = commit(u.history, "a");
    expect(canRedo(h)).toBe(false);
  });

  it("is capped and drops the oldest", () => {
    let h = emptyHistory<number>();
    for (let i = 0; i < LIMIT + 10; i++) h = commit(h, i);
    expect(h.past.length).toBe(LIMIT);
    expect(h.past[0]).toBe(10);
  });

  it("does not mutate its input", () => {
    const h = commit(emptyHistory<number>(), 1);
    const before = JSON.stringify(h);
    undo(h, 2);
    redo(h, 2);
    expect(JSON.stringify(h)).toBe(before);
  });
});
