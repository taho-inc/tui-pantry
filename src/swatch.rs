use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
    widgets::Widget,
};

/// Background gradient between two RGB endpoints.
///
/// Uses half-block characters (`▀`) for 2x vertical color resolution.
/// Brightness dims slightly toward the bottom for depth.
pub(crate) struct GradientSwatch {
    left: (f32, f32, f32),
    right: (f32, f32, f32),
}

impl GradientSwatch {
    pub(crate) fn new(left: (f32, f32, f32), right: (f32, f32, f32)) -> Self {
        Self { left, right }
    }
}

impl Widget for GradientSwatch {
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
                let fg = color_at(self.left, self.right, vx, vy_fg);
                let bg = color_at(self.left, self.right, vx, vy_bg);
                buf[(x, y)].set_char('▀').set_fg(fg).set_bg(bg);
            }
        }
    }
}

/// Lerp between left and right along x, dim slightly toward the bottom along y.
fn color_at(left: (f32, f32, f32), right: (f32, f32, f32), x: f32, y: f32) -> Color {
    let brightness = 0.7 + (y * 0.3);
    let r = lerp(left.0, right.0, x) * brightness;
    let g = lerp(left.1, right.1, x) * brightness;
    let b = lerp(left.2, right.2, x) * brightness;
    Color::Rgb(r as u8, g as u8, b as u8)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
