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
    /// Background gradient left endpoint.
    pub gradient_left: (f32, f32, f32),
    /// Background gradient right endpoint.
    pub gradient_right: (f32, f32, f32),
}

impl PantryTheme {
    pub const fn dark() -> Self {
        Self {
            accent: Color::Rgb(120, 52, 245),
            panel_bg: Color::Rgb(13, 13, 13),
            cursor_bg: Color::Rgb(30, 14, 58),
            border: Color::Rgb(61, 61, 61),
            text: Color::White,
            text_dim: Color::DarkGray,
            gradient_left: (120.0, 52.0, 245.0),
            gradient_right: (46.0, 21.0, 116.0),
        }
    }

    pub const fn light() -> Self {
        Self {
            accent: Color::Rgb(120, 52, 245),
            panel_bg: Color::Rgb(245, 245, 245),
            cursor_bg: Color::Rgb(237, 229, 252),
            border: Color::Rgb(212, 212, 212),
            text: Color::Rgb(30, 30, 46),
            text_dim: Color::Rgb(136, 136, 136),
            gradient_left: (200.0, 180.0, 245.0),
            gradient_right: (220.0, 210.0, 250.0),
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
