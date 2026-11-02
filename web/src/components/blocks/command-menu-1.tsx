"use client";

import {
  useCallback,
  useEffect,
  useId,
  useMemo,
  useRef,
  useState,
  type KeyboardEvent,
  type ReactNode,
} from "react";
import { Icon } from "@/ui/icons";

const cx = (...c: (string | false | null | undefined)[]) =>
  c.filter(Boolean).join(" ");

const backdrop = "bg-neutral-900/25 dark:bg-neutral-950/60";
const text1 = "text-neutral-900 dark:text-neutral-100";
const text2 = "text-neutral-900/80 dark:text-neutral-100/80";
const text3 = "text-neutral-900/66 dark:text-neutral-100/66";
const typeBodySm = "text-[13px]/[18px] font-normal";
const transition =
  "transition-[color,background-color,border-color,box-shadow,outline-color,opacity,scale] duration-[120ms] ease-[cubic-bezier(0.2,0,0,1)] active:duration-[60ms]";
const secondaryButton =
  "relative inline-flex shrink-0 cursor-pointer items-center justify-center whitespace-nowrap select-none transition-[color,background-color,border-color,box-shadow,outline-color,scale,filter] duration-[120ms] ease-[cubic-bezier(0.2,0,0,1)] active:scale-[0.98] active:duration-[60ms] motion-reduce:active:scale-100 disabled:cursor-not-allowed aria-busy:cursor-progress focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[var(--rb-accent,oklch(20.5%_0_0))] dark:focus-visible:outline-[var(--rb-accent,oklch(100%_0_0))] border border-neutral-900/12 bg-white text-neutral-900 shadow-[0_1px_2px_-1px_rgb(0_0_0/0.12)] hover:not-disabled:border-neutral-900/32 hover:not-disabled:bg-linear-to-b hover:not-disabled:from-neutral-900/3 hover:not-disabled:to-neutral-900/3 active:shadow-[inset_0_1px_2px_rgb(0_0_0/0.12)] disabled:border-neutral-900/8 disabled:text-neutral-900/40 disabled:shadow-none dark:border-neutral-100/14 dark:bg-neutral-900 dark:text-neutral-100 dark:shadow-[0_1px_2px_-1px_rgb(0_0_0/0.36)] dark:hover:not-disabled:border-neutral-100/32 dark:hover:not-disabled:from-neutral-100/3 dark:hover:not-disabled:to-neutral-100/3 dark:active:shadow-[inset_0_1px_2px_rgb(0_0_0/0.4)] dark:disabled:border-neutral-100/8 dark:disabled:text-neutral-100/40";
const buttonMd =
  "h-8 gap-2 rounded-[var(--rb-r-sm,6px)] px-4 text-[14px]/[16px] font-[450]";
const kbd =
  "inline-flex min-h-5 min-w-5 w-fit items-center justify-center gap-1 rounded-[var(--rb-r-xs,4px)] border border-neutral-900/12 dark:border-neutral-100/14 bg-white dark:bg-neutral-900 px-1 font-mono text-[12px]/[16px] tabular-nums text-neutral-900/80 dark:text-neutral-100/80 select-none";
const itemContent = "flex min-w-0 flex-1 flex-col gap-0.5";
const emptyTitle =
  "text-[16px]/[20px] font-[450] tracking-[-0.015em] text-neutral-900 dark:text-neutral-100";
const emptyDescription =
  "max-w-[48ch] text-[13px]/[18px] text-neutral-900/80 dark:text-neutral-100/80";
const menuLabel =
  "px-2 pt-2 pb-1 text-[12px]/[16px] font-normal tabular-nums text-neutral-900/66 dark:text-neutral-100/66";
const menuItem =
  "relative flex min-h-8 w-full cursor-default items-center gap-2 rounded-[var(--rb-r-sm,6px)] px-2 py-1.5 text-left text-[14px]/[16px] font-[450] text-neutral-900 outline-none select-none [&_svg]:shrink-0 [&_svg]:text-neutral-900/80 hover:bg-neutral-900/5 focus-visible:bg-neutral-900/5 data-[highlighted]:bg-neutral-900/5 aria-selected:bg-neutral-900/9 aria-disabled:cursor-not-allowed aria-disabled:text-neutral-900/40 dark:text-neutral-100 dark:[&_svg]:text-neutral-100/80 dark:hover:bg-neutral-100/5 dark:focus-visible:bg-neutral-100/5 dark:data-[highlighted]:bg-neutral-100/5 dark:aria-selected:bg-neutral-100/9 dark:aria-disabled:text-neutral-100/40";
