# Rovelo web

React front end for Rovelo. All diagram logic (parse, serialize, Mermaid import, layout, geometry, SVG export,
brand kits) is the Rust code in `crates/rovelo-core` and `crates/rovelo-dsl`, compiled to WebAssembly by
`crates/rovelo-wasm` and called from TypeScript. The page keeps the diagram as plain JSON and edits it with
immutable updates (`src/lib/ops.ts`); Rust is asked only for layout, geometry, text and SVG.

## Run it

Prerequisites: Rust with the `wasm32-unknown-unknown` target, `wasm-bindgen-cli` 0.2.130 (`cargo install wasm-bindgen-cli --version 0.2.130`), Node 26, pnpm 11.

```bash
cd web
pnpm install
pnpm wasm          # builds the Rust core to src/wasm/pkg (re-run after changing any crate)
pnpm dev           # http://localhost:5173
pnpm test          # Vitest, 71 tests (the wasm test runs if `pnpm wasm` has been run)
pnpm build         # type-check + production bundle in dist/
cargo test -p rovelo-wasm    # from the repo root: native tests of the wasm wrapper logic
```

Desktop shell (Tauri 2, macOS):

```bash
pnpm tauri dev                   # window around the dev server
pnpm tauri build --bundles app   # Rovelo.app in src-tauri/target/release/bundle/macos/
```

## Layout of the code

- `src/wasm/index.ts` typed bridge over the wasm exports (JSON in, JSON out).
- `src/state/` the store (`store.ts`), every user action (`actions.ts`), burst-to-one-undo-step helper (`burst.ts`).
- `src/lib/` pure logic with tests: `history`, `geom` (snap, marquee, viewport), `tasks`, `ops`, `nodeIcons`, `files`.
- Making a step follows UI-SPECS › Flows › Making a step, ported from the original app: `src/lib/quickCreate.ts` (placement maths) and the plus handles, ghost and Add step tool in `src/ui/Canvas.tsx`.
- Groups on the canvas are HTML glass layers under the SVG (real backdrop blur); exports keep the core's SVG.
- `src/components/InsetSlider.tsx` the app's only slider (WoopKIT Inset look); its maths is `src/lib/slider.ts`.
- `src/ui/` Canvas, floating chrome, panels, inspector, `slots.tsx` (React Bits swap points).
- `src/components/` React Bits (free) and shadcn/ui components, as installed by the shadcn CLI.
- Icons: only `/assets/icons/*.svg` (Nucleo UI duotone), inlined by `src/ui/icons.tsx`.
- Files: `.rovelo.json` for diagrams, `.brand.json` for brand kits. Tauri dialogs in the desktop shell, the File System Access API in browsers that have it, download and upload elsewhere.

## Dependencies, one line each

Runtime:
- `react`, `react-dom`: the UI.
- `radix-ui`: accessible primitives behind the shadcn components (menus, select, dialog, slider, switch, tooltip).
- `class-variance-authority`: variant styling for shadcn buttons.
- `cn` (installed by the shadcn CLI): class-name merging used by every shadcn component.
- `shadcn`: provides the `tailwind.css` base layer the generated components import.
- `tw-animate-css`: enter and exit animations for shadcn overlays.
- `@fontsource-variable/inter`: Inter, bundled so the app works offline.
- `motion`: used by the React Bits `BlurText` and `CountUp` components (`CountUp` is in use; `BlurText` is kept but unused).
- `gsap`: used by the React Bits `FadeContent` component.
- `@tauri-apps/api`, `@tauri-apps/plugin-dialog`, `@tauri-apps/plugin-fs`: native open and save in the desktop shell (loaded only there).

Development:
- `vite`, `@vitejs/plugin-react`: dev server and bundler. The wasm file is loaded with Vite's `?url` import and wasm-bindgen's `--target web` glue, so no wasm plugin is needed.
- `typescript`, `@types/*`: type-checking.
- `tailwindcss`, `@tailwindcss/vite`: styling (Tailwind v4, fuchsia and zinc ramps).
- `vitest`, `jsdom`: unit tests.
- `@tauri-apps/cli`: builds the desktop shell without a global install.

## Notes

- The text language still carries its working name in its first line (`%% rovelo 1`); that is the Rust crate's grammar and is unchanged here.
- The diagram autosaves to browser storage and is restored on reload; Save writes a `.rovelo.json` file.
- Dark is the default and the system appearance is never followed; the sun/moon button in the top-right pill switches theme and the choice is kept in browser storage. `?theme=dark|light` forces one (for screenshots).
- React Bits Pro blocks (empty state, command palette, checkbox and progress) are in `src/components/blocks/`; see `REACTBITS.md`. `Cmd/Ctrl+K` opens the command palette.
