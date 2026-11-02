# Design system: Rovelo

Mode: Operate. Dark by default; light is a choice in the theme toggle and Settings, and the choice is remembered. The app never follows the system appearance on its own. Cool zinc paper, hairline borders, one fuchsia accent, floating panels over a dot-grid canvas.

## Ramps (Tailwind)

Brand: **fuchsia**. Neutral: **zinc**. Every chrome colour is a step of one of these two ramps, or of Tailwind green, amber and red for status.

| Step | 50 | 100 | 200 | 300 | 400 | 500 | 600 | 700 | 800 | 900 | 950 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| fuchsia | #fdf4ff | #fae8ff | #f5d0fe | #f0abfc | #e879f9 | #d946ef | #c026d3 | #a21caf | #86198f | #701a75 | #4a044e |
| zinc | #fafafa | #f4f4f5 | #e4e4e7 | #d4d4d8 | #a1a1aa | #71717a | #52525b | #3f3f46 | #27272a | #18181b | #09090b |

## Colour (chrome only; the user's brand kit colours the diagram)

| Token | Light | Dark | Use |
|---|---|---|---|
| paper | zinc-50 | zinc-950 | window and canvas background |
| surface | white | zinc-900 | floating panels, cards |
| surface-sunk | zinc-100 | zinc-800 | segmented controls, inputs at rest |
| border | zinc-200 | zinc-800 | all hairlines |
| border-strong | zinc-300 | zinc-700 | input hover, dividers that must read |
| dot | zinc-300 | zinc-800 | canvas grid dots |
| text | zinc-900 | zinc-50 | primary text |
| text-muted | zinc-500 | zinc-400 | labels, captions, icons at rest |
| accent | fuchsia-600 | fuchsia-600 | primary button fill, selection outline and handles, focus ring |
| accent-soft | fuchsia-50 | fuchsia-950 | focus-ring halo on inputs only |
| accent-text | fuchsia-700 | fuchsia-300 | accent used as text or icon on paper or surface |
| success | green-600 #16a34a | green-400 #4ade80 | not used for checked or active state; reserved for diagram roles |
| warning | amber-600 #d97706 | amber-400 #fbbf24 | due today |
| danger | red-600 #dc2626 | red-400 #f87171 | overdue, errors |

Contrast checked: white on fuchsia-600 4.7:1 (so the accent fill stays fuchsia-600 in dark mode too; white on fuchsia-500 is only 3.5:1), fuchsia-700 on fuchsia-50 5.9:1, zinc-500 on zinc-50 4.6:1, zinc-400 on zinc-900 6.9:1, fuchsia-300 on fuchsia-950 8.4:1.

**States follow the shadcn defaults that React Bits Pro builds on, checked against dark editors on Mobbin (Rive, Krea, Webflow, Zoho Canvas, Magnific).** Fuchsia is for doing and checking, never for "where am I".
- **Selected and active are neutral.** Selected tab, segment, filter, active tool, selected task row, current list item, selected preset or template card: a lighter zinc surface with full-strength text and icon. Dark: `zinc-800` fill, `zinc-50` text and icon. Light: segments and tabs are a raised `white` chip with the hairline shadow on the `zinc-100` track; tools and rows get a `zinc-100` fill with `zinc-900` text. Unselected items are `text-muted` with no fill.
- **Checked and on are fuchsia.** Checkbox, switch, radio: solid `accent` (fuchsia-600) with a white mark, as shadcn's `data-[state=checked]:bg-primary`.
- **Fuchsia also marks:** the primary button, the focus ring (2px), the canvas selection outline and resize handles, the progress bar fill, the marquee, and text links.
- **Selected swatch:** a 2px ring in `text` (zinc-50 dark, zinc-900 light) with a 2px gap, as Figma and shadcn colour pickers do; never a fuchsia ring around a colour.
- Hover is always neutral: `zinc-800` dark at 60% / `zinc-100` light.

Rejected 2026-11-02: fuchsia-tinted selected states (fuchsia-950 fill with fuchsia-300 text on every tab, segment, row and tool). The user found the all-pink selection jarring; modern editors keep selection neutral and save the brand colour for actions and checked controls.

## Type

Inter (system fallback). One scale, no other sizes:

| Token | Size / line | Weight | Use |
|---|---|---|---|
| text-xs | 12 / 16 | 500 | section labels, chips, captions, badges |
| text-sm | 14 / 20 | 400, 500 for controls | body, inputs, buttons, task titles, tabs |
| text-md | 16 / 24 | 600 | panel and inspector titles |
| text-lg | 24 / 32 | 600 | empty-state title |
| mono | 13 / 20 | 400 | Code tab, ids |

Body moved from 13 to 14: Mozaika's measured corpus puts 13px at the 6th percentile (median 16, p25 14; mozaika.design/measured). 14 is the floor for an editor that is read all day. Section labels are sentence case, never caps. An id shown next to a title is `text-xs` muted in Inter, not mono, and shares the title's baseline.

