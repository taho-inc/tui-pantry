//! TOML-driven style sheet ingredients.
//!
//! Consumers declare their design system in TOML and the pantry renders
//! color swatches and typography samples automatically. Keys use
//! Rust-friendly snake_case identifiers; display names are derived at
//! runtime via [`display_name`].
//!
//! ```toml
//! source = "my_crate::styles"
//!
//! [colors.brand]
//! deep_purple = "#2E1574"
//! white = "#FFFFFF"
//!
//! # Numeric keys render as a horizontal scale strip
//! [colors.green]
//! 100 = "#DCFCE7"
//! 500 = "#22C55E"
//! 900 = "#14532D"
//!
//! [typography]
//! text = { color = "#FFFFFF", description = "Primary content" }
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
/// plus one for typography (if declared).
///
/// Panics on malformed TOML or invalid color strings — this is a dev tool
/// fed by `include_str!`, so errors surface immediately.
pub fn from_toml(toml_str: &str) -> Vec<Box<dyn Ingredient>> {
    let table: toml::Table = toml_str
        .parse()
        .expect("stylesheet: invalid TOML");

    let source = table
        .get("config")
        .and_then(|v| v.as_table())
        .and_then(|c| c.get("style_source"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_owned();

    let mut ingredients: Vec<Box<dyn Ingredient>> = Vec::new();

    if let Some(colors) = table.get("colors").and_then(|v| v.as_table()) {
        for (group_name, group_val) in colors {
            let group_table = group_val
                .as_table()
                .expect("colors group should be a table");

            let entries: Vec<ColorEntry> = group_table
                .iter()
                .map(|(label, val)| {
                    let hex = val
                        .as_str()
                        .expect("color value should be a string");
                    ColorEntry {
                        label: display_name(label),
                        hex: hex.to_owned(),
                        color: parse_color(hex),
                    }
                })
                .collect();

            let is_scale = !entries.is_empty()
                && entries.iter().all(|e| e.label.parse::<u32>().is_ok());

            let source_path = if source.is_empty() {
                group_name.to_lowercase().replace(' ', "_")
            } else {
                format!("{}::{}", source, group_name.to_lowercase().replace(' ', "_"))
            };

            ingredients.push(Box::new(ColorGroupIngredient {
                name: display_name(group_name),
                source: source_path,
                mode: if is_scale { SwatchMode::Scale } else { SwatchMode::Named },
                entries,
            }));
        }
    }

    if let Some(typography) = table.get("typography").and_then(|v| v.as_table()) {
        let levels: Vec<TextLevel> = typography
            .iter()
            .map(|(name, val)| {
                let t = val
                    .as_table()
                    .expect("typography entry should be a table");
                let color_str = t
                    .get("color")
                    .and_then(|v| v.as_str())
                    .expect("typography entry should have a color field");
                let description = t
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                TextLevel {
                    name: display_name(name),
                    description: description.to_owned(),
                    color: parse_color(color_str),
                }
            })
            .collect();

        if !levels.is_empty() {
            let source_path = if source.is_empty() {
                "typography".to_owned()
            } else {
                format!("{source}::typography")
            };

            ingredients.push(Box::new(TypographyIngredient {
                source: source_path,
                levels,
            }));
        }
    }

    ingredients
}

// ---------------------------------------------------------------------------
// Color parsing
// ---------------------------------------------------------------------------

fn parse_color(s: &str) -> Color {
    if let Some(hex) = s.strip_prefix('#') {
        let bytes = u32::from_str_radix(hex, 16)
            .expect("should be valid hex color");
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
    name: String,
    source: String,
    mode: SwatchMode,
    entries: Vec<ColorEntry>,
}

struct TypographyIngredient {
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
const DIM: Style = Style::new().fg(Color::DarkGray);

/// Bold white for headings and labels.
const LABEL: Style = Style::new().fg(Color::White);

fn render_named_swatches(entries: &[ColorEntry], area: Rect, buf: &mut Buffer) {
    let label_width = entries.iter().map(|e| e.label.len()).max().unwrap_or(0);

    for (i, entry) in entries.iter().enumerate() {
        let y = area.y + i as u16;
        if y >= area.bottom() {
            break;
        }

        let line = Line::from(vec![
            Span::styled("  ████  ", Style::new().fg(entry.color)),
            Span::styled(
                format!("{:<width$}", entry.label, width = label_width),
                LABEL.add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(&entry.hex, DIM),
        ]);
        line.render(Rect { y, height: 1, ..area }, buf);
    }
}

fn render_scale_swatches(entries: &[ColorEntry], area: Rect, buf: &mut Buffer) {
    if entries.is_empty() || area.height < 2 || area.width < 4 {
        return;
    }

    let n = entries.len() as u16;
    let step_width = area.width / n;
    if step_width == 0 {
        return;
    }

    // Row 0: colored blocks
    for (i, entry) in entries.iter().enumerate() {
        let x = area.x + (i as u16) * step_width;
        let block_width = if i as u16 == n - 1 {
            area.width - (i as u16) * step_width
        } else {
            step_width
        };

        let block = "█".repeat(block_width as usize);
        let span = Span::styled(block, Style::new().fg(entry.color));
        buf.set_line(x, area.y, &Line::from(span), block_width);
    }

    // Row 1: step labels
    if area.height >= 2 {
        for (i, entry) in entries.iter().enumerate() {
            let x = area.x + (i as u16) * step_width;
            let w = if i as u16 == n - 1 {
                area.width - (i as u16) * step_width
            } else {
                step_width
            };
            let label = &entry.label;
            let span = Span::styled(label.as_str(), DIM);
            buf.set_line(x, area.y + 1, &Line::from(span), w);
        }
    }

    // Row 2: hex values
    if area.height >= 3 {
        for (i, entry) in entries.iter().enumerate() {
            let x = area.x + (i as u16) * step_width;
            let w = if i as u16 == n - 1 {
                area.width - (i as u16) * step_width
            } else {
                step_width
            };
            let span = Span::styled(entry.hex.as_str(), DIM);
            buf.set_line(x, area.y + 2, &Line::from(span), w);
        }
    }
}

fn render_typography(levels: &[TextLevel], area: Rect, buf: &mut Buffer) {
    let name_width = levels.iter().map(|l| l.name.len()).max().unwrap_or(0);

    for (i, level) in levels.iter().enumerate() {
        // Each level gets two rows: sample line + blank separator.
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
        line.render(Rect { y, height: 1, ..area }, buf);
    }
}

// ---------------------------------------------------------------------------
// Ingredient impls
// ---------------------------------------------------------------------------

impl Ingredient for ColorGroupIngredient {
    fn tab(&self) -> &str { "Styles" }
    fn group(&self) -> &str { "Colors" }
    fn name(&self) -> &str { &self.name }
    fn source(&self) -> &str { &self.source }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        match self.mode {
            SwatchMode::Named => render_named_swatches(&self.entries, area, buf),
            SwatchMode::Scale => render_scale_swatches(&self.entries, area, buf),
        }
    }
}

impl Ingredient for TypographyIngredient {
    fn tab(&self) -> &str { "Styles" }
    fn group(&self) -> &str { "Typography" }
    fn name(&self) -> &str { "Text Hierarchy" }
    fn source(&self) -> &str { &self.source }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        render_typography(&self.levels, area, buf);
    }
}
