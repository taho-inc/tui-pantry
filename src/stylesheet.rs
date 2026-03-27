//! TOML-driven style sheet ingredients.
//!
//! Consumers declare their design system in TOML and the pantry renders
//! color swatches and typography samples automatically. Keys use
//! Rust-friendly snake_case identifiers; display names are derived at
//! runtime via [`display_name`].
//!
//! ```toml
//! [config]
//! style_source = "my_crate::styles"
//!
//! # Theme-independent raw pigments
//! [palette]
//! vivid_purple = "#7834F5"
//! mint = "#44BBA4"
//!
//! # Themed sections — each sub-table becomes a color or content group
//! [dark.brand]
//! primary = "#7834F5"
//! secondary = "#2E1574"
//!
//! [dark.content]
//! primary = { color = "#FFFFFF", description = "Headings, body, values" }
//!
//! [light.brand]
//! primary = "#7834F5"
//! secondary = "#EDE5FC"
//! ```

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};

use crate::Ingredient;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Convert a snake_case identifier to a Title Case display name.
///
/// Splits on `_`, capitalises the first character of each word, and
/// joins with spaces. Numeric-only tokens (scale keys like `500`) pass
/// through unchanged.
fn display_name(ident: &str) -> String {
    ident
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    let mut out = first.to_uppercase().to_string();
                    out.push_str(chars.as_str());
                    out
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Parse a TOML style sheet and return one [`Ingredient`] per color group
/// plus one for each content/typography section.
///
/// Supports three layouts:
/// - `[palette]` — flat color table, group = "Palette"
/// - `[colors.*]` — legacy per-family color tables, group = "Colors"
/// - `[dark.*]` / `[light.*]` — themed sections auto-detecting color vs content
/// - `[typography]` — legacy typography, group = "Typography"
///
/// Panics on malformed TOML or invalid color strings — this is a dev tool
/// fed by `include_str!`, so errors surface immediately.
pub fn from_toml(toml_str: &str) -> Vec<Box<dyn Ingredient>> {
    let table: toml::Table = toml_str.parse().expect("stylesheet: invalid TOML");

    let source = table
        .get("config")
        .and_then(|v| v.as_table())
        .and_then(|c| c.get("style_source"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_owned();

    let mut ingredients: Vec<Box<dyn Ingredient>> = Vec::new();

    // [palette] — flat theme-independent pigments
    if let Some(palette) = table.get("palette").and_then(|v| v.as_table()) {
        let entries = parse_color_entries(palette);
        if !entries.is_empty() {
            ingredients.push(Box::new(ColorGroupIngredient {
                group: "Palette".to_owned(),
                name: "Colors".to_owned(),
                source: source_path(&source, "palette"),
                mode: swatch_mode(&entries),
                entries,
            }));
        }
    }

    // [colors.*] — legacy per-family color tables
    if let Some(colors) = table.get("colors").and_then(|v| v.as_table()) {
        for (group_name, group_val) in colors {
            let group_table = group_val
                .as_table()
                .expect("colors group should be a table");
            let entries = parse_color_entries(group_table);
            ingredients.push(Box::new(ColorGroupIngredient {
                group: "Colors".to_owned(),
                name: display_name(group_name),
                source: source_path(&source, group_name),
                mode: swatch_mode(&entries),
                entries,
            }));
        }
    }

    // [dark.*] / [light.*] — themed sections
    for theme_key in ["dark", "light"] {
        if let Some(theme_table) = table.get(theme_key).and_then(|v| v.as_table()) {
            let group = display_name(theme_key);
            ingredients.extend(parse_themed_section(&group, theme_table, &source));
        }
    }

    // [typography] — legacy top-level typography
    if let Some(typography) = table.get("typography").and_then(|v| v.as_table()) {
        let levels = parse_content_levels(typography);
        if !levels.is_empty() {
            ingredients.push(Box::new(ContentIngredient {
                group: "Typography".to_owned(),
                name: "Text Hierarchy".to_owned(),
                source: source_path(&source, "typography"),
                levels,
            }));
        }
    }

    ingredients
}

// ---------------------------------------------------------------------------
// Parsing helpers
// ---------------------------------------------------------------------------

fn source_path(prefix: &str, segment: &str) -> String {
    let slug = segment.to_lowercase().replace(' ', "_");
    if prefix.is_empty() {
        slug
    } else {
        format!("{prefix}::{slug}")
    }
}

fn parse_color_entries(table: &toml::Table) -> Vec<ColorEntry> {
    table
        .iter()
        .map(|(label, val)| {
            let hex = val.as_str().expect("color value should be a string");
            ColorEntry {
                label: display_name(label),
                hex: hex.to_owned(),
                color: parse_color(hex),
            }
        })
        .collect()
}

fn parse_content_levels(table: &toml::Table) -> Vec<TextLevel> {
    table
        .iter()
        .map(|(name, val)| {
            let t = val.as_table().expect("content entry should be a table");
            let color_str = t
                .get("color")
                .and_then(|v| v.as_str())
                .expect("content entry should have a color field");
            let description = t.get("description").and_then(|v| v.as_str()).unwrap_or("");
            TextLevel {
                name: display_name(name),
                description: description.to_owned(),
                color: parse_color(color_str),
            }
        })
        .collect()
}

fn swatch_mode(entries: &[ColorEntry]) -> SwatchMode {
    if !entries.is_empty() && entries.iter().all(|e| e.label.parse::<u32>().is_ok()) {
        SwatchMode::Scale
    } else {
        SwatchMode::Named
    }
}

/// Auto-detect sub-tables as color groups or content (typography) groups.
///
/// A sub-table whose values are inline tables with a `color` key is
/// treated as content; otherwise it's a color group.
fn parse_themed_section(
    group: &str,
    theme_table: &toml::Table,
    source: &str,
) -> Vec<Box<dyn Ingredient>> {
    let mut out: Vec<Box<dyn Ingredient>> = Vec::new();

    for (section_name, section_val) in theme_table {
        let section_table = section_val
            .as_table()
            .unwrap_or_else(|| panic!("{group}.{section_name} should be a table"));

        let is_content = section_table
            .values()
            .any(|v| v.as_table().is_some_and(|t| t.contains_key("color")));

        if is_content {
            let levels = parse_content_levels(section_table);
            out.push(Box::new(ContentIngredient {
                group: group.to_owned(),
                name: display_name(section_name),
                source: source_path(source, section_name),
                levels,
            }));
        } else {
            let entries = parse_color_entries(section_table);
            out.push(Box::new(ColorGroupIngredient {
                group: group.to_owned(),
                name: display_name(section_name),
                source: source_path(source, section_name),
                mode: swatch_mode(&entries),
                entries,
            }));
        }
    }

    out
}

// ---------------------------------------------------------------------------
// Color parsing
// ---------------------------------------------------------------------------

pub(crate) fn parse_color(s: &str) -> Color {
    if let Some(hex) = s.strip_prefix('#') {
        let bytes = u32::from_str_radix(hex, 16).expect("should be valid hex color");
        Color::Rgb((bytes >> 16) as u8, (bytes >> 8) as u8, bytes as u8)
    } else {
        match s {
            "Black" => Color::Black,
            "Red" => Color::Red,
            "Green" => Color::Green,
            "Yellow" => Color::Yellow,
            "Blue" => Color::Blue,
            "Magenta" => Color::Magenta,
            "Cyan" => Color::Cyan,
            "Gray" => Color::Gray,
            "DarkGray" => Color::DarkGray,
            "LightRed" => Color::LightRed,
            "LightGreen" => Color::LightGreen,
            "LightYellow" => Color::LightYellow,
            "LightBlue" => Color::LightBlue,
            "LightMagenta" => Color::LightMagenta,
            "LightCyan" => Color::LightCyan,
            "White" => Color::White,
            _ => panic!("stylesheet: unknown color name \"{s}\""),
        }
    }
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct ColorEntry {
    label: String,
    hex: String,
    color: Color,
}

enum SwatchMode {
    Named,
    Scale,
}

struct ColorGroupIngredient {
    group: String,
    name: String,
    source: String,
    mode: SwatchMode,
    entries: Vec<ColorEntry>,
}

struct ContentIngredient {
    group: String,
    name: String,
    source: String,
    levels: Vec<TextLevel>,
}

#[derive(Clone)]
struct TextLevel {
    name: String,
    description: String,
    color: Color,
}

// ---------------------------------------------------------------------------
// Rendering helpers
// ---------------------------------------------------------------------------

/// Dim annotation style for hex values and secondary text.
const DIM: Style = Style::new().fg(Color::Gray);

/// Pick white or black text for readability on a given background.
fn contrasting_fg(bg: Color) -> Color {
    match bg {
        Color::Rgb(r, g, b) => {
            // Relative luminance (sRGB approximation)
            let lum = 0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64;
            if lum > 128.0 {
                Color::Rgb(0, 0, 0)
            } else {
                Color::Rgb(255, 255, 255)
            }
        }
        _ => Color::White,
    }
}

/// Swatch height in rows (3 color + 1 separator).
const SWATCH_ROWS: u16 = 3;
const SWATCH_STRIDE: u16 = SWATCH_ROWS + 1;
const SWATCH_COLS: &str = "              ";

fn render_named_swatches(entries: &[ColorEntry], area: Rect, buf: &mut Buffer) {
    let label_width = entries.iter().map(|e| e.label.len()).max().unwrap_or(0);

    for (i, entry) in entries.iter().enumerate() {
        let base_y = area.y + (i as u16) * SWATCH_STRIDE;
        if base_y + SWATCH_ROWS > area.bottom() {
            break;
        }

        let swatch = Span::styled(SWATCH_COLS, Style::new().bg(entry.color));

        // Top and bottom swatch rows
        Line::from(swatch.clone()).render(
            Rect {
                y: base_y,
                height: 1,
                ..area
            },
            buf,
        );
        Line::from(swatch.clone()).render(
            Rect {
                y: base_y + 2,
                height: 1,
                ..area
            },
            buf,
        );

        // Middle row: swatch + label + hex
        let label_fg = contrasting_fg(entry.color);
        let mid = Line::from(vec![
            swatch,
            Span::raw("  "),
            Span::styled(
                format!("{:<width$}", entry.label, width = label_width),
                Style::new().fg(label_fg).add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(&entry.hex, DIM),
        ]);
        mid.render(
            Rect {
                y: base_y + 1,
                height: 1,
                ..area
            },
            buf,
        );
    }
}

fn render_scale_swatches(entries: &[ColorEntry], area: Rect, buf: &mut Buffer) {
    if entries.is_empty() || area.height < SWATCH_ROWS || area.width < 4 {
        return;
    }

    let n = entries.len() as u16;
    let step_width = area.width / n;
    if step_width == 0 {
        return;
    }

    let step_w = |i: u16| -> u16 {
        if i == n - 1 {
            area.width - i * step_width
        } else {
            step_width
        }
    };

    // 3-row color blocks
    for row in 0..SWATCH_ROWS {
        let y = area.y + row;
        for (i, entry) in entries.iter().enumerate() {
            let x = area.x + (i as u16) * step_width;
            let w = step_w(i as u16);
            let block = " ".repeat(w as usize);
            Line::from(Span::styled(block, Style::new().bg(entry.color))).render(
                Rect {
                    x,
                    y,
                    width: w,
                    height: 1,
                },
                buf,
            );
        }
    }

    // Labels below the swatch block
    let label_y = area.y + SWATCH_ROWS;
    if label_y < area.bottom() {
        for (i, entry) in entries.iter().enumerate() {
            let x = area.x + (i as u16) * step_width;
            let w = step_w(i as u16);
            Line::from(Span::styled(entry.label.as_str(), DIM)).render(
                Rect {
                    x,
                    y: label_y,
                    width: w,
                    height: 1,
                },
                buf,
            );
        }
    }

    let hex_y = label_y + 1;
    if hex_y < area.bottom() {
        for (i, entry) in entries.iter().enumerate() {
            let x = area.x + (i as u16) * step_width;
            let w = step_w(i as u16);
            Line::from(Span::styled(entry.hex.as_str(), DIM)).render(
                Rect {
                    x,
                    y: hex_y,
                    width: w,
                    height: 1,
                },
                buf,
            );
        }
    }
}

fn render_content(levels: &[TextLevel], area: Rect, buf: &mut Buffer) {
    let name_width = levels.iter().map(|l| l.name.len()).max().unwrap_or(0);

    for (i, level) in levels.iter().enumerate() {
        let y = area.y + (i as u16) * 2;
        if y >= area.bottom() {
            break;
        }

        let line = Line::from(vec![
            Span::styled(
                format!("  {:<width$}", level.name, width = name_width),
                Style::new().fg(level.color).add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(&level.description, Style::new().fg(level.color)),
        ]);
        line.render(
            Rect {
                y,
                height: 1,
                ..area
            },
            buf,
        );
    }
}

// ---------------------------------------------------------------------------
// Ingredient impls
// ---------------------------------------------------------------------------

impl Ingredient for ColorGroupIngredient {
    fn tab(&self) -> &str {
        "Styles"
    }
    fn group(&self) -> &str {
        &self.group
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn source(&self) -> &str {
        &self.source
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        match self.mode {
            SwatchMode::Named => render_named_swatches(&self.entries, area, buf),
            SwatchMode::Scale => render_scale_swatches(&self.entries, area, buf),
        }
    }
}

impl Ingredient for ContentIngredient {
    fn tab(&self) -> &str {
        "Styles"
    }
    fn group(&self) -> &str {
        &self.group
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn source(&self) -> &str {
        &self.source
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        render_content(&self.levels, area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- parse_color ----------------------------------------------------------

    #[test]
    fn parse_hex_color() {
        assert_eq!(parse_color("#FF8800"), Color::Rgb(255, 136, 0));
    }

    #[test]
    #[should_panic(expected = "unknown color name")]
    fn parse_unknown_color_panics() {
        parse_color("Chartreuse");
    }

    // -- from_toml integration ------------------------------------------------

    #[test]
    fn from_toml_palette_section() {
        let toml = r##"
            [palette]
            vivid_purple = "#7834F5"
            mint = "#44BBA4"
        "##;
        let ingredients = from_toml(toml);
        assert_eq!(ingredients.len(), 1);
        assert_eq!(ingredients[0].group(), "Palette");
        assert_eq!(ingredients[0].name(), "Colors");
        assert_eq!(ingredients[0].tab(), "Styles");
    }

    #[test]
    fn from_toml_colors_section() {
        let toml = r##"
            [colors.brand]
            primary = "#7834F5"
            secondary = "#2E1574"
        "##;
        let ingredients = from_toml(toml);
        assert_eq!(ingredients.len(), 1);
        assert_eq!(ingredients[0].group(), "Colors");
        assert_eq!(ingredients[0].name(), "Brand");
    }

    #[test]
    fn from_toml_themed_content_section() {
        let toml = r##"
            [dark.content]
            primary = { color = "#FFFFFF", description = "Headings" }
        "##;
        let ingredients = from_toml(toml);
        assert_eq!(ingredients.len(), 1);
        assert_eq!(ingredients[0].group(), "Dark");
        assert_eq!(ingredients[0].name(), "Content");
    }

    #[test]
    fn from_toml_typography_section() {
        let toml = r##"
            [typography]
            heading = { color = "#FFFFFF", description = "Page titles" }
            body = { color = "#CCCCCC", description = "Body text" }
        "##;
        let ingredients = from_toml(toml);
        assert_eq!(ingredients.len(), 1);
        assert_eq!(ingredients[0].group(), "Typography");
        assert_eq!(ingredients[0].name(), "Text Hierarchy");
    }

    #[test]
    fn from_toml_with_config_source() {
        let toml = r##"
            [config]
            style_source = "my_crate::styles"

            [palette]
            red = "#FF0000"
        "##;
        let ingredients = from_toml(toml);
        assert_eq!(ingredients[0].source(), "my_crate::styles::palette");
    }

    #[test]
    fn from_toml_empty() {
        let ingredients = from_toml("");
        assert!(ingredients.is_empty());
    }

    #[test]
    fn from_toml_multiple_sections() {
        let toml = r##"
            [palette]
            red = "#FF0000"

            [dark.brand]
            accent = "#00FF00"

            [light.brand]
            accent = "#0000FF"

            [typography]
            body = { color = "#FFF", description = "text" }
        "##;
        let ingredients = from_toml(toml);
        // palette(1) + dark.brand(1) + light.brand(1) + typography(1) = 4
        assert_eq!(ingredients.len(), 4);
    }
}
