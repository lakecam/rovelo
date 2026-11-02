// Tasks tab: progress, Upcoming / Overdue / Completed, filters, rows grouped by linked step,
// inline "Add a task", and a detail pane for the selected task.
import { useMemo, useRef, useState } from "react";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { allTags, dueInfo, filterTasks, groupByNode, progress, sortTasks, tabCounts, todayISO, type Tab } from "@/lib/tasks";
import { mapTask, removeTask } from "@/lib/ops";
import type { Id, Priority, Task } from "@/lib/types";
import * as A from "@/state/actions";
import { edit, setState, useStore } from "@/state/store";
import { Chip, Segmented, SectionLabel } from "./kit";
import { Icon } from "./icons";
import { SlotNumber } from "./slots";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { checkbox, checkboxBox, checkboxMark, progressFill, progressTrack } from "@/components/blocks/list-10";

const today = () => todayISO();

export function TasksTab() {
  const taskSel = useStore((s) => s.taskSel);
  const tasks = useStore((s) => s.diagram.tasks);
  const sel = tasks.find((t) => t.id === taskSel);
  return sel ? <TaskDetail task={sel} /> : <TaskList />;
}

/** React Bits Pro checkbox (list-10), default states. */
function CheckCircle({ done, onChange, label }: { done: boolean; onChange: () => void; label: string }) {
  return (
    <span className={checkboxBox} onClick={(e) => e.stopPropagation()}>
      <input type="checkbox" checked={done} aria-label={label} onChange={onChange} className={checkbox} />
      <Icon name="check" size={12} className={checkboxMark} />
    </span>
  );
}

function TaskList() {
  const diagram = useStore((s) => s.diagram);
  const selection = useStore((s) => s.selection);
  const [tab, setTab] = useState<Tab>("upcoming");
  const [query, setQuery] = useState("");
  const [tag, setTag] = useState<string | null>(null);
  const [onlySel, setOnlySel] = useState(false);
  const t0 = today();
  const { done, total } = progress(diagram.tasks);
  const counts = tabCounts(diagram.tasks, t0);
  const tags = useMemo(() => allTags(diagram.tasks), [diagram.tasks]);
  const selNode = selection.nodes.length === 1 ? selection.nodes[0] : null;
  const filtered = sortTasks(filterTasks(diagram.tasks, { tab, query, tag, node: onlySel ? selNode : null }, t0));
  const sections = groupByNode(filtered, diagram.nodes.map((n) => n.id));
  const label = (id: Id | null) => (id ? diagram.nodes.find((n) => n.id === id)?.label.replace(/\n/g, " ") ?? "Unlinked" : "No step");
  // Sections that have rows, plus an always-available "No step" section to add loose tasks.
  const addLoose = tab === "upcoming" && !query && !tag && !onlySel;
  const shown = [...sections.map((s) => s.node), ...(addLoose && !sections.some((s) => s.node === null) ? [null] : [])];
  const rows = (node: Id | null) => sections.find((s) => s.node === node)?.tasks ?? [];

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-4">
      <div className="shrink-0 px-4">
        <div className="flex items-baseline justify-between">
          <div className="text-base font-medium"><SlotNumber value={done} /> <span className="text-ink-muted">of {total} done</span></div>
          <span className="text-xs tabular-nums text-ink-muted">{total ? Math.round((done / total) * 100) : 0}%</span>
        </div>
        <div className={cn(progressTrack, "mt-2 h-1.5")} role="progressbar" aria-valuenow={done} aria-valuemin={0} aria-valuemax={total} aria-label="Tasks done">
          <div className={progressFill} style={{ width: `${total ? (done / total) * 100 : 0}%` }} />
        </div>
      </div>
      <div className="shrink-0 px-4">
        <Segmented<Tab> label="Task filter" value={tab} onChange={setTab} options={[
          { value: "upcoming", label: <>Upcoming <span className="tabular-nums opacity-60">{counts.upcoming}</span></> },
          { value: "overdue", label: <>Overdue <span className={cn("tabular-nums", counts.overdue ? "text-bad" : "opacity-60")}>{counts.overdue}</span></> },
          { value: "done", label: <>Done <span className="tabular-nums opacity-60">{counts.done}</span></> },
        ]} />
      </div>
      <div className="flex shrink-0 flex-col gap-2 px-4">
        <div className="relative">
          <Icon name="search" size={14} className="pointer-events-none absolute left-2 top-2 text-ink-muted" />
          <Input value={query} onChange={(e) => setQuery(e.target.value)} placeholder="Search tasks" aria-label="Search tasks" className="pl-7" />
        </div>
        {(tags.length > 0 || selNode) && (
          <div className="flex flex-wrap gap-1">
            {selNode && (
              <button aria-pressed={onlySel} onClick={() => setOnlySel(!onlySel)}>
                <Chip tone={onlySel ? "accent" : "plain"}><Icon name="link" size={14} />Selected step</Chip>
              </button>
            )}
            {tags.map((t) => (
              <button key={t} aria-pressed={tag === t} onClick={() => setTag(tag === t ? null : t)}>
                <Chip tone={tag === t ? "accent" : "plain"}><Icon name="tag" size={14} />{t}</Chip>
              </button>
            ))}
          </div>
        )}
      </div>
      <div className="min-h-0 flex-1 overflow-y-auto p-2">
        {shown.length === 0 && (
          <p className="px-2 py-6 text-center text-xs text-ink-muted">
            {tab === "overdue" ? "Nothing is overdue." : tab === "done" ? "Completed tasks show up here." : "No matching tasks."}
          </p>
        )}
        {shown.map((node) => (
          <section key={node ?? "loose"} className="mb-2">
            <div className="flex items-center gap-2 px-2 pb-1 pt-2">
              {node && <button className="truncate text-xs font-medium text-ink-muted hover:text-ink" onClick={() => A.focusNode(node)} title="Show on canvas">{label(node)}</button>}
              {!node && <SectionLabel>No step</SectionLabel>}
              <span className="text-xs tabular-nums text-ink-muted/70">{rows(node).length || ""}</span>
            </div>
            {rows(node).map((t) => <TaskRow key={t.id} task={t} />)}
            {tab !== "done" && <AddRow node={node} />}
          </section>
        ))}
      </div>
    </div>
  );
}

