import * as React from "react"
import { cn } from "cn"

function Input({ className, type, ...props }: React.ComponentProps<"input">) {
  return (
    <input
      type={type}
      data-slot="input"
      className={cn(
        "h-8 w-full min-w-0 rounded-lg border border-hair bg-sunk px-3 py-1 text-sm outline-none transition-[border-color,box-shadow] duration-150 placeholder:text-ink-muted hover:border-hair-strong focus-visible:border-brand focus-visible:ring-[3px] focus-visible:ring-brand-soft focus-visible:outline-none disabled:pointer-events-none disabled:opacity-50",
        className
      )}
      {...props}
    />
  )
}

export { Input }
