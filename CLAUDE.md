# tui-pantry — TUI Widget Preview Harness

Storybook-like harness for developing ratatui widgets in isolation. Top-bar tabs organize Widgets, Views, and Styles. Sidebar navigation with live widget preview.

## Crate Structure

```
tui-pantry (lib + cargo-pantry bin)  ← generic harness + cargo subcommand (OSS-extractable)
  └── tui-pantry-macros              ← proc macro (re-exported, invisible to consumers)
```

`tui-pantry` owns the `Ingredient` trait, nav tree, app loop, and all rendering chrome. It also ships the `cargo-pantry` binary — a cargo subcommand that auto-creates `examples/pantry.rs` and delegates to `cargo run --example pantry --features pantry`. `tui-pantry-macros` provides the `pantry_ingredients!()` proc macro that reads `pantry.toml` at compile time.

Widget crates (e.g. `taho-tui`) keep `pantry.toml` at their crate root and colocate `.ingredient.rs` files behind `#[cfg(feature = "pantry")]`.

## `pantry.toml`

Single config file at the widget crate root declaring both styles and widget ingredients:

```toml
source = "my_crate::styles"

[colors.brand]
deep_purple = "#2E1574"
white = "#FFFFFF"

[typography]
text = { color = "#FFFFFF", description = "Primary content" }

[ingredients]
source = "my_crate"
modules = [
    "widgets::node_table",
    "widgets::event_list",
]
```

**Styles** are parsed at runtime by `stylesheet::from_toml` — color swatches and typography samples appear in the Styles tab. The optional top-level `source` field annotates breadcrumbs with the originating module path.

**Ingredients** are evaluated at compile time by the `pantry_ingredients!()` proc macro. Each module entry expands to `{source}::{module}::ingredient::ingredients()`. Multiple source groups are supported via `[[ingredients]]` array-of-tables syntax.

`run()` looks for `pantry.toml` first, falling back to `styles.toml` for backward compatibility.

## Ingredient Convention

Each widget ships colocated `.ingredient.rs` files gated behind `#[cfg(feature = "pantry")]`:

```rust
// in widget's mod.rs
#[cfg(feature = "pantry")]
#[path = "node_table.ingredient.rs"]
pub mod ingredient;
```

Each ingredient file exports `pub fn ingredients() -> Vec<Box<dyn Ingredient>>`.

The `Ingredient` trait provides optional `description()` and `props()` methods for self-documenting widgets. When present, the harness renders a doc panel below the preview showing the widget's purpose and configurable properties.

Adding a new widget requires two touches: the `#[path]` declaration in `mod.rs` and a module entry in `pantry.toml`.

## Running

```bash
cargo pantry        # workspace alias (runs taho-tui example)
```

External users: `cargo install tui-pantry` then `cargo pantry` from their widget crate root.

Keys: `j/k` or `↑/↓` navigate, `h/l` or `←/→` collapse/expand, `Enter` toggle or enter preview, `1-3` direct tab access, `Tab`/`Shift-Tab` cycle tabs, `q` quit.

## Development

Watch for changes and re-run the harness:

```bash
cargo watch -w taho-tui -x "pantry"
```

## Key Files

- [src/lib.rs](src/lib.rs) — `run!` macro and `run()` entry
- [src/bin/cargo-pantry.rs](src/bin/cargo-pantry.rs) — cargo subcommand runner
- [crates/tui-pantry-macros/src/lib.rs](crates/tui-pantry-macros/src/lib.rs) — `pantry_ingredients!()` proc macro
- [src/ingredient.rs](src/ingredient.rs) — `Ingredient` trait definition
- [src/stylesheet.rs](src/stylesheet.rs) — TOML parser → color/typography ingredients
- [src/app.rs](src/app.rs) — event loop and key dispatch
- [src/nav.rs](src/nav.rs) — `NavTree`: grouped entries, expand/collapse, cursor, viewport scrolling
- [src/ui.rs](src/ui.rs) — two-pane layout, top bar tabs, sidebar, preview, bottom bar
- [src/pane.rs](src/pane.rs) — `Pane` widget: titled border delegating to an ingredient
- [src/swatch.rs](src/swatch.rs) — purple gradient background

## Top-Bar Tabs

Three tabs organize ingredient types:

- **Widgets** — Component-level widget stories (default tab)
- **Views** — Composition of multiple widgets together
- **Styles** — Color palette swatches, typography, and theme documentation

`Ingredient::tab()` returns which tab an ingredient belongs to (default: `"Widgets"`). The harness builds a separate `NavTree` per tab, preserving independent selection, expansion, and scroll state when switching. Each tab maintains its own navigation context.

## Planned

- Themable pantry chrome (`PantryTheme` config passed to `run()`) for OSS consumers
- Alt-key accelerators for direct navigation jumps (Phase 3 in [phased plan](docs/tui-pantry-phased-plan.md))
- `.pantry-state` persistence across restarts
- Additional widgets: JobQueue, MeshTopology, StatusBar, LogStream
