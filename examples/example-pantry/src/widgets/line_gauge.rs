use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Style,
    widgets::{Block, LineGauge as RatatuiLineGauge, Widget},
};
use tui_pantry::Ingredient;

use crate::styles::MOCHA;

pub mod ingredient {
    use super::*;

    pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
        vec![
            Box::new(LineGaugeLow),
            Box::new(LineGaugeMedium),
            Box::new(LineGaugeHigh),
        ]
    }
}

fn render_line_gauge(label: &str, ratio: f64, area: Rect, buf: &mut Buffer) {
    let [_, row, _] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(3),
        Constraint::Fill(1),
    ])
    .areas(area);

    let [_, col, _] = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(40),
        Constraint::Fill(1),
    ])
    .areas(row);

    let color = MOCHA.ratio_color(ratio as f32);

    RatatuiLineGauge::default()
        .block(
            Block::bordered()
                .title(format!(" {label} "))
                .title_style(Style::default().fg(MOCHA.text))
                .border_style(Style::default().fg(MOCHA.border)),
        )
        .filled_style(Style::default().fg(color).bg(MOCHA.surface_raised))
        .unfilled_style(Style::default().fg(MOCHA.border).bg(MOCHA.surface))
        .ratio(ratio)
        .label(format!("{:.0}%", ratio * 100.0))
        .render(col, buf);
}

// ── Low ──

struct LineGaugeLow;

impl Ingredient for LineGaugeLow {
    fn section(&self) -> Option<&str> {
        Some("Charts")
    }

    fn group(&self) -> &str {
        "Line Gauge"
    }
    fn name(&self) -> &str {
        "Low (green)"
    }
    fn source(&self) -> &str {
        "ratatui::widgets::LineGauge"
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        render_line_gauge("Bandwidth", 0.34, area, buf);
    }
}

// ── Medium ──

struct LineGaugeMedium;

impl Ingredient for LineGaugeMedium {
    fn section(&self) -> Option<&str> {
        Some("Charts")
    }

    fn group(&self) -> &str {
        "Line Gauge"
    }
    fn name(&self) -> &str {
        "Medium (yellow)"
    }
    fn source(&self) -> &str {
        "ratatui::widgets::LineGauge"
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        render_line_gauge("Bandwidth", 0.74, area, buf);
    }
}

// ── High ──

struct LineGaugeHigh;

impl Ingredient for LineGaugeHigh {
    fn section(&self) -> Option<&str> {
        Some("Charts")
    }

    fn group(&self) -> &str {
        "Line Gauge"
    }
    fn name(&self) -> &str {
        "High (red)"
    }
    fn source(&self) -> &str {
        "ratatui::widgets::LineGauge"
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        render_line_gauge("Bandwidth", 0.92, area, buf);
    }
}
