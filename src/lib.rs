mod app;
mod color_depth;
mod ingredient;
pub mod layout;
mod nav;
mod pane;
pub mod stylesheet;
pub mod theme;
mod ui;

use std::io;

pub use ingredient::{Ingredient, PropInfo, is_click};
pub use pane::Pane;

/// Re-export ratatui primitives that ingredient authors need.
pub use ratatui;

/// Re-export the proc macro so consumers only depend on `tui-pantry`.
pub use tui_pantry_macros::pantry_ingredients;

/// Boot the pantry harness.
///
/// **No-args form** — reads `[[ingredients]]` from `pantry.toml` via
/// proc macro and discovers styles at runtime:
///
/// ```ignore
/// tui_pantry::run!()
/// ```
///
/// **With ingredients** — caller supplies the ingredient list; styles
/// are still discovered from `pantry.toml` / `styles.toml`:
///
/// ```ignore
/// tui_pantry::run!(my_ingredients)
/// ```
#[macro_export]
macro_rules! run {
    () => {
        $crate::run($crate::pantry_ingredients!(), env!("CARGO_MANIFEST_DIR"))
    };
    ($ingredients:expr) => {
        $crate::run($ingredients, env!("CARGO_MANIFEST_DIR"))
    };
}

/// Boot the pantry harness with the given ingredients.
///
/// Reads `<manifest_dir>/pantry.toml` at startup (falling back to
/// `styles.toml`). If present, parses the stylesheet sections and
/// prepends the resulting color/typography ingredients. Prefer the
/// [`run!`] macro which captures the manifest directory automatically.
///
/// Takes ownership of the terminal for the duration. Restores
/// terminal state on exit (normal or panic).
pub fn run(ingredients: Vec<Box<dyn Ingredient>>, manifest_dir: &str) -> io::Result<()> {
    let base = std::path::Path::new(manifest_dir);

    let styles_content = std::fs::read_to_string(base.join("pantry.toml"))
        .or_else(|_| std::fs::read_to_string(base.join("styles.toml")));

    let (mut all, chrome) = match styles_content {
        Ok(ref content) => {
            let table: toml::Table = content.parse().expect("pantry.toml: invalid TOML");
            (
                stylesheet::from_toml(content),
                theme::PantryTheme::from_toml(&table),
            )
        }
        Err(_) => (Vec::new(), theme::PantryTheme::dark()),
    };
    all.extend(ingredients);

    let terminal = ratatui::init();
    ratatui::crossterm::execute!(
        std::io::stdout(),
        ratatui::crossterm::event::EnableMouseCapture
    )?;
    let result = app::App::new(all, chrome).run(terminal);
    let _ = ratatui::crossterm::execute!(
        std::io::stdout(),
        ratatui::crossterm::event::DisableMouseCapture
    );
    ratatui::restore();
    result
}
