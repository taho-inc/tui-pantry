use ratatui::style::Color;

use super::palette::{accent, surface, text};

/// Semantic color mapping. Widgets reference roles, not raw colors.
pub struct Theme {
    pub primary: Color,
    pub accent: Color,

    pub text: Color,
    pub text_dim: Color,
    pub text_disabled: Color,

    pub surface: Color,
    pub surface_raised: Color,
    pub border: Color,
    pub border_focus: Color,

    pub ok: Color,
    pub warn: Color,
    pub critical: Color,
    pub info: Color,
}

impl Theme {
    pub const fn mocha() -> Self {
        Self {
            primary: accent::BLUE,
            accent: accent::MAUVE,

            text: text::TEXT,
            text_dim: text::SUBTEXT1,
            text_disabled: surface::OVERLAY0,

            surface: surface::BASE,
            surface_raised: surface::SURFACE0,
            border: surface::SURFACE1,
            border_focus: accent::MAUVE,

            ok: accent::GREEN,
            warn: accent::YELLOW,
            critical: accent::RED,
            info: accent::BLUE,
        }
    }

    /// Map a 0.0–1.0 ratio to a state color using threshold bands.
    pub const fn ratio_color(&self, ratio: f32) -> Color {
        if ratio >= 0.85 {
            self.critical
        } else if ratio >= 0.70 {
            self.warn
        } else {
            self.ok
        }
    }

    /// Map a 0–100 percent to a state color using threshold bands.
    pub const fn percent_color(&self, percent: u8) -> Color {
        if percent >= 85 {
            self.critical
        } else if percent >= 70 {
            self.warn
        } else {
            self.ok
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::mocha()
    }
}

pub const MOCHA: Theme = Theme::mocha();
