mod app;
mod color_depth;
mod dump;
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
/// **No-args form** — reads `[ingredients]` from `pantry.toml` via
/// proc macro and discovers styles at runtime:
///
/// ```ignore
/// tui_pantry::run!()
/// ```
///
/// **With ingredients** — caller supplies the ingredient list; styles
/// are still discovered from `pantry.toml`:
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
/// Reads `<manifest_dir>/pantry.toml` at startup. If present, parses
/// the stylesheet sections and prepends the resulting color/typography
/// ingredients. Prefer the [`run!`] macro which captures the manifest
/// directory automatically.
///
/// In headless mode (`--dump` or `--list` in argv), renders to stdout
/// and returns without touching the terminal. Otherwise, takes
/// ownership of the terminal for the duration and restores state on
/// exit (normal or panic).
pub fn run(ingredients: Vec<Box<dyn Ingredient>>, manifest_dir: &str) -> io::Result<()> {
    let base = std::path::Path::new(manifest_dir);

    let styles_content = std::fs::read_to_string(base.join("pantry.toml"));

    let (mut all, themes, preview_backgrounds) = match styles_content {
        Ok(ref content) => {
            let table: toml::Table = content.parse().expect("pantry.toml: invalid TOML");
            (
                stylesheet::from_toml(content),
                theme::ThemePair::from_toml(&table),
                theme::PreviewBackgrounds::from_toml(&table),
            )
        }
        Err(_) => (
            Vec::new(),
            theme::ThemePair::default(),
            theme::PreviewBackgrounds::default(),
        ),
    };
    all.extend(ingredients);

    if let Some(action) = dump::parse_headless_args() {
        return match action {
            dump::HeadlessAction::List => dump::list(&all),
            dump::HeadlessAction::Dump(args) => dump::dump(&all, &args),
        };
    }

    let terminal = ratatui::init();
    ratatui::crossterm::execute!(
        std::io::stdout(),
        ratatui::crossterm::event::EnableMouseCapture
    )?;
    let result = app::App::new(all, themes, preview_backgrounds).run(terminal);
    let _ = ratatui::crossterm::execute!(
        std::io::stdout(),
        ratatui::crossterm::event::DisableMouseCapture
    );
    ratatui::restore();
    result
}