function TaskRow({ task }: { task: Task }) {
  const nodes = useStore((s) => s.diagram.nodes);
  const node = nodes.find((n) => n.id === task.node);
  const info = task.due ? dueInfo(task, today()) : null;
  const open = () => setState({ taskSel: task.id });
  return (
    <div
      role="button"
      tabIndex={0}
      onClick={open}
      onKeyDown={(e) => { if (e.key === "Enter") open(); }}
      className="flex min-h-9 cursor-default items-start gap-3 rounded-lg p-2 transition-[background-color] duration-150 hover:bg-accent"
    >
      <span className="flex h-5 items-center">
        <CheckCircle done={task.done} label={`Mark "${task.title}" ${task.done ? "not done" : "done"}`} onChange={() => edit((d) => mapTask(d, task.id, (t) => ({ ...t, done: !t.done })))} />
      </span>
      <div className="min-w-0 flex-1">
        <div className={cn("text-sm leading-5", task.done && "text-ink-muted line-through")}>{task.title}</div>
        {(info || node || task.priority === "high" || task.tags.length > 0) && (
          <div className="mt-1 flex flex-wrap gap-1">
            {info && <Chip tone={info.tone === "late" ? "late" : info.tone === "today" ? "today" : "plain"}>{info.label}</Chip>}
            {task.priority === "high" && !task.done && <Chip><Icon name="flag" size={14} />High</Chip>}
            {task.tags.map((t) => <Chip key={t}><Icon name="tag" size={14} />{t}</Chip>)}
            {node && (
              <button aria-label={`Show ${node.label.replace(/\n/g, " ")} on canvas`} title="Show on canvas" onClick={(e) => { e.stopPropagation(); A.focusNode(node.id); }}>
                <Chip className="max-w-40 hover:border-hair-strong hover:text-ink"><Icon name="link" size={14} /><span className="truncate">{node.label.replace(/\n/g, " ")}</span></Chip>
              </button>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

function AddRow({ node }: { node: Id | null }) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLInputElement>(null);
  if (!open)
    return (
      <button className="flex h-8 w-full items-center gap-3 rounded-lg px-2 text-sm text-ink-muted transition-[background-color,color] duration-150 hover:bg-accent hover:text-ink" onClick={() => { setOpen(true); setTimeout(() => ref.current?.focus(), 0); }}>
        <span className="grid size-4 place-items-center"><Icon name="plus" size={14} /></span>Add a task
      </button>
    );
  return (
    <div className="flex h-8 items-center gap-3 px-2">
      <span className="size-4 shrink-0 rounded border border-dashed border-hair-strong" />
      <input
        ref={ref}
        autoFocus
        aria-label="New task title"
        placeholder="Task title, then Enter"
        className="h-8 min-w-0 flex-1 rounded-lg bg-transparent px-1 text-sm outline-none placeholder:text-ink-muted"
        onBlur={(e) => { if (!e.currentTarget.value.trim()) setOpen(false); }}
        onKeyDown={(e) => {
          if (e.key === "Escape") setOpen(false);
          if (e.key === "Enter") {
            if (A.addTask(e.currentTarget.value, node)) e.currentTarget.value = "";
          }
        }}
      />
    </div>
  );
}

// ---- detail pane ---------------------------------------------------------------------------
function TaskDetail({ task }: { task: Task }) {
  const diagram = useStore((s) => s.diagram);
  const patch = (p: Partial<Task>) => edit((d) => mapTask(d, task.id, (t) => ({ ...t, ...p })));
  const siblings = task.node ? diagram.tasks.filter((t) => t.node === task.node) : [];
  const sibDone = siblings.filter((t) => t.done).length;
  return (
    <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto p-4 pt-0" key={task.id}>
      <button className="flex h-8 items-center gap-1 self-start text-sm text-ink-muted hover:text-ink" onClick={() => setState({ taskSel: null })}>
        <Icon name="chevron-right" size={14} style={{ transform: "rotate(180deg)" }} />All tasks
      </button>
      <div className="flex items-start gap-2">
        <div className="pt-2"><CheckCircle done={task.done} label="Done" onChange={() => patch({ done: !task.done })} /></div>
        <Textarea
          aria-label="Task title"
          defaultValue={task.title}
          rows={2}
          className={cn("min-h-0 resize-none border-transparent bg-transparent px-2 text-base font-semibold shadow-none", task.done && "text-ink-muted line-through")}
          onBlur={(e) => e.currentTarget.value.trim() && e.currentTarget.value !== task.title && patch({ title: e.currentTarget.value.trim() })}
          onKeyDown={(e) => { if (e.key === "Enter") { e.preventDefault(); e.currentTarget.blur(); } }}
        />
      </div>
      <Field label="Linked step">
        <div className="flex gap-2">
          <Select value={task.node ?? "none"} onValueChange={(v) => patch({ node: v === "none" ? null : v })}>
            <SelectTrigger size="sm" className="min-w-0 flex-1" aria-label="Linked step"><SelectValue /></SelectTrigger>
            <SelectContent>
              <SelectItem value="none">No step</SelectItem>
              {diagram.nodes.map((n) => <SelectItem key={n.id} value={n.id}>{n.label.replace(/\n/g, " ")}</SelectItem>)}
            </SelectContent>
          </Select>
          {task.node && <Button variant="secondary" size="sm" onClick={() => A.focusNode(task.node!)}>Show</Button>}
        </div>
      </Field>
      <Field label="Due date">
        <div className="flex gap-2">
          <Input type="date" aria-label="Due date" value={task.due ?? ""} onChange={(e) => patch({ due: e.target.value || null })} />
          {task.due && <Button variant="ghost" size="sm" onClick={() => patch({ due: null })}>Clear</Button>}
        </div>
      </Field>
      <Field label="Priority">
        <Segmented<Priority> label="Priority" value={task.priority} onChange={(p) => patch({ priority: p })} options={[
          { value: "low", label: "Low" }, { value: "normal", label: "Normal" }, { value: "high", label: "High" },
        ]} />
      </Field>
      <Field label="Tags">
        <Input
          aria-label="Tags"
          placeholder="design, copy"
          defaultValue={task.tags.join(", ")}
          onBlur={(e) => {
            const tags = e.currentTarget.value.split(/[,\s]+/).map((x) => x.trim()).filter(Boolean);
            if (tags.join() !== task.tags.join()) patch({ tags });
          }}
          onKeyDown={(e) => e.key === "Enter" && e.currentTarget.blur()}
        />
      </Field>
      <Field label="Notes">
        <Textarea aria-label="Notes" defaultValue={task.notes} rows={3} className="min-h-0 resize-none" onBlur={(e) => e.currentTarget.value !== task.notes && patch({ notes: e.currentTarget.value })} />
      </Field>
      {task.node && (
        <Field label={`Tasks on this step  ${sibDone}/${siblings.length}`}>
          <ul className="flex flex-col">
            {siblings.map((t) => (
              <li key={t.id} className="flex h-8 items-center gap-3">
                <CheckCircle done={t.done} label={t.title} onChange={() => edit((d) => mapTask(d, t.id, (x) => ({ ...x, done: !x.done })))} />
                <button className={cn("min-w-0 flex-1 truncate text-left text-sm", t.done && "text-ink-muted line-through", t.id === task.id && "font-medium")} onClick={() => setState({ taskSel: t.id })}>{t.title}</button>
              </li>
            ))}
          </ul>
        </Field>
      )}
      <Button variant="ghost" size="sm" className="self-start text-bad hover:bg-bad-soft" onClick={() => { edit((d) => removeTask(d, task.id)); setState({ taskSel: null }); }}>
        <Icon name="trash" />Delete task
      </Button>
    </div>
  );
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex flex-col gap-2">
      <SectionLabel>{label}</SectionLabel>
      {children}
    </div>
  );
}
