use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Style,
    widgets::{Block, Gauge as RatatuiGauge, Widget},
};
use tui_pantry::{Ingredient, PropInfo};

use crate::styles::MOCHA;

pub mod ingredient {
    use super::*;

    pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
        vec![
            Box::new(GaugeLow),
            Box::new(GaugeMedium),
            Box::new(GaugeHigh),
            Box::new(GaugeStacked),
        ]
    }
}

const DESCRIPTION: &str = "Horizontal progress bar with label";

const PROPS: &[PropInfo] = &[
    PropInfo {
        name: "ratio",
        ty: "f64",
        description: "Fill amount from 0.0 to 1.0",
    },
    PropInfo {
        name: "label",
        ty: "Span",
        description: "Text centered over the bar",
    },
    PropInfo {
        name: "gauge_style",
        ty: "Style",
        description: "Fill foreground and empty background",
    },
    PropInfo {
        name: "block",
        ty: "Block",
        description: "Surrounding border and title",
    },
];

fn render_gauge(label: &str, ratio: f64, area: Rect, buf: &mut Buffer) {
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

    RatatuiGauge::default()
        .block(
            Block::bordered()
                .title(format!(" {label} "))
                .title_style(Style::default().fg(MOCHA.text))
                .border_style(Style::default().fg(MOCHA.border)),
        )
        .gauge_style(Style::default().fg(color).bg(MOCHA.surface_raised))
        .ratio(ratio)
        .label(format!("{:.0}%", ratio * 100.0))
        .render(col, buf);
}

// ── Low ──

struct GaugeLow;

impl Ingredient for GaugeLow {
    fn group(&self) -> &str {
        "Gauge"
    }
    fn name(&self) -> &str {
        "Low (green)"
    }
    fn source(&self) -> &str {
        "ratatui::widgets::Gauge"
    }
    fn description(&self) -> &str {
        DESCRIPTION
    }
    fn props(&self) -> &[PropInfo] {
        PROPS
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        render_gauge("CPU", 0.34, area, buf);
    }
}

// ── Medium ──

struct GaugeMedium;

impl Ingredient for GaugeMedium {
    fn group(&self) -> &str {
        "Gauge"
    }
    fn name(&self) -> &str {
        "Medium (yellow)"
    }
    fn source(&self) -> &str {
        "ratatui::widgets::Gauge"
    }
    fn description(&self) -> &str {
        DESCRIPTION
    }
    fn props(&self) -> &[PropInfo] {
        PROPS
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        render_gauge("CPU", 0.74, area, buf);
    }
}

// ── High ──

struct GaugeHigh;

impl Ingredient for GaugeHigh {
    fn group(&self) -> &str {
        "Gauge"
    }
    fn name(&self) -> &str {
        "High (red)"
    }
    fn source(&self) -> &str {
        "ratatui::widgets::Gauge"
    }
    fn description(&self) -> &str {
        DESCRIPTION
    }
    fn props(&self) -> &[PropInfo] {
        PROPS
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        render_gauge("CPU", 0.92, area, buf);
    }
}

// ── Stacked ──

struct GaugeStacked;

impl Ingredient for GaugeStacked {
    fn group(&self) -> &str {
        "Gauge"
    }
    fn name(&self) -> &str {
        "Stacked (3-up)"
    }
    fn source(&self) -> &str {
        "ratatui::widgets::Gauge"
    }
    fn description(&self) -> &str {
        DESCRIPTION
    }
    fn props(&self) -> &[PropInfo] {
        PROPS
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let gauges: &[(&str, f64)] = &[("CPU", 0.34), ("Memory", 0.71), ("Disk", 0.88)];

        let constraints: Vec<Constraint> = gauges.iter().map(|_| Constraint::Length(3)).collect();
        let rows = Layout::vertical(constraints).split(area);

        for ((label, ratio), row) in gauges.iter().zip(rows.iter()) {
            let color = MOCHA.ratio_color(*ratio as f32);
            RatatuiGauge::default()
                .block(
                    Block::bordered()
                        .title(format!(" {label} "))
                        .title_style(Style::default().fg(MOCHA.text))
                        .border_style(Style::default().fg(MOCHA.border)),
                )
                .gauge_style(Style::default().fg(color).bg(MOCHA.surface_raised))
                .ratio(*ratio)
                .label(format!("{:.0}%", ratio * 100.0))
                .render(*row, buf);
        }
    }
}
