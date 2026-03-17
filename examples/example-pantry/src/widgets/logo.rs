use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    widgets::{RatatuiLogo, RatatuiLogoSize, RatatuiMascot},
};
use tui_pantry::{Ingredient, layout::render_centered};

pub mod ingredient {
    use super::*;

    pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
        vec![
            Box::new(LogoSmall),
            Box::new(LogoTiny),
            Box::new(Mascot),
        ]
    }
}

// ── Logo Small ──

struct LogoSmall;

impl Ingredient for LogoSmall {
    fn group(&self) -> &str { "Logo & Mascot" }
    fn name(&self) -> &str { "Logo (small)" }
    fn source(&self) -> &str { "ratatui::widgets::RatatuiLogo" }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        render_centered(
            RatatuiLogo::new(RatatuiLogoSize::Small),
            Some(Constraint::Length(32)),
            Some(Constraint::Length(2)),
            area,
            buf,
        );
    }
}

// ── Logo Tiny ──

struct LogoTiny;

impl Ingredient for LogoTiny {
    fn group(&self) -> &str { "Logo & Mascot" }
    fn name(&self) -> &str { "Logo (tiny)" }
    fn source(&self) -> &str { "ratatui::widgets::RatatuiLogo" }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        render_centered(
            RatatuiLogo::new(RatatuiLogoSize::Tiny),
            Some(Constraint::Length(16)),
            Some(Constraint::Length(1)),
            area,
            buf,
        );
    }
}

// ── Mascot ──

struct Mascot;

impl Ingredient for Mascot {
    fn group(&self) -> &str { "Logo & Mascot" }
    fn name(&self) -> &str { "Mascot" }
    fn source(&self) -> &str { "ratatui::widgets::RatatuiMascot" }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        render_centered(
            RatatuiMascot::default(),
            Some(Constraint::Length(32)),
            Some(Constraint::Length(16)),
            area,
            buf,
        );
    }
}