## Measurements (binding: every value here, nothing in between)

**Grid.** 4px base. Spacing steps: 4, 8, 12, 16, 24, 32. Nothing else (no 6, 10, 14, 20). Mozaika: 4px base is the corpus median.

**Nested radius rule.** A rounded element sitting `p` px inside a rounded parent of radius `R` gets radius `R − p`. Every container below is sized so the rule lands on a token; never pick a radius by eye.

| Element | Size | Padding | Radius | Inner radius it produces |
|---|---|---|---|---|
| radius-sm | — | — | 4 | checkbox, focus ring corners on chips |
| radius-md | — | — | 8 | every control: buttons, icon buttons, inputs, selects, segments, task rows, cards inside panels |
| Floating pill (title, tools, file, zoom) | height 40 | 4 all round | 12 | 12 − 4 = 8 → its 32px controls use radius-md |
| Floating panel (left panel, inspector, popovers) | — | 8 around lists, 16 around forms | 16 | lists: 16 − 8 = 8 → rows radius-md; forms: gap ≥ 16 so controls keep radius-md |
| Segmented control track | height 32 | 2 | 10 | 10 − 2 = 8 → segments 28 high, radius-md |
| Chip | height 24 | 0 8 | 6 | — |
| Checkbox | 16 | — | 4 | — |
| Template card on the empty canvas | 200 × 160 | 8 | 16 | 16 − 8 = 8 → preview tile radius-md |

Mozaika: 8px control radius is a normal value (33rd percentile; corpus median 12). Pills are 12, not fully round, so their 8px buttons nest exactly.

**Controls.** Every clickable control is 32 high (buttons, icon buttons, inputs, selects) except segments (28, inside their 32 track) and chips (24). Icon buttons are 32 × 32 with a 16px icon centred (8px each side). Text buttons: 0 12 padding, 8 gap between icon and label, icon 16. The primary button is the same 32 height as its neighbours; it never grows past the pill.

**Pills.** Height 40 = 32 control + 4 + 4. Gap 4 between controls in a group. Groups are split by a divider: 1 × 16, zinc `border`, 4 margin each side. A divider goes between every group, including before the primary button. Pills sit 16 from the window edges and 16 from each other.

**Panels.** 16 from the window edges and 16 below the pill row (top = 16 + 40 + 16 = 72). Header row: 32 high, padding 0 16, title `text-md` left, close icon button (32, `close` icon) right, both centred on the row's midline. 16 between header and body. Section label to its control: 8. Between form rows: 16.

**Task row.** Min height 36, padding 8, radius-md (panel list padding 8 → 16 − 8). Checkbox 16 at the left, 12 gap to the title, chips on a second line 4 below the title, aligned to the title's left edge, chips and tags in one wrapping flow with gap 4.

**Borders and elevation.** 1px everywhere. Light: hairline `0 1px 2px rgba(24,24,27,.05)`; panels `0 0 0 1px border, 0 8px 24px rgba(24,24,27,.08), 0 2px 6px rgba(24,24,27,.04)`. Dark (the default): no drop shadows; elevation is the 1px `border` ring plus the surface step (zinc-900 on zinc-950).

**Motion.** Hover and press 150ms ease-out; panel fade 200ms. Mozaika puts 120ms at the 7th percentile (median 200), so 120 is retired.

## Layout

- The canvas fills the whole window; there is no top bar, no docked side panels, no status bar.
- Top-left: a floating pill with the Rovelo mark, the diagram title (click to rename) and an unsaved dot.
- Top-centre: a floating tool pill: Select, Add node, Connect, then divider, Undo, Redo, divider, Auto-layout, Fit. Icon buttons 32 with tooltips and shortcuts.
- Top-right: a floating pill with Open, Save, Export (the only filled accent button here is Export).
- Left: a floating 300px panel, collapsible, with a segmented switch Tasks / Brand / Code. It never exceeds the window height minus 24px margins and scrolls inside.
- Right: the inspector floats at the right only while something is selected, 280px, and closes on Esc or click on empty canvas.
- Bottom-left: zoom pill (− 100% +, fit). Bottom-right: nothing.
- Empty canvas: a centred "Start a flow" block with three template cards (User flow, Task flow, Blank), each showing a small drawn preview of its shape, plus a one-line hint ("or double-click to add a step"). No modal frame, no backdrop.

## Components

