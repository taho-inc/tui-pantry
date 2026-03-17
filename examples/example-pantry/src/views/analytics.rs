use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::Span,
    widgets::{Axis, Block, Chart, Dataset, GraphType, LineGauge, Widget},
};
use tui_pantry::Ingredient;

use crate::styles::{palette::accent, MOCHA};

pub mod ingredient {
    use super::*;

    pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
        vec![Box::new(AnalyticsDefault)]
    }
}

// ── Sample data ──

const LATENCY_P50: [(f64, f64); 12] = [
    (0.0, 12.0), (1.0, 14.0), (2.0, 11.0), (3.0, 15.0), (4.0, 13.0), (5.0, 16.0),
    (6.0, 14.0), (7.0, 18.0), (8.0, 15.0), (9.0, 13.0), (10.0, 17.0), (11.0, 14.0),
];

const LATENCY_P99: [(f64, f64); 12] = [
    (0.0, 45.0), (1.0, 52.0), (2.0, 38.0), (3.0, 61.0), (4.0, 48.0), (5.0, 55.0),
    (6.0, 42.0), (7.0, 70.0), (8.0, 58.0), (9.0, 44.0), (10.0, 65.0), (11.0, 50.0),
];

const ERROR_SCATTER: [(f64, f64); 8] = [
    (1.0, 2.0), (3.0, 5.0), (4.5, 1.0), (6.0, 8.0),
    (7.0, 3.0), (8.5, 12.0), (9.0, 4.0), (11.0, 6.0),
];

fn render_bandwidth_gauges(area: Rect, buf: &mut Buffer) {
    let metrics: &[(&str, f64)] = &[
        ("Inbound", 0.42),
        ("Outbound", 0.67),
        ("Cache Hit", 0.91),
    ];

    let cols = Layout::horizontal(vec![Constraint::Ratio(1, 3); 3]).split(area);

    for ((label, ratio), col) in metrics.iter().zip(cols.iter()) {
        let color = MOCHA.ratio_color(*ratio as f32);
        LineGauge::default()
            .block(
                Block::bordered()
                    .title(format!(" {label} "))
                    .title_style(Style::default().fg(MOCHA.text))
                    .border_style(Style::default().fg(MOCHA.border)),
            )
            .filled_style(Style::default().fg(color).bg(MOCHA.surface_raised))
            .unfilled_style(Style::default().fg(MOCHA.border).bg(MOCHA.surface))
            .ratio(*ratio)
            .label(format!("{:.0}%", ratio * 100.0))
            .render(*col, buf);
    }
}

fn render_latency_chart(area: Rect, buf: &mut Buffer) {
    let datasets = vec![
        Dataset::default()
            .name("p50")
            .graph_type(GraphType::Line)
            .style(Style::default().fg(accent::GREEN))
            .data(&LATENCY_P50),
        Dataset::default()
            .name("p99")
            .graph_type(GraphType::Line)
            .style(Style::default().fg(accent::PEACH))
            .data(&LATENCY_P99),
    ];

    let x_labels = ["0m", "6m", "12m"]
        .map(|s| Span::styled(s, Style::default().fg(MOCHA.text_dim)));
    let y_labels = ["0ms", "50ms", "100ms"]
        .map(|s| Span::styled(s, Style::default().fg(MOCHA.text_dim)));

    Chart::new(datasets)
        .block(
            Block::bordered()
                .title(" Request Latency ")
                .title_style(Style::default().fg(MOCHA.text))
                .border_style(Style::default().fg(MOCHA.border)),
        )
        .style(Style::default().bg(MOCHA.surface))
        .x_axis(
            Axis::default()
                .title(Span::styled("time", Style::default().fg(MOCHA.text_dim)))
                .bounds([0.0, 11.0])
                .labels(x_labels),
        )
        .y_axis(
            Axis::default()
                .title(Span::styled("ms", Style::default().fg(MOCHA.text_dim)))
                .bounds([0.0, 100.0])
                .labels(y_labels),
        )
        .render(area, buf);
}

fn render_error_scatter(area: Rect, buf: &mut Buffer) {
    let datasets = vec![
        Dataset::default()
            .name("errors")
            .graph_type(GraphType::Scatter)
            .style(Style::default().fg(accent::RED))
            .data(&ERROR_SCATTER),
    ];

    let x_labels = ["0m", "6m", "12m"]
        .map(|s| Span::styled(s, Style::default().fg(MOCHA.text_dim)));
    let y_labels = ["0", "8", "16"]
        .map(|s| Span::styled(s, Style::default().fg(MOCHA.text_dim)));

    Chart::new(datasets)
        .block(
            Block::bordered()
                .title(" Error Events ")
                .title_style(Style::default().fg(MOCHA.text))
                .border_style(Style::default().fg(MOCHA.border)),
        )
        .style(Style::default().bg(MOCHA.surface))
        .x_axis(
            Axis::default()
                .bounds([0.0, 12.0])
                .labels(x_labels),
        )
        .y_axis(
            Axis::default()
                .bounds([0.0, 16.0])
                .labels(y_labels),
        )
        .render(area, buf);
}

// ── Analytics Default ──

struct AnalyticsDefault;

impl Ingredient for AnalyticsDefault {
    fn tab(&self) -> &str { "Views" }
    fn group(&self) -> &str { "Analytics" }
    fn name(&self) -> &str { "Default" }
    fn source(&self) -> &str { "example_pantry::views::analytics" }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let [gauge_row, chart_row] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(10),
        ])
        .areas(area);

        render_bandwidth_gauges(gauge_row, buf);

        let [latency_col, error_col] =
            Layout::horizontal([Constraint::Percentage(60), Constraint::Percentage(40)])
                .areas(chart_row);

        render_latency_chart(latency_col, buf);
        render_error_scatter(error_col, buf);
    }
}
