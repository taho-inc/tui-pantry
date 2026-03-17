use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Style,
    widgets::{Block, Gauge, Widget},
};
use tui_pantry::{Ingredient, layout::render_centered};

use crate::styles::MOCHA;

pub mod ingredient {
    use super::*;

    pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
        vec![
            Box::new(ResourceGaugesHealthy),
            Box::new(ResourceGaugesStressed),
        ]
    }
}

struct Metric {
    label: &'static str,
    ratio: f64,
}

fn render_gauges(metrics: &[Metric], area: Rect, buf: &mut Buffer) {
    let cols = Layout::horizontal(vec![Constraint::Ratio(1, metrics.len() as u32); metrics.len()])
        .split(area);

    for (metric, col) in metrics.iter().zip(cols.iter()) {
        let color = MOCHA.ratio_color(metric.ratio as f32);
        Gauge::default()
            .block(
                Block::bordered()
                    .title(format!(" {} ", metric.label))
                    .title_style(Style::default().fg(MOCHA.text))
                    .border_style(Style::default().fg(MOCHA.border)),
            )
            .gauge_style(Style::default().fg(color).bg(MOCHA.surface_raised))
            .ratio(metric.ratio)
            .label(format!("{:.0}%", metric.ratio * 100.0))
            .render(*col, buf);
    }
}

// ── Healthy ──

struct ResourceGaugesHealthy;

const HEALTHY: &[Metric] = &[
    Metric { label: "CPU", ratio: 0.34 },
    Metric { label: "Memory", ratio: 0.52 },
    Metric { label: "Disk", ratio: 0.41 },
];

impl Ingredient for ResourceGaugesHealthy {
    fn tab(&self) -> &str { "Panes" }
    fn group(&self) -> &str { "Resource Gauges" }
    fn name(&self) -> &str { "Healthy" }
    fn source(&self) -> &str { "example_pantry::panes::resource_gauges" }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        render_centered(GaugeStrip(HEALTHY), Some(Constraint::Max(60)), Some(Constraint::Length(3)), area, buf);
    }
}

// ── Stressed ──

struct ResourceGaugesStressed;

const STRESSED: &[Metric] = &[
    Metric { label: "CPU", ratio: 0.92 },
    Metric { label: "Memory", ratio: 0.87 },
    Metric { label: "Disk", ratio: 0.76 },
];

impl Ingredient for ResourceGaugesStressed {
    fn tab(&self) -> &str { "Panes" }
    fn group(&self) -> &str { "Resource Gauges" }
    fn name(&self) -> &str { "Stressed" }
    fn source(&self) -> &str { "example_pantry::panes::resource_gauges" }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        render_centered(GaugeStrip(STRESSED), Some(Constraint::Max(60)), Some(Constraint::Length(3)), area, buf);
    }
}

struct GaugeStrip(&'static [Metric]);

impl Widget for GaugeStrip {
    fn render(self, area: Rect, buf: &mut Buffer) {
        render_gauges(self.0, area, buf);
    }
}
