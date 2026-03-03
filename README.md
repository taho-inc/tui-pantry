# tui-pantry

Storybook-like harness for previewing [ratatui](https://ratatui.rs) widgets in isolation. Generic library with zero application dependencies.

## Installation

Install the cargo subcommand:

```bash
cargo install --path .          # from the tui-pantry directory
```

Then add `tui-pantry` as an **optional** dependency in your widget crate:

```toml
[features]
pantry = ["dep:tui-pantry"]

[dependencies]
tui-pantry = { version = "0.1.0", optional = true }
```

## Getting Started

### 1. Create `pantry.toml`

Place a `pantry.toml` at your widget crate root. This declares which modules contain ingredients and (optionally) defines your color palette and typography:

```toml
[ingredients]
source = "my_crate"
modules = [
    "widgets::gauge",
    "widgets::table",
]
```

### 2. Write ingredient files

For each module listed, create a colocated `.ingredient.rs` file gated behind the `pantry` feature:

```rust
// widgets/gauge/mod.rs
#[cfg(feature = "pantry")]
#[path = "gauge.ingredient.rs"]
pub mod ingredient;
```

Each ingredient file exports a factory (see [Creating Ingredients](#creating-ingredients) below):

```rust
// widgets/gauge/gauge.ingredient.rs
pub fn ingredients() -> Vec<Box<dyn tui_pantry::Ingredient>> {
    vec![Box::new(GaugeDefault), Box::new(GaugeHigh)]
}
```

### 3. Run

```bash
cargo pantry                    # from your widget crate root
```

`cargo pantry` auto-creates `examples/pantry.rs` if missing, then runs `cargo run --example pantry --features pantry`. The example is a one-liner:

```rust
fn main() -> std::io::Result<()> {
    tui_pantry::run!()
}
```

The `run!()` macro reads `pantry.toml` at compile time via a proc macro to discover ingredients, and at runtime to parse stylesheet entries. It takes ownership of the terminal, renders a two-pane browser (sidebar + live preview), and restores terminal state on exit. `tui-pantry` re-exports `ratatui` so ingredient authors don't need a separate dependency.

## Concepts

**Ingredients** are the unit of display — one "story" per widget configuration. They organize into:

- **Tabs** — top-level categories: Widgets (default), Views, Styles
- **Groups** — the widget name, shown as a collapsible tree parent
- **Variants** — specific configurations under a group

## Creating Ingredients

### Basic ingredient

```rust
use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};
use tui_pantry::{Ingredient, PropInfo};

struct GaugeDefault;

impl Ingredient for GaugeDefault {
    fn group(&self) -> &str { "Resource Gauge" }
    fn name(&self) -> &str { "Default" }
    fn source(&self) -> &str { "my_crate::widgets::resource_gauge" }

    fn description(&self) -> &str {
        "Horizontal bar showing resource utilization with color thresholds"
    }

    fn props(&self) -> &[PropInfo] {
        &[
            PropInfo { name: "label", ty: "&str", description: "Resource name displayed left of the bar" },
            PropInfo { name: "ratio", ty: "f64", description: "Fill from 0.0 to 1.0; drives color threshold" },
        ]
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        ResourceGauge::new("CPU", 0.34).render(area, buf);
    }
}
```

`group()`, `name()`, `source()`, and `render()` are required. Everything else has defaults.

### Interactive ingredient

Set `interactive()` to `true` to receive keyboard input when the preview pane has focus. Press Enter in the sidebar to focus; Esc to return.

```rust
struct TableInteractive {
    selected: usize,
    rows: Vec<Row>,
}

impl Ingredient for TableInteractive {
    fn group(&self) -> &str { "Node Table" }
    fn name(&self) -> &str { "Interactive" }
    fn source(&self) -> &str { "my_crate::widgets::node_table" }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        NodeTable::new(&self.rows, Some(self.selected)).render(area, buf);
    }

    fn interactive(&self) -> bool { true }

    fn handle_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Up | KeyCode::Char('k') => { self.selected = self.selected.saturating_sub(1); true }
            KeyCode::Down | KeyCode::Char('j') => { self.selected = (self.selected + 1).min(self.rows.len().saturating_sub(1)); true }
            _ => false,
        }
    }
}
```

Return `true` from `handle_key()` to consume the event, `false` to ignore it.

### Tab assignment

Override `tab()` to place ingredients in the Views or Styles tab:

```rust
fn tab(&self) -> &str { "Views" }     // multi-widget compositions
fn tab(&self) -> &str { "Styles" }    // palettes, typography, tokens
```

## Trait Reference

### Required

| Method | Purpose |
|--------|---------|
| `group()` | Widget name — collapsible heading in the sidebar. Shared `group` values nest together. |
| `name()` | Variant label — leaf node under the group. |
| `source()` | Module path shown as a breadcrumb in the preview pane. |
| `render()` | Draw the widget into the preview area. |

### Optional (with defaults)

| Method | Default | Purpose |
|--------|---------|---------|
| `tab()` | `"Widgets"` | Top-level tab: `"Widgets"`, `"Views"`, or `"Styles"`. |
| `description()` | `""` | One-line summary displayed in the preview pane. |
| `props()` | `&[]` | `PropInfo` slice documenting the widget's configurable surface. |
| `interactive()` | `false` | Whether the preview pane captures keyboard input. |
| `handle_key()` | `false` | Process a key event while the preview pane has focus. |

### PropInfo

```rust
pub struct PropInfo {
    pub name: &'static str,
    pub ty: &'static str,
    pub description: &'static str,
}
```

Props describe the widget's API, not a specific variant's data. Typically only the "Default" variant in a group returns them.

## Registration

### Via `pantry.toml` (recommended)

The `[ingredients]` table in `pantry.toml` maps module paths to their `ingredient::ingredients()` factories. The `pantry_ingredients!()` proc macro reads this at compile time:

```toml
[ingredients]
source = "my_crate"
modules = [
    "widgets::gauge",
    "widgets::node_table",
]
```

Each entry expands to `my_crate::widgets::gauge::ingredient::ingredients()`, etc. Multiple source crates are supported via array-of-tables syntax:

```toml
[[ingredients]]
source = "crate_a"
modules = ["widgets::foo"]

[[ingredients]]
source = "crate_b"
modules = ["widgets::bar"]
```

Adding a new widget requires two touches: the `#[cfg]` declaration in `mod.rs` and a module entry in `pantry.toml`.

### Manual aggregation

For full control, pass an ingredient vector directly:

```rust
fn main() -> std::io::Result<()> {
    tui_pantry::run!(my_crate::pantry::ingredients())
}
```

### Feature-gating

Gate ingredient modules so they don't compile into production builds:

```toml
# widget library Cargo.toml
[features]
pantry = ["dep:tui-pantry"]
```

```rust
// widget's mod.rs
#[cfg(feature = "pantry")]
#[path = "gauge.ingredient.rs"]
pub mod ingredient;
```

## Stylesheet

The Styles tab can be driven entirely from `pantry.toml` — no manual ingredient code required. Add `source`, `[colors]`, and `[typography]` sections alongside your `[ingredients]`:

```toml
source = "my_crate::styles"

[colors.brand]
deep_purple = "#2E1574"
white = "#FFFFFF"

# Numeric keys render as a horizontal scale strip
[colors.green]
100 = "#DCFCE7"
500 = "#22C55E"
900 = "#14532D"

[typography]
text = { color = "#FFFFFF", description = "Primary content" }
text_dim = { color = "DarkGray", description = "Secondary labels" }

[ingredients]
source = "my_crate"
modules = ["widgets::gauge"]
```

A standalone `styles.toml` is also supported as a fallback if `pantry.toml` is absent.

### Colors

Each `[colors.<family>]` table becomes a sidebar group under the Styles tab. Named keys (snake_case) render as individual swatches with a colored block, display name, and hex value. Numeric keys render as a horizontal scale strip showing the gradient across values.

The optional top-level `source` field sets the breadcrumb module path for all generated ingredients (e.g., `my_crate::styles::palette::brand`).

### Typography

Each key under `[typography]` renders sample text in its own color with the description alongside. Color values accept hex (`"#FFFFFF"`) or named ratatui colors (`"DarkGray"`).

### Missing stylesheet

If `styles.toml` is absent, the Styles tab displays an inline prompt showing the expected TOML format. Programmatic `Ingredient` implementations with `tab() = "Styles"` still work alongside or instead of the TOML-driven approach.

## Layout Helpers

`tui_pantry::layout::render_centered` centers a widget on one or both axes:

```rust
use tui_pantry::layout::render_centered;

fn render(&self, area: Rect, buf: &mut Buffer) {
    render_centered(
        MyWidget::new(),
        Some(Constraint::Length(40)),   // width, or None to fill
        Some(Constraint::Length(10)),   // height, or None to fill
        area, buf,
    );
}
```

## Variant Patterns

| Pattern | Use |
|---------|-----|
| Unit struct | Static mock data, no interaction |
| Stateful struct | Interactive variants with mutable state |
| Data-driven | Same widget, different values (e.g., gauge at 0.3 / 0.7 / 0.9) |
| Composition | Multiple widgets in one preview |
| Empty state | How the widget handles no data |

## Keys

| Key | Sidebar | Preview |
|-----|---------|---------|
| `j/k` `↑/↓` | Navigate | Forwarded to ingredient |
| `h/l` `←/→` | Collapse/expand | — |
| `Enter` | Toggle group or focus preview | — |
| `1` `2` `3` / `Tab` | Switch tabs | — |
| `Esc` | Quit | Return to sidebar |
| `q` | Quit | — |
