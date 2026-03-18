use tui_pantry::ratatui::buffer::Buffer;
use tui_pantry::ratatui::layout::{Constraint, Layout, Rect};
use tui_pantry::ratatui::style::Style;
use tui_pantry::ratatui::widgets::{Block, Gauge, Widget};
use tui_pantry::{Ingredient, PropInfo};

pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
    vec![
        Box::new(ResourceGaugesHealthy),
        Box::new(ResourceGaugesStressed),
        Box::new(ActivityFeedDefault),
    ]
}

// ── Resource Gauges ──
//
// Horizontal strip of gauges — one per resource. Reuses the threshold
// coloring from widgets::gauge_color but composes multiple gauges into
// a single pane.

const RESOURCE_GAUGE_PROPS: &[PropInfo] = &[
    PropInfo { name: "metrics", ty: "&[Metric]", description: "Slice of label + ratio pairs rendered as adjacent gauges" },
];

pub fn render_gauge_strip(metrics: &[(&str, f64)], area: Rect, buf: &mut Buffer) {
    let cols = Layout::horizontal(
        vec![Constraint::Ratio(1, metrics.len() as u32); metrics.len()],
    )
    .split(area);

    for ((label, ratio), col) in metrics.iter().zip(cols.iter()) {
        let color = crate::widgets::gauge_color(*ratio);
        Gauge::default()
            .block(
                Block::bordered()
                    .title(format!(" {label} "))
                    .title_style(Style::default().fg(crate::TEXT))
                    .border_style(Style::default().fg(crate::BORDER)),
            )
            .gauge_style(Style::default().fg(color))
            .ratio(*ratio)
            .label(format!("{:.0}%", ratio * 100.0))
            .render(*col, buf);
    }
}

struct ResourceGaugesHealthy;

impl Ingredient for ResourceGaugesHealthy {
    fn tab(&self) -> &str { "Panes" }
    fn group(&self) -> &str { "Resource Gauges" }
    fn name(&self) -> &str { "Healthy" }
    fn source(&self) -> &str { "widget_preview::panes" }
    fn description(&self) -> &str { "Three gauges at comfortable utilization levels" }
    fn props(&self) -> &[PropInfo] { RESOURCE_GAUGE_PROPS }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        render_gauge_strip(
            &[("CPU", 0.34), ("Memory", 0.52), ("Disk", 0.41)],
            area,
            buf,
        );
    }
}

struct ResourceGaugesStressed;

impl Ingredient for ResourceGaugesStressed {
    fn tab(&self) -> &str { "Panes" }
    fn group(&self) -> &str { "Resource Gauges" }
    fn name(&self) -> &str { "Stressed" }
    fn source(&self) -> &str { "widget_preview::panes" }
    fn description(&self) -> &str { "Three gauges at high utilization — colors shift to warn/critical" }
    fn props(&self) -> &[PropInfo] { RESOURCE_GAUGE_PROPS }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        render_gauge_strip(
            &[("CPU", 0.92), ("Memory", 0.87), ("Disk", 0.76)],
            area,
            buf,
        );
    }
}

// ── Activity Feed ──
//
// Side-by-side BarChart and Sparklines showing request throughput
// and error rates. Composes the atomic BarChart and Sparkline widgets
// into a single monitoring pane.

const ACTIVITY_FEED_PROPS: &[PropInfo] = &[
    PropInfo { name: "left", ty: "BarChart", description: "Regional request rates" },
    PropInfo { name: "right", ty: "Sparkline[]", description: "Stacked time-series for throughput and errors" },
];

struct ActivityFeedDefault;

impl Ingredient for ActivityFeedDefault {
    fn tab(&self) -> &str { "Panes" }
    fn group(&self) -> &str { "Activity Feed" }
    fn name(&self) -> &str { "Default" }
    fn source(&self) -> &str { "widget_preview::panes" }
    fn description(&self) -> &str { "BarChart + Sparklines side by side for request monitoring" }
    fn props(&self) -> &[PropInfo] { ACTIVITY_FEED_PROPS }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let [chart_col, spark_col] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(area);

        crate::widgets::render_barchart(chart_col, buf);

        let [throughput_row, error_row] =
            Layout::vertical([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
                .areas(spark_col);

        crate::widgets::render_sparkline(
            "Throughput",
            crate::widgets::THROUGHPUT_DATA,
            crate::GREEN,
            throughput_row,
            buf,
        );
        crate::widgets::render_sparkline(
            "Errors",
            crate::widgets::ERROR_DATA,
            crate::RED,
            error_row,
            buf,
        );
    }
}
