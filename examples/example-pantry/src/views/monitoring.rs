use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Style,
    widgets::{Block, Gauge, Sparkline, Widget},
};
use tui_pantry::Ingredient;

use crate::styles::{palette::accent, MOCHA};

pub mod ingredient {
    use super::*;

    pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
        vec![
            Box::new(MonitoringDefault),
            Box::new(MonitoringAlert),
        ]
    }
}

struct MetricPanel {
    label: &'static str,
    ratio: f64,
    history: &'static [u64],
    spark_color: ratatui::style::Color,
}

fn render_metric(metric: &MetricPanel, area: Rect, buf: &mut Buffer) {
    let [gauge_row, spark_row] =
        Layout::vertical([Constraint::Length(3), Constraint::Min(3)]).areas(area);

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
        .render(gauge_row, buf);

    Sparkline::default()
        .data(metric.history)
        .style(Style::default().fg(metric.spark_color).bg(MOCHA.surface))
        .block(
            Block::bordered()
                .border_style(Style::default().fg(MOCHA.border)),
        )
        .render(spark_row, buf);
}

// ── Default ──

struct MonitoringDefault;

const HEALTHY_METRICS: &[MetricPanel] = &[
    MetricPanel {
        label: "CPU",
        ratio: 0.34,
        history: &[3, 4, 3, 5, 4, 3, 4, 3, 5, 4, 3, 3, 4, 5, 3, 4, 3, 5, 4, 3, 4, 5, 3, 4, 3, 5, 4, 3, 4, 3],
        spark_color: accent::GREEN,
    },
    MetricPanel {
        label: "Memory",
        ratio: 0.52,
        history: &[5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 5, 6, 5, 5, 5, 5, 6, 5, 5, 5, 5, 6, 5, 5, 5, 6, 5, 5, 5, 5],
        spark_color: accent::BLUE,
    },
    MetricPanel {
        label: "Network I/O",
        ratio: 0.28,
        history: &[2, 4, 1, 5, 3, 2, 4, 1, 3, 5, 2, 4, 1, 3, 2, 4, 1, 5, 3, 2, 4, 1, 3, 5, 2, 4, 1, 3, 2, 4],
        spark_color: accent::TEAL,
    },
    MetricPanel {
        label: "Disk",
        ratio: 0.41,
        history: &[4, 4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5],
        spark_color: accent::LAVENDER,
    },
];

impl Ingredient for MonitoringDefault {
    fn tab(&self) -> &str { "Views" }
    fn group(&self) -> &str { "Monitoring" }
    fn name(&self) -> &str { "Healthy" }
    fn source(&self) -> &str { "example_pantry::views::monitoring" }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let cols = Layout::horizontal(vec![Constraint::Ratio(1, 4); 4]).split(area);

        for (metric, col) in HEALTHY_METRICS.iter().zip(cols.iter()) {
            render_metric(metric, *col, buf);
        }
    }
}

// ── Alert state ──

struct MonitoringAlert;

const ALERT_METRICS: &[MetricPanel] = &[
    MetricPanel {
        label: "CPU",
        ratio: 0.92,
        history: &[5, 6, 7, 7, 8, 8, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9],
        spark_color: accent::RED,
    },
    MetricPanel {
        label: "Memory",
        ratio: 0.87,
        history: &[5, 5, 6, 6, 7, 7, 8, 8, 8, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9],
        spark_color: accent::RED,
    },
    MetricPanel {
        label: "Network I/O",
        ratio: 0.76,
        history: &[3, 4, 5, 6, 7, 7, 8, 8, 7, 8, 8, 7, 8, 7, 8, 8, 7, 8, 8, 7, 8, 7, 8, 8, 7, 8, 8, 7, 8, 8],
        spark_color: accent::YELLOW,
    },
    MetricPanel {
        label: "Disk",
        ratio: 0.44,
        history: &[4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 5, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5],
        spark_color: accent::LAVENDER,
    },
];

impl Ingredient for MonitoringAlert {
    fn tab(&self) -> &str { "Views" }
    fn group(&self) -> &str { "Monitoring" }
    fn name(&self) -> &str { "Alert" }
    fn source(&self) -> &str { "example_pantry::views::monitoring" }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let cols = Layout::horizontal(vec![Constraint::Ratio(1, 4); 4]).split(area);

        for (metric, col) in ALERT_METRICS.iter().zip(cols.iter()) {
            render_metric(metric, *col, buf);
        }
    }
}