const dialogShell =
  "flex max-h-[calc(100%-4rem)] flex-col overflow-hidden rounded-[var(--rb-r-lg,10px)] border border-neutral-900/10 dark:border-neutral-100/14 bg-neutral-50 text-neutral-900 dark:text-neutral-100 outline-none shadow-[0_12px_24px_-12px_rgb(0_0_0/0.2)] dark:shadow-[0_12px_24px_-12px_rgb(0_0_0/0.6)] dark:bg-neutral-900";
const dialogCard =
  "mx-1 mt-1 flex min-h-0 flex-1 flex-col overflow-y-auto rounded-[var(--rb-r-sm,6px)] border border-neutral-900/8 bg-white [&:last-child]:mb-1 dark:border-neutral-100/8 dark:bg-neutral-950";
const commandSearch =
  "flex min-h-10 shrink-0 items-center gap-2 border-b border-neutral-900/8 ps-4 pe-2 text-neutral-900/66 dark:text-neutral-100/66 has-[input:focus-visible]:border-transparent has-[input:focus-visible]:shadow-[inset_0_0_0_1px_var(--rb-accent,oklch(20.5%_0_0))] dark:border-neutral-100/8 dark:has-[input:focus-visible]:shadow-[inset_0_0_0_1px_var(--rb-accent,oklch(100%_0_0))]";
const commandInput =
  "h-full min-w-0 flex-1 bg-transparent text-[14px]/[16px] font-[450] text-neutral-900 dark:text-neutral-100 outline-none placeholder:text-neutral-900/66 dark:placeholder:text-neutral-100/66";
const fadeTop =
  "bg-linear-to-b from-white to-transparent transition-opacity duration-[120ms] ease-[cubic-bezier(0.2,0,0,1)] dark:from-neutral-950";
const fadeBottom =
  "bg-linear-to-t from-white to-transparent transition-opacity duration-[120ms] ease-[cubic-bezier(0.2,0,0,1)] dark:from-neutral-950";

const detailText = cx(typeBodySm, text2);
const metaText = cx(typeBodySm, text3);

function useScrollFade<T extends HTMLElement>() {
  const ref = useRef<T>(null);
  const [edges, setEdges] = useState({ start: false, end: false });

  const update = useCallback(() => {
    const el = ref.current;
    if (!el) return;
    const { scrollTop, scrollHeight, clientHeight } = el;
    setEdges({
      start: scrollTop > 1,
      end: Math.ceil(scrollTop + clientHeight) < scrollHeight - 1,
    });
  }, []);

  useEffect(() => {
    update();
    const el = ref.current;
    const view = el?.ownerDocument.defaultView;
    if (!el || !view?.ResizeObserver) return;
    const observer = new view.ResizeObserver(update);
    observer.observe(el);
    return () => observer.disconnect();
  }, [update]);

  return { ref, edges, onScroll: update };
}

export type Command = {
  id: string;
  group: string;
  title: string;
  detail?: string;
  shortcut: string;
  icon: string;
  run: () => void;
};

const EXIT_MS = 140;

const FADE_PX = 32;

const FOCUSABLE =
  'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

function matches(command: Command, query: string) {
  const q = query.trim().toLowerCase();
  if (!q) return true;
  return `${command.title} ${command.detail ?? ""} ${command.group}`
    .toLowerCase()
    .includes(q);
}

function Kbd({ children }: { children: ReactNode }) {
  return <kbd className={kbd}>{children}</kbd>;
}

