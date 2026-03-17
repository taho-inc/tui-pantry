use ratatui::style::Color;

/// Chrome color palette for the pantry harness.
///
/// Two built-in modes — `dark` (default) and `light` — selected via
/// `[config] theme` in `pantry.toml`.
pub struct PantryTheme {
    pub accent: Color,
    pub panel_bg: Color,
    pub cursor_bg: Color,
    pub border: Color,
    pub text: Color,
    pub text_dim: Color,
    /// Whether this is dark mode.
    pub dark: bool,
}

impl PantryTheme {
    /// Catppuccin Mocha
    pub const fn dark() -> Self {
        Self {
            accent: Color::Rgb(203, 166, 247),   // Mauve
            panel_bg: Color::Rgb(30, 30, 46),    // Base
            cursor_bg: Color::Rgb(49, 50, 68),   // Surface0
            border: Color::Rgb(69, 71, 90),      // Surface1
            text: Color::Rgb(205, 214, 244),     // Text
            text_dim: Color::Rgb(108, 112, 134), // Overlay0
            dark: true,
        }
    }

    /// Catppuccin Latte
    pub const fn light() -> Self {
        Self {
            accent: Color::Rgb(136, 57, 239),     // Mauve
            panel_bg: Color::Rgb(239, 241, 245),  // Base
            cursor_bg: Color::Rgb(204, 208, 218), // Surface0
            border: Color::Rgb(188, 192, 204),    // Surface1
            text: Color::Rgb(76, 79, 105),        // Text
            text_dim: Color::Rgb(108, 111, 133),  // Subtext0
            dark: false,
        }
    }

    pub fn toggle(&self) -> Self {
        if self.dark {
            Self::light()
        } else {
            Self::dark()
        }
    }

    /// Parse from a TOML table. Reads `[config].theme` — `"light"` or
    /// `"dark"` (default when absent).
    pub fn from_toml(table: &toml::Table) -> Self {
        let mode = table
            .get("config")
            .and_then(|v| v.as_table())
            .and_then(|c| c.get("theme"))
            .and_then(|v| v.as_str())
            .unwrap_or("dark");

        match mode.to_ascii_lowercase().as_str() {
            "light" => Self::light(),
            _ => Self::dark(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_switches_mode() {
        let dark = PantryTheme::dark();
        let light = dark.toggle();
        assert!(!light.dark);
        let back = light.toggle();
        assert!(back.dark);
    }

    #[test]
    fn from_toml_defaults_to_dark() {
        let table: toml::Table = "".parse().unwrap();
        assert!(PantryTheme::from_toml(&table).dark);
    }

    #[test]
    fn from_toml_light() {
        let table: toml::Table = r#"[config]
theme = "light""#
            .parse()
            .unwrap();
        assert!(!PantryTheme::from_toml(&table).dark);
    }

    #[test]
    fn from_toml_case_insensitive() {
        let table: toml::Table = r#"[config]
theme = "LIGHT""#
            .parse()
            .unwrap();
        assert!(!PantryTheme::from_toml(&table).dark);
    }

    #[test]
    fn from_toml_unknown_defaults_to_dark() {
        let table: toml::Table = r#"[config]
theme = "solarized""#
            .parse()
            .unwrap();
        assert!(PantryTheme::from_toml(&table).dark);
    }
}