- Icon button: 32 square, radius-md, no fill at rest, neutral hover, `zinc-800` fill (light: `zinc-100`) with full-strength icon when active, 2px focus ring in `accent` offset 2 (focus-visible only, one control at a time).
- Primary button: `accent` fill, white `text-sm` 500, height 32, radius-md, padding 0 12, icon 16 with 8 gap. Secondary: `surface` fill, 1px `border`. Ghost: no fill.
- Segmented control: `surface-sunk` track, active segment raised neutral (dark `zinc-700` on the `zinc-800` track; light `white` with hairline shadow on `zinc-100`), label `text`; inactive segments `text-muted`.
- Task row: 36px, checkbox 16 radius-sm (solid `accent` with a white tick when done, title struck through muted), title, then due date chip and a node link chip (clicking selects and centres the node). Overdue date in danger with a "!" so colour is never the only signal. The progress bar fill is `accent`.
- Input: `surface-sunk` fill, 1px `border`, radius-md, height 32, padding 0 12, focus border `accent` plus a 3px `accent-soft` ring.
- Slider: WoopKIT's `LabelSlider` in its Inset look (WoopAPPS/WoopKIT, Theme.swift `insetBody`), copied as is. One 32-tall row that is the track: `surface-sunk` fill, radius-md. Label inside at the left and the value inside at the right, both `text-sm`, 16 in from the ends. A wash of the `text` ink at 16 % from the left edge up to the handle. Handle: a 4 × 20 bar, radius 2, in `text`, always shown, 8 in from the ends, a quarter taller while dragged (150ms, cubic-bezier(0.25, 0, 0, 1)). The value is a fixed-width box sized to its widest form with digits as zeros, right-aligned, so it never moves; `text-muted` at rest, `text` on hover and while dragging. The label is `text-muted`, and `text` at weight 500 once the value differs from its default. A press anywhere sets the value and the pointer carries it; Option drags at 0.15×. Double-click the number to type a value (the value from before the first click comes back first; Return or a click away sets it, clamped and rounded to the step; Esc keeps the old one); double-click anywhere else resets to the default. A value with no natural span (a position) draws no wash and no handle and scrubs one step per point. Never fuchsia: the slider is neutral like every other selection.
- Swatch: 24 circle, 1px `border`, role name in a tooltip; selected has a 2px `accent` ring with a 2px gap.

## Motion

Feedback only: 150ms ease-out for hover tint and press (0.98 scale on buttons). Panels fade in and out over 200ms; no slides. Honour reduced motion.

## Rules

Never put a coloured fill behind chrome that is not the accent. No gradients, no glass blur, no decorative borders. Icons come only from `assets/icons/` (Nucleo UI, duotone filled, 18px grid, second tone at 0.4 opacity), drawn at 16px in buttons and 14px in chips and rows, coloured `text-muted` at rest and `text` on hover and when active.

## Diagram nodes and groups (default "Rovelo" kit)

From dark workflow canvases on Mobbin (Plain, Attio, AirOps, StackAI, Vapi, Runway). Colour lives in a small icon tile, not in the whole node; groups are tinted glass with a vibrant edge.

**Node style "Card" (default; the kit's `node_style`, alternative "Shape" keeps the classic outlines).**
- A card: kit `surface` fill (dark zinc-900, light white), 1px border (dark zinc-800, light zinc-200), radius 12, padding 12, min width 192, height 48 (one line) or 64 (title plus subtitle). Light adds the panel shadow; dark has none.
- Icon tile at the left: 28 × 28, radius 8, fill = the node's role colour at 16 %, glyph 16 in the role colour (dark: the role's 400 step; light: its 600 step). 12 gap to the text.
- Title `text-sm` 500 in `text`; subtitle `text-xs` muted. The subtitle shows to-do progress ("2 of 3 to-dos") when the step has to-dos, else the node's note; no separate floating badge. A step whose to-dos are all done shows a check glyph in the subtitle in fuchsia.
- Kind is shown by the icon, not the outline: start → play, decision → branch, end → flag, screen → window, note → note, plain step → the node's own icon or a dot. Decisions are cards too, labelled outgoing edges carry the branches.
- Ports: 8 circles, `surface` fill with a 1.5px border in zinc-500, centred on the card's edges, shown on hover, while connecting, and on the selected node.
- Selected: 2px `accent` outline 3 outside the card, radius 15.
- Shape style keeps diamonds and the rest of the outline shapes, drawn with the same surface fill, border and icon rules.

**Edges.** 1.5px, dark zinc-600 / light zinc-400, orthogonal with 12 radius corners (or the kit's routing), arrowhead 8. Role-coloured edges use the role's 400 step at 80 %, dashed 4 4, never dotted. Labels are chips: height 20, padding 0 8, radius 6, `surface` fill, 1px border zinc-700 / zinc-200, `text-xs` muted; never white pills on the dark canvas.

**Groups: tinted glass.**
- Fill: the group's role colour at 10 % (light 8 %) over a 16px backdrop blur where the renderer can blur (web, export via filter fallback); where it can't (egui), a vertical highlight from white 6 % at the top to 0 at 40 % stands in for it.
- Border: 1.5px in the role colour's vibrant step (dark 400, light 500) at 100 %, plus a 1px inner highlight of white 8 % along the top edge.
- Radius 20, padding 16 around members, 36 at the top for the title.
- Title: inside at top-left, 12 in, `text-xs` 600 in the role's vibrant step, preceded by a 8 dot of the same colour.
- Default group role is the kit's secondary colour (Rovelo: violet). No grey or lavender opaque frames.
