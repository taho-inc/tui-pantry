# tui-pantry — TUI Widget Preview Harness

Storybook-like harness for developing ratatui widgets in isolation. Top-bar tabs organize Widgets, Views, and Styles. Sidebar navigation with live widget preview.

## Crate Structure

```
tui-pantry (lib + cargo-pantry bin)  ← generic harness + cargo subcommand
  ├── tui-pantry-macros              ← proc macro (re-exported, invisible to consumers)
  └── examples/example-pantry        ← ratatui stock widget showcase (Catppuccin Mocha)
```

`tui-pantry` owns the `Ingredient` trait, nav tree, app loop, and all rendering chrome. It also ships the `cargo-pantry` binary — a cargo subcommand that auto-creates `examples/pantry.rs` and delegates to `cargo run --example pantry --features pantry`. `tui-pantry-macros` provides the `pantry_ingredients!()` proc macro that reads `pantry.toml` at compile time.

Widget crates (e.g. `taho-tui`) keep `pantry.toml` at their crate root and colocate `.ingredient.rs` files behind `#[cfg(feature = "pantry")]`.

`example-pantry` is a reference implementation showing how to stand up a pantry from scratch using ratatui's built-in widgets (Block, Paragraph, List, Table, Tabs, Gauge, BarChart, Sparkline) with a complete Catppuccin Mocha color system. Run with `cargo run -p example-pantry`.

## `pantry.toml`

Single config file at the widget crate root declaring harness config, styles, and widget ingredients:

```toml
[config]
theme = "light"                        # "dark" (default) or "light"
style_source = "my_crate::styles"      # breadcrumb prefix for color/typography ingredients

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

**Config** (`[config]`) is parsed at runtime. `theme` selects chrome colors (`PantryTheme::dark()` or `PantryTheme::light()`). `style_source` annotates breadcrumbs with the originating module path.

**Styles** (`[colors.*]`, `[typography]`) are parsed at runtime by `stylesheet::from_toml` — color swatches and typography samples appear in the Styles tab.

**Ingredients** are evaluated at compile time by the `pantry_ingredients!()` proc macro. Each module entry expands to `{source}::{module}::ingredient::ingredients()`. Multiple source groups are supported via `[[ingredients]]` array-of-tables syntax.

`run()` looks for `pantry.toml` first, falling back to `styles.toml` for backward compatibility.

## Ingredient Convention

Each module listed in `pantry.toml` must export `pub mod ingredient` with a `pub fn ingredients() -> Vec<Box<dyn Ingredient>>`. Two patterns:

**Inline** (dedicated pantry crates like `example-pantry`) — ingredient structs live alongside the widget in a single file:

```rust
// widgets/barchart.rs
pub mod ingredient {
    pub fn ingredients() -> Vec<Box<dyn Ingredient>> { vec![...] }
}
```

**Colocated** (production widget crates like `taho-tui`) — separate `.ingredient.rs` files gated behind a feature flag so the widget crate doesn't unconditionally depend on `tui-pantry`:

```rust
// widgets/node_table/mod.rs
#[cfg(feature = "pantry")]
#[path = "node_table.ingredient.rs"]
pub mod ingredient;
```

The `Ingredient` trait provides optional `description()` and `props()` methods for self-documenting widgets. When present, the harness renders a doc panel below the preview showing the widget's purpose and configurable properties.

Adding a new widget requires two touches: the `pub mod ingredient` declaration and a module entry in `pantry.toml`.

## Running

```bash
cargo pantry                 # workspace alias (runs taho-tui example)
cargo run -p example-pantry  # ratatui stock widget showcase
```

External users: `cargo install tui-pantry` then `cargo pantry` from their widget crate root.

Keys: `j/k` or `↑/↓` navigate, `h/l` or `←/→` collapse/expand, `Enter` toggle or enter preview, `1-3` direct tab access, `Tab`/`Shift-Tab` cycle tabs, `q` quit. Mouse: click sidebar entries to navigate, click tabs to switch, scroll wheel in sidebar to move up/down.

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
- [src/theme.rs](src/theme.rs) — `PantryTheme`: dark/light chrome palettes, parsed from `[config]`
- [src/pane.rs](src/pane.rs) — `Pane` widget: titled border delegating to an ingredient
- [src/swatch.rs](src/swatch.rs) — `GradientSwatch`: parameterized background gradient
- [examples/example-pantry/](examples/example-pantry/) — reference pantry with ratatui stock widgets and Catppuccin Mocha styles

## Top-Bar Tabs

Three tabs organize ingredient types:

- **Widgets** — Component-level widget stories (default tab)
- **Views** — Composition of multiple widgets together
- **Styles** — Color palette swatches, typography, and theme documentation

`Ingredient::tab()` returns which tab an ingredient belongs to (default: `"Widgets"`). The harness builds a separate `NavTree` per tab, preserving independent selection, expansion, and scroll state when switching. Each tab maintains its own navigation context.

## Planned

- Alt-key accelerators for direct navigation jumps (Phase 3 in [phased plan](docs/tui-pantry-phased-plan.md))
- `.pantry-state` persistence across restarts
