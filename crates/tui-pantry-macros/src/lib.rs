//! Proc macros for `tui-pantry`. Not intended for direct use —
//! depend on `tui-pantry` which re-exports everything.

use proc_macro::TokenStream;

use quote::quote;

/// Reads `pantry.toml` from the consuming crate's manifest directory
/// and generates a `Vec<Box<dyn tui_pantry::Ingredient>>` that
/// aggregates every listed ingredient module.
///
/// The `[ingredients]` table expects:
/// - `source` — crate path prefix (e.g. `"taho_tui"`)
/// - `modules` — array of module paths relative to source
///
/// Each module is expanded to `{source}::{module}::ingredient::ingredients()`.
///
/// The generated code includes an `include_str!` of `pantry.toml` so
/// Cargo tracks it for rebuild-on-change.
#[proc_macro]
pub fn pantry_ingredients(_input: TokenStream) -> TokenStream {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR not set");

    let pantry_path = std::path::Path::new(&manifest_dir).join("pantry.toml");
    let content = std::fs::read_to_string(&pantry_path).unwrap_or_else(|e| {
        panic!(
            "tui-pantry: could not read {}: {e}",
            pantry_path.display()
        )
    });

    let table: toml::Table = content.parse().unwrap_or_else(|e| {
        panic!(
            "tui-pantry: invalid TOML in {}: {e}",
            pantry_path.display()
        )
    });

    let extends = parse_ingredients(&table);

    let output = quote! {
        {
            // Track pantry.toml for rebuild-on-change.
            const _: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/pantry.toml"));

            let mut __pantry_v: Vec<Box<dyn tui_pantry::Ingredient>> = Vec::new();
            #(#extends)*
            __pantry_v
        }
    };

    output.into()
}

/// Parse the `[ingredients]` table into extend statements.
///
/// Supports two forms:
///
/// Grouped (recommended):
/// ```toml
/// [ingredients]
/// source = "taho_tui"
/// modules = ["widgets::node_table", "widgets::node_card"]
/// ```
/// Expands each to `{source}::{module}::ingredient::ingredients()`.
///
/// Array of tables (multi-source):
/// ```toml
/// [[ingredients]]
/// source = "crate_a"
/// modules = ["widgets::foo"]
///
/// [[ingredients]]
/// source = "crate_b"
/// modules = ["widgets::bar"]
/// ```
fn parse_ingredients(table: &toml::Table) -> Vec<proc_macro2::TokenStream> {
    let Some(ingredients) = table.get("ingredients") else {
        return Vec::new();
    };

    match ingredients {
        toml::Value::Table(t) => parse_ingredient_group(t),
        toml::Value::Array(arr) => arr
            .iter()
            .flat_map(|v| {
                let t = v
                    .as_table()
                    .expect("tui-pantry: each [[ingredients]] entry must be a table");
                parse_ingredient_group(t)
            })
            .collect(),
        _ => panic!("tui-pantry: `ingredients` must be a table or array of tables"),
    }
}

/// Parse a single ingredient group with `source` and `modules`.
fn parse_ingredient_group(table: &toml::Table) -> Vec<proc_macro2::TokenStream> {
    let source = table
        .get("source")
        .and_then(|v| v.as_str())
        .expect("tui-pantry: ingredient group needs a `source` string");

    let modules = table
        .get("modules")
        .and_then(|v| v.as_array())
        .expect("tui-pantry: ingredient group needs a `modules` array");

    modules
        .iter()
        .map(|m| {
            let module = m
                .as_str()
                .expect("tui-pantry: each module entry must be a string");

            let full_path = format!("{source}::{module}::ingredient");
            let path: proc_macro2::TokenStream = full_path.parse().unwrap_or_else(|e| {
                panic!("tui-pantry: invalid module path \"{full_path}\": {e}")
            });

            quote! {
                __pantry_v.extend(#path::ingredients());
            }
        })
        .collect()
}
