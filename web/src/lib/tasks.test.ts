import { describe, expect, it } from "vitest";
import { allTags, dayNumber, dueInfo, filterTasks, groupByNode, isOverdue, matchesTab, progress, sortTasks, tabCounts, todayISO } from "./tasks";
import type { Task } from "./types";

const T = "2026-11-02";
const task = (id: string, o: Partial<Task> = {}): Task => ({ id, title: id, done: false, node: null, due: null, priority: "normal", tags: [], notes: "", ...o });

describe("dates", () => {
  it("formats local today", () => {
    expect(todayISO(new Date(2026, 10, 2, 23, 59))).toBe("2026-11-02");
    expect(todayISO(new Date(2027, 0, 5))).toBe("2027-01-05");
  });

  it("parses strictly", () => {
    expect(dayNumber("2026-11-02")! - dayNumber("2026-11-01")!).toBe(1);
    expect(dayNumber("2025-02-30")).toBeNull();
    expect(dayNumber("soon")).toBeNull();
    expect(dayNumber(null)).toBeNull();
  });
});

describe("overdue", () => {
  it("is open and strictly before today", () => {
    expect(isOverdue({ due: "2026-11-01", done: false }, T)).toBe(true);
    expect(isOverdue({ due: "2026-11-02", done: false }, T)).toBe(false);
    expect(isOverdue({ due: "2026-11-01", done: true }, T)).toBe(false);
    expect(isOverdue({ due: null, done: false }, T)).toBe(false);
    expect(isOverdue({ due: "garbage", done: false }, T)).toBe(false);
  });

  it("labels chips, with a bang on late so colour is never the only cue", () => {
    expect(dueInfo({ due: "2026-10-30", done: false }, T)).toEqual({ label: "! 3 d late", tone: "late" });
    expect(dueInfo({ due: T, done: false }, T)).toEqual({ label: "Due today", tone: "today" });
    expect(dueInfo({ due: "2026-11-03", done: false }, T)).toEqual({ label: "Due tomorrow", tone: "soon" });
    expect(dueInfo({ due: "2026-11-09", done: false }, T)).toEqual({ label: "Nov 9", tone: "later" });
    expect(dueInfo({ due: "2027-01-09", done: false }, T).label).toBe("Jan 9, 2027");
    expect(dueInfo({ due: "2026-10-01", done: true }, T).tone).toBe("later");
  });
});

describe("filters", () => {
  const tasks = [
    task("late", { due: "2026-10-30", node: "a", tags: ["qa"] }),
    task("soon", { due: "2026-11-05", node: "a", priority: "high" }),
    task("open"),
    task("done", { done: true, due: "2026-10-01", node: "b" }),
  ];

  it("splits upcoming, overdue and done", () => {
    const ids = (tab: "upcoming" | "overdue" | "done") => filterTasks(tasks, { tab }, T).map((t) => t.id);
    expect(ids("upcoming")).toEqual(["soon", "open"]);
    expect(ids("overdue")).toEqual(["late"]);
    expect(ids("done")).toEqual(["done"]);
    expect(tabCounts(tasks, T)).toEqual({ upcoming: 2, overdue: 1, done: 1 });
    expect(matchesTab(tasks[3], "overdue", T)).toBe(false);
  });

  it("combines query, tag, priority and node", () => {
    expect(filterTasks(tasks, { tab: "upcoming", query: "SO" }, T).map((t) => t.id)).toEqual(["soon"]);
    expect(filterTasks(tasks, { tab: "overdue", tag: "qa" }, T).length).toBe(1);
    expect(filterTasks(tasks, { tab: "upcoming", priority: "high" }, T).map((t) => t.id)).toEqual(["soon"]);
    expect(filterTasks(tasks, { tab: "upcoming", node: "a" }, T).map((t) => t.id)).toEqual(["soon"]);
    expect(filterTasks(tasks, { tab: "upcoming", query: "qa" }, T)).toEqual([]);
    expect(filterTasks(tasks, { tab: "overdue", query: "qa" }, T).length).toBe(1); // tags are searched too
  });
});

describe("progress and grouping", () => {
  it("counts done over total", () => {
    expect(progress([task("a", { done: true }), task("b")])).toEqual({ done: 1, total: 2 });
    expect(progress([])).toEqual({ done: 0, total: 0 });
  });

  it("collects tags once, sorted", () => {
    expect(allTags([task("a", { tags: ["z", "a"] }), task("b", { tags: ["a"] })])).toEqual(["a", "z"]);
  });

  it("groups by node in diagram order with loose tasks last", () => {
    const g = groupByNode([task("1", { node: "b" }), task("2", { node: "a" }), task("3"), task("4", { node: "ghost" })], ["a", "b"]);
    expect(g.map((s) => s.node)).toEqual(["a", "b", null]);
    expect(g[2].tasks.map((t) => t.id)).toEqual(["3", "4"]);
  });

  it("sorts by due date, undated last, then priority", () => {
    const s = sortTasks([task("n"), task("later", { due: "2026-12-01" }), task("hi", { due: "2026-11-05", priority: "high" }), task("lo", { due: "2026-11-05", priority: "low" })]);
    expect(s.map((t) => t.id)).toEqual(["hi", "lo", "later", "n"]);
  });
});
