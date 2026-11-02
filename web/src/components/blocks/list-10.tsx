// Style kit from React Bits Pro "list-10" (runbook checklist with progress), used as shipped by the
// Tasks tab: the checkbox, its tick, and the progress bar. The block's demo content was removed.
export const checkbox =
  "peer size-4 shrink-0 cursor-pointer appearance-none rounded-[var(--rb-r-xs,4px)] border border-neutral-900/50 bg-white shadow-[0_1px_2px_-1px_rgb(0_0_0/0.12)] transition-[background-color,border-color,box-shadow,outline-color] duration-[120ms] ease-[cubic-bezier(0.2,0,0,1)] hover:not-checked:not-disabled:border-neutral-900 active:not-checked:bg-neutral-900/8 checked:border-transparent checked:bg-[var(--rb-accent,oklch(20.5%_0_0))] indeterminate:border-transparent indeterminate:bg-[var(--rb-accent,oklch(20.5%_0_0))] aria-invalid:border-[oklch(60%_0.236_26)] disabled:cursor-not-allowed disabled:border-neutral-900/40 disabled:shadow-none disabled:checked:bg-neutral-900/40 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[var(--rb-accent,oklch(20.5%_0_0))] dark:focus-visible:outline-[var(--rb-accent,oklch(100%_0_0))] dark:border-neutral-100/50 dark:bg-neutral-900 dark:shadow-[0_1px_2px_-1px_rgb(0_0_0/0.36)] dark:hover:not-checked:not-disabled:border-neutral-100 dark:active:not-checked:bg-neutral-100/8 dark:checked:bg-[var(--rb-accent,oklch(100%_0_0))] dark:indeterminate:bg-[var(--rb-accent,oklch(100%_0_0))] dark:aria-invalid:border-[oklch(65.2%_0.219_26)] dark:disabled:border-neutral-100/40 dark:disabled:checked:bg-neutral-100/40";
export const checkboxBox =
  "relative inline-flex size-4 shrink-0 items-center justify-center";
export const checkboxMark =
  "pointer-events-none absolute text-[var(--rb-accent-fg,oklch(100%_0_0))] opacity-0 peer-checked:opacity-100 peer-indeterminate:opacity-100 dark:text-[var(--rb-accent-fg,oklch(20.5%_0_0))]";
export const progressTrack =
  "relative block h-1 w-full overflow-hidden bg-neutral-900/7 dark:bg-neutral-100/7";
export const progressFill =
  "block h-full bg-neutral-900 transition-[width] duration-[120ms] ease-[cubic-bezier(0.2,0,0,1)] motion-reduce:transition-none dark:bg-neutral-100";
export const frame =
  "relative flex min-w-0 flex-col overflow-hidden rounded-[var(--rb-r-lg,10px)] border border-neutral-900/10 dark:border-neutral-100/14 bg-white dark:bg-neutral-900 text-neutral-900 dark:text-neutral-100";
export const rowsRuled = "divide-y divide-neutral-900/8 dark:divide-neutral-100/8";
export const typeLabel = "text-[14px]/[16px] font-[450]";
export const typeBodySm = "text-[13px]/[18px] font-normal";
export const text1 = "text-neutral-900 dark:text-neutral-100";
export const text2 = "text-neutral-900/80 dark:text-neutral-100/80";
export const text3 = "text-neutral-900/66 dark:text-neutral-100/66";
export const hoverFill = "hover:bg-neutral-900/5 dark:hover:bg-neutral-100/5";
export const cornerSm = "rounded-[var(--rb-r-sm,6px)]";
