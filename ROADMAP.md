# Rovelo — roadmap

Read this before starting work, and update it when work lands: move finished
items out of **Next**, add what the work opened up.

**This file wins.** When a work order, audit, prompt or brief disagrees with
anything here — Done, Next, Settled, Working or Constraints — follow this
file, skip the conflicting part, and say which parts you skipped.

**Where a decision goes.** `PRD.md` holds the core mission only (none yet;
run /grill-it). `ROADMAP.md` names every feature and holds product-level
decisions: scope, data, architecture, process. `UI-SPECS.md` holds every UI
intricacy: layout, placement, sizes, motion, interactions, shortcuts, wording
and the directions chosen between alternatives. `DESIGN.md` holds the UI
tokens `UI-SPECS.md` names. Never record one decision in two places.

**Reversing.** A settled decision changes only when the user says so. Replace
it in place and keep what it replaced: in the roadmap, the bullet ends with
"Was: <old decision> — changed <date> because <why>".

## Done

- Diagram model: steps, arrows, groups and to-dos linked to steps; three kinds (flowchart, user flow, task flow); one-step-per-gesture undo.
- Brand kits: palette roles, fonts, radius, stroke, routing, fill mode, node style (card or shape), logo; presets Rovelo (default), Slate, Paper, Midnight; `.brand.json` import and export.
- Layered auto-layout with orthogonal routing that keeps clear of groups, and label chips that avoid group borders.
- SVG export (card steps, glass groups, Nucleo icons) and PNG export.
- Rovelo text format (`%% rovelo 1`) with round-trip laws, and Mermaid flowchart import.
- Native editor (`rovelo-app`, egui): floating layout, Tasks / Brand / Code panels, inspector, templates, making steps with plus handles, WoopKIT inset sliders, dark by default.
- Web editor (`web/`, React on the Rust core through `rovelo-wasm`): the same features, React Bits Pro empty state, ⌘K command menu and task list, real backdrop blur on groups; packaged as `Rovelo.app` with Tauri.

## Next

1. Click through the native open, save and export dialogs in both editors and fix what breaks.
2. Screenshot-check what no one has looked at yet: the native app in light theme, the User flow template, a Shape-style kit, a step whose to-dos are all done, and the web canvas with the new card steps.
3. When an arrow enters a selected step from the side, the plus handle sits on it; move the handle or the arrowhead so both stay visible.
4. A research pass on step, edge and group styling with the Refero tools (now connected), checked against `DESIGN.md` › Diagram nodes and groups.
5. Bring back features the original app had and Rovelo left out, one at a time: more diagram families (sequence, state, ERD, mind map), D2 and Structurizr import, animated export (GIF/MP4), the AI assistant, the agent bridge.
6. Write the PRD with /grill-it.

## Settled — don't reopen

- All diagram logic lives in Rust (`rovelo-core`, `rovelo-dsl`); both editors call it, the web one through `rovelo-wasm`. Why: the user asked for a complete Rust rebuild of openflowkit.
- Two front ends: the native egui editor and the React + Tauri editor. Why: the user wants React Bits Pro components, which only exist for React, without giving up the Rust core.
- The web UI uses React Bits Pro where a component exists, shadcn for the rest, with their default states. Why: the user's instruction ("use default states in React Bits Pro, stop improvising").
- UI decisions come from references (the original app, Mobbin, Mozaika, Refero, WoopKIT), never invented. Why: the user rejected improvised UI repeatedly.
- The React Bits licence key stays in the shadcn tool's own environment and never in a file. Why: it is a paid key.
- Rovelo started as a fork of openflowkit and has its own history from its first commit (2026-11-02). Why: the original's commit messages carried AI co-author lines.
- Name: Rovelo. File extension `.rovelo.json`; text format pragma `%% rovelo 1`.
- To-dos live on the diagram: a to-do links to a step or stands alone, and a step shows its to-do progress. Why: the product is a to-do app built around diagrams.

## Reading map

Always: this file, `UI-SPECS.md` (binding for anything a person sees), `DESIGN.md` (binding tokens and component rules), `README.md`. Before any judgment call: `PRD.md` once it exists; until then `PRODUCT.md` (product context for the design tools).

- Touches the canvas or exports → `DESIGN.md` › Diagram nodes and groups; `crates/rovelo-core/src/{geometry,layout,svg,svg_shapes,card}.rs`; `crates/rovelo-app/src/{canvas,cards,quick}.rs`; `web/src/ui/Canvas.tsx`, `web/src/lib/{quickCreate,glass}.ts`.
- Touches making steps → `UI-SPECS.md` › Flows › Making a step.
- Touches the text format or Mermaid import → `crates/rovelo-dsl/GRAMMAR.md`.
- Touches the web editor → `web/README.md`, `web/REACTBITS.md`.
- Touches icons → `assets/icons/README.md` (Nucleo UI, duotone filled only).
- Touches sliders → `DESIGN.md` › Components › Slider (WoopKIT's `LabelSlider`, Inset look).

## Working in this repo

- Several agent sessions may edit this working tree at once. Re-read a file right before editing it, never revert changes you didn't make, and don't commit or push unless asked.
- Never force-push or rewrite history without the user's explicit OK.
- Build: `cargo build --release -p rovelo-app` (binary `target/release/rovelo`); web: `cd web && pnpm install && pnpm wasm && pnpm build`; Mac app: `cd web && pnpm tauri build --bundles app`.
- Test: `cargo test --workspace` and `cd web && pnpm test`.
- Run: `target/release/rovelo`, or `cd web && pnpm dev` (http://localhost:5173).
- Screenshot helpers for the native app: `ROVELO_TEMPLATE=user|task`, `ROVELO_THEME=light|dark`, `ROVELO_TAB=brand|code`, `ROVELO_SELECT=<step id>`.
- Before calling UI work done: build, test, then look at the change on a real screenshot.
- Never rename `.rovelo.json`, `.brand.json` or the `%% rovelo 1` pragma; saved files depend on them.

## Constraints that shape "next"

- `/Volumes/Work` is nearly full (about 4 GB free). A full Rust plus Tauri build needs about 2 GB; clean only this repo's `target/` when space runs out.
- React Bits Pro installs need `REACTBITS_LICENSE_KEY` in the shadcn tool's environment, and the shadcn tool must run from `web/` to see the registries.
