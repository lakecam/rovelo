// Task helpers: dates, overdue, filters, progress. `today` is always passed in so tests are stable.
import type { Id, Priority, Task } from "./types";

export type Tab = "upcoming" | "overdue" | "done";

export function todayISO(now = new Date()): string {
  const m = String(now.getMonth() + 1).padStart(2, "0");
  const d = String(now.getDate()).padStart(2, "0");
  return `${now.getFullYear()}-${m}-${d}`;
}

/** Days since 1970-01-01 for a strict ISO date, or null. */
export function dayNumber(iso: string | null | undefined): number | null {
  if (!iso || !/^\d{4}-\d{2}-\d{2}$/.test(iso)) return null;
  const [y, m, d] = iso.split("-").map(Number);
  const t = Date.UTC(y, m - 1, d);
  const back = new Date(t);
  if (back.getUTCFullYear() !== y || back.getUTCMonth() !== m - 1 || back.getUTCDate() !== d) return null;
  return Math.round(t / 86_400_000);
}

export const isOverdue = (t: Pick<Task, "due" | "done">, today: string) => {
  const due = dayNumber(t.due);
  const now = dayNumber(today);
  return !t.done && due !== null && now !== null && due < now;
};

export type DueTone = "late" | "today" | "soon" | "later" | "none";

export interface DueInfo { label: string; tone: DueTone }

const MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/** Chip text and tone. Overdue gets a "!" so colour is never the only signal. */
export function dueInfo(t: Pick<Task, "due" | "done">, today: string): DueInfo {
  const due = dayNumber(t.due);
  const now = dayNumber(today);
  if (due === null || now === null) return { label: t.due ?? "", tone: "none" };
  const diff = due - now;
  if (t.done) return { label: shortDate(t.due!, today), tone: "later" };
  if (diff < 0) return { label: `! ${-diff} d late`, tone: "late" };
  if (diff === 0) return { label: "Due today", tone: "today" };
  if (diff === 1) return { label: "Due tomorrow", tone: "soon" };
  return { label: shortDate(t.due!, today), tone: "later" };
}

export function shortDate(iso: string, today: string): string {
  const [y, m, d] = iso.split("-").map(Number);
  const [ty] = today.split("-").map(Number);
  return y === ty ? `${MONTHS[m - 1]} ${d}` : `${MONTHS[m - 1]} ${d}, ${y}`;
}

export interface TaskFilter {
  tab: Tab;
  query?: string;
  tag?: string | null;
  priority?: Priority | null;
  node?: Id | null;
}

/** Tabs: upcoming = open and not overdue, overdue = open and late, done = completed. */
export function matchesTab(t: Task, tab: Tab, today: string): boolean {
  if (tab === "done") return t.done;
  if (t.done) return false;
  return tab === "overdue" ? isOverdue(t, today) : !isOverdue(t, today);
}

export function filterTasks(tasks: Task[], f: TaskFilter, today: string): Task[] {
  const q = f.query?.trim().toLowerCase() ?? "";
  return tasks.filter(
    (t) =>
      matchesTab(t, f.tab, today) &&
      (!q || t.title.toLowerCase().includes(q) || t.tags.some((g) => g.toLowerCase().includes(q))) &&
      (!f.tag || t.tags.includes(f.tag)) &&
      (!f.priority || t.priority === f.priority) &&
      (!f.node || t.node === f.node),
  );
}

export function tabCounts(tasks: Task[], today: string): Record<Tab, number> {
  return {
    upcoming: tasks.filter((t) => matchesTab(t, "upcoming", today)).length,
    overdue: tasks.filter((t) => matchesTab(t, "overdue", today)).length,
    done: tasks.filter((t) => t.done).length,
  };
}

export const progress = (tasks: Task[]) => ({ done: tasks.filter((t) => t.done).length, total: tasks.length });

export const allTags = (tasks: Task[]) => [...new Set(tasks.flatMap((t) => t.tags))].sort();

/** Group by linked node, in the diagram's node order; unlinked tasks come last. */
export function groupByNode(tasks: Task[], nodeOrder: Id[]): { node: Id | null; tasks: Task[] }[] {
  const out: { node: Id | null; tasks: Task[] }[] = [];
  for (const id of nodeOrder) {
    const ts = tasks.filter((t) => t.node === id);
    if (ts.length) out.push({ node: id, tasks: ts });
  }
  const loose = tasks.filter((t) => !t.node || !nodeOrder.includes(t.node));
  if (loose.length) out.push({ node: null, tasks: loose });
  return out;
}

const PRIORITY_RANK: Record<Priority, number> = { high: 0, normal: 1, low: 2 };

/** Earliest due first, undated last, then higher priority. */
export function sortTasks(tasks: Task[]): Task[] {
  return [...tasks].sort((a, b) => {
    const da = dayNumber(a.due) ?? Infinity;
    const db = dayNumber(b.due) ?? Infinity;
    return da - db || PRIORITY_RANK[a.priority] - PRIORITY_RANK[b.priority];
  });
}
