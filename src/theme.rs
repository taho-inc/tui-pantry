use ratatui::style::Color;

/// Chrome color palette for the pantry UI.
///
/// Two built-in modes — `dark` (default) and `light` — with optional
/// per-field overrides from `[pantry.dark]` / `[pantry.light]` in
/// `pantry.toml`.
pub struct PantryTheme {
    pub accent: Color,
    pub panel_bg: Color,
    pub cursor_bg: Color,
    pub border: Color,
    pub border_dim: Color,
    pub text: Color,
    pub text_dim: Color,
    pub doc_accent: Color,
    pub doc_text: Color,
    pub doc_type: Color,
    pub indicator: Color,
    pub dark: bool,
}

impl PantryTheme {
    /// Catppuccin Mocha
    pub const fn dark() -> Self {
        Self {
            accent: Color::Rgb(245, 194, 231),     // Pink
            panel_bg: Color::Rgb(30, 30, 46),      // Base
            cursor_bg: Color::Rgb(49, 50, 68),     // Surface0
            border: Color::Rgb(116, 199, 236),     // Sapphire
            border_dim: Color::Rgb(49, 50, 68),    // Surface0
            text: Color::Rgb(180, 190, 254),       // Lavender
            text_dim: Color::Rgb(88, 91, 112),     // Surface2
            doc_accent: Color::Rgb(250, 179, 135), // Peach
            doc_text: Color::Rgb(166, 173, 200),   // Subtext0
            doc_type: Color::Rgb(148, 226, 213),   // Teal
            indicator: Color::Rgb(249, 226, 175),  // Yellow
            dark: true,
        }
    }

    /// Catppuccin Latte
    pub const fn light() -> Self {
        Self {
            accent: Color::Rgb(136, 57, 239),      // Mauve
            panel_bg: Color::Rgb(239, 241, 245),   // Base
            cursor_bg: Color::Rgb(204, 208, 218),  // Surface0
            border: Color::Rgb(114, 135, 253),     // Sapphire
            border_dim: Color::Rgb(204, 208, 218), // Surface0
            text: Color::Rgb(76, 79, 105),         // Text
            text_dim: Color::Rgb(156, 160, 176),   // Overlay0
            doc_accent: Color::Rgb(254, 100, 11),  // Peach
            doc_text: Color::Rgb(108, 111, 133),   // Subtext0
            doc_type: Color::Rgb(23, 146, 153),    // Teal
            indicator: Color::Rgb(223, 142, 29),   // Yellow
            dark: false,
        }
    }

    /// Apply overrides from a TOML sub-table. Missing keys keep defaults.
    fn with_overrides(mut self, overrides: &toml::Table) -> Self {
        use crate::stylesheet::parse_color;

        macro_rules! override_color {
            ($field:ident) => {
                if let Some(v) = overrides.get(stringify!($field)).and_then(|v| v.as_str()) {
                    self.$field = parse_color(v);
                }
            };
        }

        override_color!(accent);
        override_color!(panel_bg);
        override_color!(cursor_bg);
        override_color!(border);
        override_color!(border_dim);
        override_color!(text);
        override_color!(text_dim);
        override_color!(doc_accent);
        override_color!(doc_text);
        override_color!(doc_type);
        override_color!(indicator);

        self
    }
}

/// Both theme modes, parsed once from `pantry.toml`.
///
/// Stored in `App` so `t` can swap without re-parsing.
pub struct ThemePair {
    dark: PantryTheme,
    light: PantryTheme,
    start_dark: bool,
}

impl Default for ThemePair {
    fn default() -> Self {
        Self {
            dark: PantryTheme::dark(),
            light: PantryTheme::light(),
            start_dark: true,
        }
    }
}

impl ThemePair {
    /// Parse from a TOML table.
    ///
    /// Reads `[config].theme` for the initial mode, then layers
    /// `[pantry.dark]` / `[pantry.light]` overrides onto Catppuccin defaults.
    pub fn from_toml(table: &toml::Table) -> Self {
        let start_dark = !matches!(
            table
                .get("config")
                .and_then(|v| v.as_table())
                .and_then(|c| c.get("theme"))
                .and_then(|v| v.as_str())
                .unwrap_or("dark")
                .to_ascii_lowercase()
                .as_str(),
            "light"
        );

        let pantry = table.get("pantry").and_then(|v| v.as_table());

        let resolve = |base: PantryTheme, key: &str| match pantry
            .and_then(|p| p.get(key))
            .and_then(|v| v.as_table())
        {
            Some(overrides) => base.with_overrides(overrides),
            None => base,
        };

        Self {
            dark: resolve(PantryTheme::dark(), "dark"),
            light: resolve(PantryTheme::light(), "light"),
            start_dark,
        }
    }