export default function CommandMenu1({ commands, open: openProp, onClose }: { commands: Command[]; open: boolean; onClose: () => void }) {
  const uid = useId();
  const rootRef = useRef<HTMLElement>(null);
  const dialogRef = useRef<HTMLDivElement>(null);
  const {
    ref: listRef,
    edges: listEdges,
    onScroll: onListScroll,
  } = useScrollFade<HTMLDivElement>();
  const inputRef = useRef<HTMLInputElement>(null);
    const frameRef = useRef<number | undefined>(undefined);
  const timerRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);

  const [open, setOpen] = useState(false);
  const [shown, setShown] = useState(false);
  const [query, setQuery] = useState("");
  const [activeIndex, setActiveIndex] = useState(0);

  const listboxId = `${uid}-listbox`;
  const filtered = useMemo(
    () => commands.filter((command) => matches(command, query)),
    [commands, query],
  );
  const groupOrder = useMemo(() => [...new Set(commands.map((x) => x.group))], [commands]);
  const activeCommand = filtered[activeIndex];
  const activeId = activeCommand ? `${uid}-${activeCommand.id}` : undefined;

  useEffect(() => {
    if (open) inputRef.current?.focus({ preventScroll: true });
  }, [open]);

  useEffect(() => {
    const scroller = listRef.current;
    const row = scroller?.querySelector<HTMLElement>('[data-active="true"]');
    if (!scroller || !row) return;
    const group = row.closest<HTMLElement>('[role="group"]');
    const opensGroup = group?.querySelector('[role="option"]') === row;
    const top = (opensGroup && group ? group : row).getBoundingClientRect().top;
    const listBox = scroller.getBoundingClientRect();
    const rowBox = row.getBoundingClientRect();
    if (top < listBox.top + FADE_PX) {
      const next = scroller.scrollTop - (listBox.top + FADE_PX - top);
      scroller.scrollTop = next < FADE_PX ? 0 : next;
    } else if (rowBox.bottom > listBox.bottom - FADE_PX) {
      const end = scroller.scrollHeight - scroller.clientHeight;
      const next =
        scroller.scrollTop + (rowBox.bottom - listBox.bottom + FADE_PX);
      scroller.scrollTop = next > end - FADE_PX ? end : next;
    }
  }, [activeIndex, query, listRef]);

  useEffect(
    () => () => {
      cancelAnimationFrame(frameRef.current ?? 0);
      clearTimeout(timerRef.current);
    },
    [],
  );

  const openMenu = useCallback((animate: boolean) => {
    clearTimeout(timerRef.current);
    setOpen(true);
    if (animate) {
      setShown(false);
      frameRef.current = requestAnimationFrame(() => setShown(true));
    } else {
      setShown(true);
    }
  }, []);

  const close = useCallback(() => {
    setShown(false);
    clearTimeout(timerRef.current);
    timerRef.current = setTimeout(() => {
      setOpen(false);
      onClose();
    }, EXIT_MS);
  }, [onClose]);

  // Opened from outside (the app owns the shortcut).
  useEffect(() => {
    if (openProp) openMenu(true);
  }, [openProp, openMenu]);

  const run = useCallback((command?: Command) => {
    setQuery("");
    setActiveIndex(0);
    close();
    command?.run();
  }, [close]);

  const move = (delta: number) => {
    setActiveIndex((i) => {
      const count = filtered.length;
      if (count === 0) return 0;
      return (i + delta + count) % count;
    });
  };

  const onInputKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    switch (event.key) {
      case "ArrowDown":
        event.preventDefault();
        move(1);
        break;
      case "ArrowUp":
        event.preventDefault();
        move(-1);
        break;
      case "Home":
        event.preventDefault();
        setActiveIndex(0);
        break;
      case "End":
        event.preventDefault();
        setActiveIndex(Math.max(filtered.length - 1, 0));
        break;
      case "Enter":
        if (!activeCommand) break;
        event.preventDefault();
        run(activeCommand);
        break;
    }
  };

  const onDialogKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    if (event.key === "Escape") {
      event.preventDefault();
      close();
      return;
    }
    if (event.key !== "Tab") return;
    const root = dialogRef.current;
    if (!root) return;
    const nodes = Array.from(root.querySelectorAll<HTMLElement>(FOCUSABLE));
    if (nodes.length === 0) return;
    const first = nodes[0];
    const last = nodes[nodes.length - 1];
    const focused = root.ownerDocument.activeElement;
    if (event.shiftKey && focused === first) {
      event.preventDefault();
      last.focus({ preventScroll: true });
    } else if (!event.shiftKey && focused === last) {
      event.preventDefault();
      first.focus({ preventScroll: true });
    }
  };

  if (!open) return null;

  return (
    <section ref={rootRef} className={cx(text1, "fixed inset-0 z-50")}>
      {open && (
        <>
          <button
            type="button"
            tabIndex={-1}
            aria-label="Close menu"
            onClick={close}
            className={cx(
              backdrop,
              "absolute inset-0 z-40 cursor-default",
              shown ? "opacity-100" : cx(transition, "opacity-0"),
            )}
          />
          <div
            ref={dialogRef}
            role="dialog"
            aria-modal="true"
            aria-labelledby={`${uid}-title`}
            onKeyDown={onDialogKeyDown}
            className={cx(
              dialogShell,
              "absolute left-1/2 top-[12%] z-50 max-h-[min(76%,calc(100%-4rem))] w-[calc(100%-2rem)] max-w-xl -translate-x-1/2",
              shown ? "opacity-100" : cx(transition, "opacity-0"),
            )}
          >
            <h2 id={`${uid}-title`} className="sr-only">
              Search or run a command
            </h2>

            <div className={dialogCard}>
              <div className={cx(commandSearch, "h-12")}>
                <Icon name="search" className="shrink-0" />
                <label htmlFor={`${uid}-input`} className="sr-only">
                  Search or run a command
                </label>
                <input
                  ref={inputRef}
                  id={`${uid}-input`}
                  role="combobox"
                  aria-expanded="true"
                  aria-controls={listboxId}
                  aria-activedescendant={activeId}
                  aria-autocomplete="list"
                  value={query}
                  onChange={(event) => {
                    setQuery(event.target.value);
                    setActiveIndex(0);
                  }}
                  onKeyDown={onInputKeyDown}
                  placeholder="Search or run a command"
                  className={commandInput}
                />
              </div>

              <div className="relative flex min-h-0 flex-1 flex-col">
                <div
                  ref={listRef}
                  onScroll={onListScroll}
                  className="min-h-0 flex-1 overflow-y-auto p-2"
                >
                  {filtered.length > 0 ? (
                    <div id={listboxId} role="listbox" aria-label="Commands">
                      {groupOrder.map((group) => {
                        const rows = filtered.filter((c) => c.group === group);
                        if (rows.length === 0) return null;
                        const groupId = `${uid}-${group}`;
                        return (
                          <div
                            key={group}
                            role="group"
                            aria-labelledby={groupId}
                            className="mb-1 last:mb-0"
                          >
                            <p id={groupId} className={menuLabel}>
                              {group}
                            </p>
                            {rows.map((command) => {
                              const index = filtered.indexOf(command);
                              const selected = index === activeIndex;
                              return (
                                <button
                                  key={command.id}
                                  id={`${uid}-${command.id}`}
                                  role="option"
                                  aria-selected={selected}
                                  data-active={selected}
                                  data-highlighted={selected || undefined}
                                  type="button"
                                  tabIndex={-1}
                                  onPointerMove={() => setActiveIndex(index)}
                                  onMouseDown={(event) => {
                                    event.preventDefault();
                                    setActiveIndex(index);
                                    run(command);
                                  }}
                                  className={cx(menuItem, "min-h-11")}
                                >
                                  <Icon name={command.icon} />
                                  <span className={itemContent}>
                                    <span className="truncate">
                                      {command.title}
                                    </span>
                                    {command.detail && (
                                      <span
                                        className={cx(detailText, "truncate")}
                                      >
                                        {command.detail}
                                      </span>
                                    )}
                                  </span>
                                  {command.shortcut && <Kbd>{command.shortcut}</Kbd>}
                                </button>
                              );
                            })}
                          </div>
                        );
                      })}
                    </div>
                  ) : (
                    <div className="flex flex-col items-center px-2.5 py-10 text-center">
                      <p className={emptyTitle}>
                        No command matches “{query.trim()}”
                      </p>
                      <p
                        className={cx(
                          emptyDescription,
                          "mt-1 max-w-xs text-balance",
                        )}
                      >
                        Check the spelling, or clear the search to see
                        everything.
                      </p>
                      <button
                        type="button"
                        onMouseDown={(event) => {
                          event.preventDefault();
                          setQuery("");
                          setActiveIndex(0);
                          inputRef.current?.focus({ preventScroll: true });
                        }}
                        className={cx(secondaryButton, buttonMd, "mt-4")}
                      >
                        Clear search
                      </button>
                    </div>
                  )}
                </div>
                <div
                  aria-hidden="true"
                  className={cx(
                    fadeTop,
                    "pointer-events-none absolute inset-x-0 top-0 h-8",
                    listEdges.start ? "opacity-100" : "opacity-0",
                  )}
                />
                <div
                  aria-hidden="true"
                  className={cx(
                    fadeBottom,
                    "pointer-events-none absolute inset-x-0 bottom-0 h-8",
                    listEdges.end ? "opacity-100" : "opacity-0",
                  )}
                />
              </div>
            </div>

            <div
              className={cx(
                metaText,
                "flex h-10 shrink-0 items-center gap-4 px-5",
              )}
            >
              <span className="hidden items-center gap-1.5 sm:inline-flex">
                <Kbd>↑</Kbd>
                <Kbd>↓</Kbd>
                to navigate
              </span>
              <span className="inline-flex items-center gap-1.5">
                <Kbd>
                  ↵
                </Kbd>
                to open
              </span>
              <span className="inline-flex items-center gap-1.5">
                <Kbd>esc</Kbd>
                to close
              </span>
              <span className="ml-auto shrink-0 tabular-nums">
                {filtered.length} {filtered.length === 1 ? "result" : "results"}
              </span>
            </div>
          </div>
        </>
      )}
    </section>
  );
}
