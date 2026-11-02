# Rovelo

A to-do app built around diagrams. Draw user flows, task flows and flowcharts, hang to-dos on any step, and restyle the whole diagram with a brand kit in one click. Local-first, no account.

Rovelo began as a fork of [openflowkit](https://github.com/openflowkit/openflowkit) and is rebuilt with a Rust core.

## Run it

```bash
cargo build --release -p rovelo-app && target/release/rovelo
```

Web and Mac app:

```bash
cd web && pnpm install && pnpm wasm && pnpm dev
```

```bash
cd web && pnpm tauri build --bundles app
```

## Layout

- `crates/rovelo-core` — model, brand kits, layout, routing, SVG export
- `crates/rovelo-dsl` — the Rovelo text format and Mermaid import ([GRAMMAR.md](crates/rovelo-dsl/GRAMMAR.md))
- `crates/rovelo-app` — native editor (egui)
- `crates/rovelo-wasm`, `web/` — React editor on the same core, packaged with Tauri
- `assets/icons` — the icon set

## Docs

[ROADMAP.md](ROADMAP.md) (features and product decisions) · [UI-SPECS.md](UI-SPECS.md) (every UI decision) · [DESIGN.md](DESIGN.md) (tokens and components) · [PRODUCT.md](PRODUCT.md) (product context) · PRD.md (not written yet)

## Licence

MIT; see [LICENSE](LICENSE).
