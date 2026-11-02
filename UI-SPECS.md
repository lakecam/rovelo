# Rovelo — UI specs

Every UI intricacy the roadmap is too high-level for: the rules every surface follows, the words it uses, and the layout, placement, sizes, motion, shortcuts and interactions chosen for each piece. Binding, not re-argued per feature.

`PRODUCT.md` → `DESIGN.md` → `UI-SPECS.md`. This file names `DESIGN.md` tokens and never repeats their values. A settled decision changes only when the user says so; replace it in place and move the old direction into that entry's Rejected line.

## Brand and design system

Every surface follows `DESIGN.md`. Every colour, type size, spacing step, radius and shadow comes from a `DESIGN.md` token. Values that belong to one layout (a panel's width) stay here as exact numbers. A UI decision that would break a `DESIGN.md` rule: stop and ask the user.

## Vocabulary

| Word | Means | Never |
|---|---|---|
| Step | A node in a diagram | Box, shape (in copy) |
| To-do | A task, linked to a step or standing alone | Ticket, item |
| Brand kit | Palette, fonts, radius, routing and logo applied to a diagram | Theme (theme is the app's light/dark) |

## Flows

### Making a step — copied from the original app, not invented
Source, in the original openflowkit repo (github.com/openflowkit/openflowkit, `src/opencanvas/`): `domain/connectors/quickCreate.ts`, `presentation/v2/v2PointerFinish.ts` (`finishCreate`, `finishConnect`), `v2PointerCommands.ts` (`placeShape`, `commitQuickCreateDelivery`), `infrastructure/pixi/PixiSelectionOverlay.ts` (`drawHandles`, `drawQuickCreateGhost`). When in doubt, read those files and do what they do.

1. **Plus handles.** The selected step, and the step under the pointer, show four round handles, one at the midpoint of each side just outside the edge. Each is a 16 px (screen size, not zoomed) circle, `surface` fill, `accent` 1.25 px stroke, with a plus inside. The hovered handle fills `accent` with the plus in `surface`.
2. **Ghost on hover.** Hovering a handle previews what a click makes: a box the size of the source, one full node size past that side (the next node's origin is the source's origin plus twice its width or height), `accent` at 5 % fill and 45 % stroke, with the arrow into it.
3. **Click a handle.** Makes that step at the ghost's place: same kind, size and look as the source, blank label (from a decision it makes a plain step, not another decision), plus the arrow from the source. One undo step. The new step is selected, the tool goes back to Select, and its label editor opens at once so you type straight away. If the new step lands under a panel or off-screen, the view pans the least it takes to show it, zoom kept.
4. **Drag from a handle.** A live arrow follows the pointer. Release on another step connects to it. Release on empty canvas makes the new step centred where you let go, connected, editor open, one undo step. Shift releases a free arrow.
5. **Add step tool (N).** A click places a default-size step centred on the click; a drag draws its box (snapped, Shift for square). The tool returns to Select and the editor opens on the new step. Esc mid-drag makes nothing.
6. **Double-click empty canvas.** A default-size step centred there, editor open.
7. **Label editor.** Edits in place on the step, in the step's own type. Enter, Tab or Esc commits and closes; Shift+Enter is a new line. Arrow routing between steps is automatic and picks the facing sides live, so steps can be rearranged freely afterwards.

## Decided directions

### Main window layout — 2026-11-02, from proto/shell (Floating, Docked, Tasks first)
- Direction: Floating. The canvas fills the window on `paper` with the dot grid; all chrome floats over it as pills and panels.
- Values: title pill top-left; tool pill top-centre (Select, Pan, Add step, Connect | Undo, Redo | Auto-layout, Fit); file pill top-right with Export as the only `accent` filled button; left panel 300 wide with a Tasks / Brand / Code segmented switch, 16 from the window edges; inspector 280 wide at the right, shown only while something is selected; zoom pill bottom-left. Panels use `surface`, radius 14, the panel shadow.
- Applies to: the whole editor window, in both the egui app and the web front end.
- Rejected: Docked (top bar, icon rail, docked panel and status bar) — took room from the canvas and looked like every other desktop tool. Tasks first (full to-do list left, diagram in a card right) — gave the diagram only about 60% of the window.

### Selected, active and checked states — 2026-11-02, from the user and Mobbin dark editors
- Direction: selection is neutral, checked is fuchsia. Shadcn's default state model (the base React Bits Pro components use), as in Rive, Krea, Webflow and Zoho Canvas.
- Values: see `DESIGN.md` › Colour, the "States follow the shadcn defaults" block.
- Applies to: every tab, segment, filter, tool, row, card, swatch, checkbox, switch and radio in both front ends.
- Rejected: fuchsia-tinted selection on every selected item — jarring, all pink.

### Diagram nodes, edges and groups — 2026-11-02, from the user and Mobbin workflow canvases
- Direction: card nodes with a coloured icon tile (Plain, Attio, AirOps), neutral edges with chip labels, groups as tinted glass with a vibrant border (StackAI's coloured frames).
- Values: see `DESIGN.md` › Diagram nodes and groups.
- Applies to: the canvas in both front ends and SVG/PNG export; the default "Rovelo" kit. Other kits keep their colours but use the same card style unless their `node_style` is Shape.
- Rejected: whole-node pastel fills with thick coloured outlines and diamonds for decisions; opaque lavender group frames with a solid border; white edge-label pills and red dotted edges on the dark canvas — "hideous", per the user.

### Sliders — 2026-11-02, from the user
- Direction: the same sliders as WoopKIT, its Inset look (`LabelSlider` › `insetBody` in /Volumes/Work/Projects/WoopAPPS/WoopKIT/Sources/WoopKIT/Kit/Theme.swift; registry entry `label-slider`).
- Values: see `DESIGN.md` › Components › Slider.
- Applies to: every slider in both front ends (Brand tab: corner radius, stroke width, font size; anything added later).
- Rejected: egui's and shadcn's stock sliders (a thin rail with a round thumb and the value in a separate label).

### Icons — 2026-11-02, from the user
- Direction: the user's Nucleo icons, UI family, duotone filled (glyph-duo). One set only, for chrome and for step icons in diagrams and exports.
- Values: files in `assets/icons/`, role → file mapping in `assets/icons/README.md`; 16 in toolbar buttons, 14 in chips and rows; second tone at the file's own 0.4 opacity.
- Applies to: every icon in every surface, including step icons on the canvas and in SVG/PNG exports.
- Rejected: hand-painted 1.5px line icons — the user asked for Nucleo duotone filled.
