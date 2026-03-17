use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Style,
    widgets::{Block, Gauge, Sparkline, Widget},
};
use tui_pantry::{Ingredient, layout::render_centered};

use crate::styles::{palette::accent, MOCHA};

pub mod ingredient {
    use super::*;

    pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
        vec![
            Box::new(MetricPanelSteady),
            Box::new(MetricPanelSpike),
        ]
    }
}

struct MetricData {
    label: &'static str,
    ratio: f64,
    history: &'static [u64],
    spark_color: ratatui::style::Color,
}

fn render_metric(data: &MetricData, area: Rect, buf: &mut Buffer) {
    let [gauge_row, spark_row] =
        Layout::vertical([Constraint::Length(3), Constraint::Min(3)]).areas(area);

    let color = MOCHA.ratio_color(data.ratio as f32);

    Gauge::default()
        .block(
            Block::bordered()
                .title(format!(" {} ", data.label))
                .title_style(Style::default().fg(MOCHA.text))
                .border_style(Style::default().fg(MOCHA.border)),
        )
        .gauge_style(Style::default().fg(color).bg(MOCHA.surface_raised))
        .ratio(data.ratio)
        .label(format!("{:.0}%", data.ratio * 100.0))
        .render(gauge_row, buf);

    Sparkline::default()
        .data(data.history)
        .style(Style::default().fg(data.spark_color).bg(MOCHA.surface))
        .block(
            Block::bordered()
                .border_style(Style::default().fg(MOCHA.border)),
        )
        .render(spark_row, buf);
}

// ── Steady ──

struct MetricPanelSteady;

const STEADY: MetricData = MetricData {
    label: "CPU",
    ratio: 0.34,
    history: &[3, 4, 3, 5, 4, 3, 4, 3, 5, 4, 3, 3, 4, 5, 3, 4, 3, 5, 4, 3, 4, 5, 3, 4, 3, 5, 4, 3, 4, 3],
    spark_color: accent::GREEN,
};

impl Ingredient for MetricPanelSteady {
    fn tab(&self) -> &str { "Panes" }
    fn group(&self) -> &str { "Metric Panel" }
    fn name(&self) -> &str { "Steady" }
    fn source(&self) -> &str { "example_pantry::panes::metric_panel" }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        render_centered(MetricWidget(&STEADY), Some(Constraint::Max(40)), Some(Constraint::Max(12)), area, buf);
    }
}

// ── Spike ──

struct MetricPanelSpike;

const SPIKE: MetricData = MetricData {
    label: "CPU",
    ratio: 0.92,
    history: &[5, 6, 7, 7, 8, 8, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9],
    spark_color: accent::RED,
};

impl Ingredient for MetricPanelSpike {
    fn tab(&self) -> &str { "Panes" }
    fn group(&self) -> &str { "Metric Panel" }
    fn name(&self) -> &str { "Spike" }
    fn source(&self) -> &str { "example_pantry::panes::metric_panel" }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        render_centered(MetricWidget(&SPIKE), Some(Constraint::Max(40)), Some(Constraint::Max(12)), area, buf);
    }
}

struct MetricWidget<'a>(&'a MetricData);

impl Widget for MetricWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        render_metric(self.0, area, buf);
    }
}
