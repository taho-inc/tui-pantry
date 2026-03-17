# tui-pantry-macros

Proc macros for [tui-pantry](https://crates.io/crates/tui-pantry). **Do not depend on this crate directly** — use `tui-pantry`, which re-exports everything.

## What it provides

`pantry_ingredients!()` — reads `pantry.toml` from your crate root at compile time and generates code that collects all listed ingredient modules into a `Vec<Box<dyn Ingredient>>`.

```toml
# pantry.toml
[ingredients]
source = "my_crate"
modules = ["widgets::button", "widgets::card"]
```

```rust
let ingredients = tui_pantry::pantry_ingredients!();
```

Each module expands to `{source}::{module}::ingredient::ingredients()`. The macro also tracks `pantry.toml` via `include_str!` so Cargo rebuilds when the config changes.

## License

MIT OR Apache-2.0
