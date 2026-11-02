# React Bits in Rovelo

The UI uses React Bits **Pro** blocks where the catalogue has a fit, the **free** React Bits components for a
few flourishes, and shadcn/ui for everything else.

## Registries

`components.json` declares three registries. The licence key is only ever read from the
`REACTBITS_LICENSE_KEY` environment variable at install time and is never written to a file.

| Name | URL | Auth |
|---|---|---|
| `@react-bits` | `https://reactbits.dev/r/{name}.json` | none |
| `@reactbits-pro` | `https://pro.reactbits.dev/api/r/pro/{name}.json` | `Authorization: Bearer ${REACTBITS_LICENSE_KEY}` |
| `@reactbits-starter` | `https://pro.reactbits.dev/api/r/starter/{name}.json` | `Authorization: Bearer ${REACTBITS_LICENSE_KEY}` |

```bash
export REACTBITS_LICENSE_KEY=...          # your shell only
pnpm dlx shadcn@latest add @reactbits-pro/<name>
```

## Pro blocks in use

Installed with `shadcn add @reactbits-pro/<name>` into `src/components/blocks/`. Their markup, classes and states are
as shipped. Edits are limited to what any block needs to be used in an app: its mock data became props, its demo
wrapper was removed, and Hugeicons were swapped for the Nucleo set (the app allows no other icon library).
`@reactbits-pro/app-ui-theme` supplied the `--rb-*` variables; `--rb-accent` and `--rb-accent-fg` point at the
shadcn `--primary` and `--primary-foreground` (fuchsia-600, white), so Pro primary buttons, focus outlines and
checked boxes follow the theme.

| Where | Pro block | File | What was wired |
|---|---|---|---|
| "Start a flow" empty state | `empty-state-1` (first-run workspace with starting paths) | `src/components/blocks/empty-state-1.tsx`, used in `src/ui/Panels.tsx` | The three starting paths are the User flow, Task flow and Blank templates; the primary button adds a step, the secondary opens Paste Mermaid. The block's skeleton-table decoration was dropped. |
| Command palette, `Cmd/Ctrl+K` (new) | `command-menu-1` (command palette with grouped results) | `src/components/blocks/command-menu-1.tsx`, wired in `src/ui/CommandPalette.tsx` | Tools, diagram actions, panels, file actions and "Go to step" for every node, all calling the same actions as the buttons. |
| Task checkboxes and progress bar | `list-10` (runbook checklist with progress) | style kit in `src/components/blocks/list-10.tsx`, used in `src/ui/TasksTab.tsx` | The checkbox with its tick and the progress track and fill. The block's runbook demo was removed. |

## Pro catalogue, searched and not used

The Pro catalogue is made of full blocks (619 items in `@reactbits-pro`), not standalone primitives, so there is no
Pro segmented control, tooltip, dropdown menu, toolbar or colour picker to swap in. What was looked at:

| Wanted | Found | Decision |
|---|---|---|
| Toolbar | `navbar-11` (editor toolbar with presence avatars), `card-8` (selectable tiles with a toolbar) | Page-level bars, not a floating icon pill; kept shadcn buttons in the pill. |
| Segmented control, tabs | none standalone (tabs appear inside blocks such as `tool-calls-5`, `features-4`) | Kept shadcn `ToggleGroup`. |
| Dropdown, tooltip, popover | only inside nav and chart blocks | Kept shadcn (Radix) menus and tooltips. |
| Colour picker, swatches | `filtering-1` (faceted filters with swatches) | Product filters, not a palette editor; kept the Swatch control. |
| Inspector, properties panel | none | Kept the shadcn-based inspector. |
| Task list | `list-10`, `onboarding-3`, `agent-plan-1`, `kanban-4` | Used the `list-10` checkbox and progress; the list keeps its grouping, tabs and filters. |
| Detail sheet | `app-dialog-6` (detail sheet with metadata) | Not needed; the task detail is a pane in the left panel. |
| Settings form | `settings-form-3` (switches and radios) | The Brand tab already uses shadcn switch, slider and select. |

## Free React Bits components in use

Each is wrapped in one slot in `src/ui/slots.tsx`.

| Slot | Where | Component | File |
|---|---|---|---|
| `SlotNumber` | task progress count | `CountUp` | `src/components/CountUp.tsx` |
| `SlotReveal` | left panel and inspector appearing (200ms fade) | `FadeContent` | `src/components/FadeContent.tsx` |
| `SlotHeadline`, `SlotCard` | no longer used (replaced by the Pro empty state); kept for reference | `BlurText`, `SpotlightCard` | `src/components/` |

## Swapping one piece

Another Pro block: `pnpm dlx shadcn@latest add @reactbits-pro/<name>` (with the key in your shell), then wire it in the
file named in the tables above. Replace any Hugeicons import with `Icon` from `src/ui/icons.tsx`.

## Slider

Every slider is `src/components/InsetSlider.tsx` (WoopKIT's Inset look). The shadcn slider was removed; no React Bits slider is used.
