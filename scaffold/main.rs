mod panes;
mod views;
mod widgets;

use tui_pantry::ratatui::style::Color;

// Shared palette — replace with your own theme colors.
pub const TEXT: Color = Color::White;
pub const TEXT_DIM: Color = Color::DarkGray;
pub const BORDER: Color = Color::Gray;
pub const GREEN: Color = Color::Green;
pub const YELLOW: Color = Color::Yellow;
pub const RED: Color = Color::Red;
pub const BLUE: Color = Color::Blue;

fn main() -> std::io::Result<()> {
    let mut all = Vec::new();
    all.extend(widgets::ingredients());
    all.extend(panes::ingredients());
    all.extend(views::ingredients());
    tui_pantry::run!(all)
}
