use tui_pantry::ratatui::buffer::Buffer;
use tui_pantry::ratatui::layout::{Constraint, Layout, Rect};
use tui_pantry::{Ingredient, PropInfo};

pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
    vec![Box::new(DashboardDefault)]
}

// ── Dashboard ──
//
// Full-page composition assembling atomic widgets (Gauge, BarChart,
// Sparkline, Table) into a monitoring dashboard. Demonstrates how
// Views compose Panes and Widgets into a complete layout.

const DASHBOARD_PROPS: &[PropInfo] = &[
    PropInfo { name: "gauge_row", ty: "Pane", description: "Top row — resource gauges (CPU, Memory, Disk)" },
    PropInfo { name: "chart_row", ty: "Pane", description: "Middle row — BarChart and Sparklines side by side" },
    PropInfo { name: "table_row", ty: "Widget", description: "Bottom row — service status table" },
];

struct DashboardDefault;

impl Ingredient for DashboardDefault {
    fn tab(&self) -> &str { "Views" }
    fn group(&self) -> &str { "Dashboard" }
    fn name(&self) -> &str { "Default" }
    fn source(&self) -> &str { "widget_preview::views" }
    fn description(&self) -> &str { "Monitoring dashboard composing gauges, charts, and a service table" }
    fn props(&self) -> &[PropInfo] { DASHBOARD_PROPS }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let [gauge_row, chart_row, table_row] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(10),
            Constraint::Min(8),
        ])
        .areas(area);

        // Top: resource gauges
        crate::panes::render_gauge_strip(
            &[("CPU", 0.58), ("Memory", 0.73), ("Disk", 0.41)],
            gauge_row,
            buf,
        );

        // Middle: charts side by side
        let [chart_col, spark_col] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(chart_row);

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

        // Bottom: service table
        crate::widgets::render_table(&crate::widgets::sample_services(), table_row, buf);
    }
}
