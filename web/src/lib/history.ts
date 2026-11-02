// Snapshot undo/redo. One user gesture = one commit of the state before it.
export interface History<T> {
  past: T[];
  future: T[];
}

export const LIMIT = 200;

export const emptyHistory = <T>(): History<T> => ({ past: [], future: [] });

/** Record `before` (state prior to an edit). Clears redo. */
export function commit<T>(h: History<T>, before: T): History<T> {
  const past = [...h.past, before];
  if (past.length > LIMIT) past.shift();
  return { past, future: [] };
}

export const canUndo = <T>(h: History<T>) => h.past.length > 0;
export const canRedo = <T>(h: History<T>) => h.future.length > 0;

/** Step back from `current`; null when there is nothing to undo. */
export function undo<T>(h: History<T>, current: T): { history: History<T>; value: T } | null {
  if (!h.past.length) return null;
  const value = h.past[h.past.length - 1];
  return { value, history: { past: h.past.slice(0, -1), future: [...h.future, current] } };
}

export function redo<T>(h: History<T>, current: T): { history: History<T>; value: T } | null {
  if (!h.future.length) return null;
  const value = h.future[h.future.length - 1];
  return { value, history: { past: [...h.past, current], future: h.future.slice(0, -1) } };
}
