// A burst of rapid edits (slider drag, colour picker, typing) becomes one undo step:
// the first change begins a gesture, each change is applied live, and the gesture ends after a pause.
import { useRef } from "react";
import type { Diagram } from "@/lib/types";
import { beginGesture, endGesture, live } from "./store";

/** `idleMs` null: the gesture ends only when `end()` is called (a slider drag ends on pointer up). */
export function useBurst(idleMs: number | null = 400) {
  const open = useRef(false);
  const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const end = () => {
    clearTimeout(timer.current);
    if (open.current) endGesture();
    open.current = false;
  };
  const run = (f: (d: Diagram) => Diagram) => {
    if (!open.current) {
      beginGesture();
      open.current = true;
    }
    live(f);
    clearTimeout(timer.current);
    if (idleMs !== null) timer.current = setTimeout(end, idleMs);
  };
  return { run, end };
}
