use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
    widgets::Widget,
};

/// Background gradient from #7834F5 (left) to #2E1574 (right).
///
/// Uses half-block characters (`▀`) for 2x vertical color resolution.
pub(crate) struct PurpleSwatch;

// #7834F5
const LEFT: (f32, f32, f32) = (120.0, 52.0, 245.0);
// #2E1574
const RIGHT: (f32, f32, f32) = (46.0, 21.0, 116.0);

impl Widget for PurpleSwatch {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width == 0 || area.height == 0 {
            return;
        }

        let w = f32::from(area.width);
        let h = f32::from(area.height);

        for (yi, y) in (area.top()..area.bottom()).enumerate() {
            let vy_fg = 1.0 - (yi as f32 / h);
            let vy_bg = 1.0 - ((yi as f32 + 0.5) / h);

            for (xi, x) in (area.left()..area.right()).enumerate() {
                let vx = xi as f32 / w;
                let fg = purple_at(vx, vy_fg);
                let bg = purple_at(vx, vy_bg);
                buf[(x, y)].set_char('▀').set_fg(fg).set_bg(bg);
            }
        }
    }
}

/// Lerp between LEFT and RIGHT along x, dim slightly toward the bottom along y.
fn purple_at(x: f32, y: f32) -> Color {
    // y: 1.0 at top (full brightness), 0.0 at bottom (dimmed)
    let brightness = 0.7 + (y * 0.3);

    let r = lerp(LEFT.0, RIGHT.0, x) * brightness;
    let g = lerp(LEFT.1, RIGHT.1, x) * brightness;
    let b = lerp(LEFT.2, RIGHT.2, x) * brightness;

    Color::Rgb(r as u8, g as u8, b as u8)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