    pub fn start_dark(&self) -> bool {
        self.start_dark
    }

    pub fn get(&self, dark: bool) -> &PantryTheme {
        if dark { &self.dark } else { &self.light }
    }
}

/// Named preview background colors parsed from `[pantry.preview_backgrounds]`.
#[derive(Default)]
pub struct PreviewBackgrounds {
    entries: Vec<(String, Color)>,
}

impl PreviewBackgrounds {
    pub fn from_toml(table: &toml::Table) -> Self {
        use crate::stylesheet::parse_color;

        let entries = table
            .get("pantry")
            .and_then(|v| v.as_table())
            .and_then(|p| p.get("preview_backgrounds"))
            .and_then(|v| v.as_table())
            .map(|bg| {
                bg.iter()
                    .map(|(name, val)| {
                        let hex = val
                            .as_str()
                            .expect("preview_backgrounds values must be color strings");
                        (name.clone(), parse_color(hex))
                    })
                    .collect()
            })
            .unwrap_or_default();

        Self { entries }
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<(&str, Color)> {
        self.entries.get(index).map(|(n, c)| (n.as_str(), *c))
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_switches_mode() {
        let pair = ThemePair::default();
        assert!(pair.get(true).dark);
        assert!(!pair.get(false).dark);
    }

    #[test]
    fn from_toml_defaults_to_dark() {
        let table: toml::Table = "".parse().unwrap();
        assert!(ThemePair::from_toml(&table).start_dark());
    }

    #[test]
    fn from_toml_light() {
        let table: toml::Table = r#"[config]
theme = "light""#
            .parse()
            .unwrap();
        assert!(!ThemePair::from_toml(&table).start_dark());
    }

    #[test]
    fn from_toml_case_insensitive() {
        let table: toml::Table = r#"[config]
theme = "LIGHT""#
            .parse()
            .unwrap();
        assert!(!ThemePair::from_toml(&table).start_dark());
    }

    #[test]
    fn from_toml_unknown_defaults_to_dark() {
        let table: toml::Table = r#"[config]
theme = "solarized""#
            .parse()
            .unwrap();
        assert!(ThemePair::from_toml(&table).start_dark());
    }

    #[test]
    fn pantry_overrides_dark_accent() {
        let table: toml::Table = r##"
[pantry.dark]
accent = "#FF0000"
"##
        .parse()
        .unwrap();

        let pair = ThemePair::from_toml(&table);
        assert_eq!(pair.get(true).accent, Color::Rgb(255, 0, 0));
        // Light mode unchanged
        assert_eq!(pair.get(false).accent, PantryTheme::light().accent);
    }

    #[test]
    fn pantry_overrides_both_modes() {
        let table: toml::Table = r##"
[pantry.dark]
text = "#AABBCC"

[pantry.light]
text = "#112233"
"##
        .parse()
        .unwrap();

        let pair = ThemePair::from_toml(&table);
        assert_eq!(pair.get(true).text, Color::Rgb(0xAA, 0xBB, 0xCC));
        assert_eq!(pair.get(false).text, Color::Rgb(0x11, 0x22, 0x33));
    }

    #[test]
    fn pantry_partial_override_keeps_defaults() {
        let table: toml::Table = r##"
[pantry.dark]
accent = "#00FF00"
"##
        .parse()
        .unwrap();

        let pair = ThemePair::from_toml(&table);
        let theme = pair.get(true);
        assert_eq!(theme.accent, Color::Rgb(0, 255, 0));
        assert_eq!(theme.panel_bg, PantryTheme::dark().panel_bg);
        assert_eq!(theme.border, PantryTheme::dark().border);
    }

    #[test]
    fn preview_backgrounds_empty_by_default() {
        let table: toml::Table = "".parse().unwrap();
        assert!(PreviewBackgrounds::from_toml(&table).is_empty());
    }

    #[test]
    fn preview_backgrounds_parsed() {
        let table: toml::Table = r##"
[pantry.preview_backgrounds]
dark = "#0D0623"
light = "#F4F3F8"
"##
        .parse()
        .unwrap();

        let bgs = PreviewBackgrounds::from_toml(&table);
        assert_eq!(bgs.len(), 2);

        let (name, _color) = bgs.get(0).unwrap();
        assert!(!name.is_empty());
    }
}
