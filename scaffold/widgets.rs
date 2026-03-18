use tui_pantry::ratatui::buffer::Buffer;
use tui_pantry::ratatui::layout::{Constraint, Rect};
use tui_pantry::ratatui::style::{Modifier, Style};
use tui_pantry::ratatui::widgets::{
    BarChart, Block, Cell, Gauge, Row, Sparkline, Table, Widget,
};
use tui_pantry::{Ingredient, PropInfo};

pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
    vec![
        Box::new(GaugeDefault),
        Box::new(GaugeHigh),
        Box::new(SparklineDefault),
        Box::new(BarChartDefault),
        Box::new(ServiceTableDefault),
    ]
}

// ── Gauge ──

const GAUGE_PROPS: &[PropInfo] = &[
    PropInfo { name: "label", ty: "&str", description: "Resource name displayed in the border title" },
    PropInfo { name: "ratio", ty: "f64", description: "Fill from 0.0 to 1.0" },
    PropInfo { name: "color", ty: "Color", description: "Bar color — typically derived from ratio thresholds" },
];

pub fn gauge_color(ratio: f64) -> tui_pantry::ratatui::style::Color {
    if ratio < 0.6 { crate::GREEN } else if ratio < 0.8 { crate::YELLOW } else { crate::RED }
}

fn render_gauge(label: &str, ratio: f64, area: Rect, buf: &mut Buffer) {
    let color = gauge_color(ratio);
    Gauge::default()
        .block(
            Block::bordered()
                .title(format!(" {label} "))
                .title_style(Style::default().fg(crate::TEXT))
                .border_style(Style::default().fg(crate::BORDER)),
        )
        .gauge_style(Style::default().fg(color))
        .ratio(ratio)
        .label(format!("{:.0}%", ratio * 100.0))
        .render(area, buf);
}

struct GaugeDefault;

impl Ingredient for GaugeDefault {
    fn group(&self) -> &str { "Gauge" }
    fn name(&self) -> &str { "Default (34%)" }
    fn source(&self) -> &str { "widget_preview::widgets" }
    fn description(&self) -> &str { "Resource utilization bar with threshold-based coloring" }
    fn props(&self) -> &[PropInfo] { GAUGE_PROPS }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        render_gauge("CPU", 0.34, area, buf);
    }
}

struct GaugeHigh;

impl Ingredient for GaugeHigh {
    fn group(&self) -> &str { "Gauge" }
    fn name(&self) -> &str { "High (88%)" }
    fn source(&self) -> &str { "widget_preview::widgets" }
    fn description(&self) -> &str { "Gauge at high utilization — color shifts to red above 80%" }
    fn props(&self) -> &[PropInfo] { GAUGE_PROPS }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        render_gauge("CPU", 0.88, area, buf);
    }
}

// ── Sparkline ──

const SPARKLINE_PROPS: &[PropInfo] = &[
    PropInfo { name: "data", ty: "&[u64]", description: "Time-series values rendered as vertical bars" },
    PropInfo { name: "color", ty: "Color", description: "Bar color" },
    PropInfo { name: "title", ty: "&str", description: "Border title label" },
];

pub const THROUGHPUT_DATA: &[u64] = &[
    4, 7, 3, 8, 6, 9, 2, 5, 8, 3, 7, 4, 6, 9, 5, 3, 7, 8, 2, 6,
    4, 9, 5, 7, 3, 6, 8, 4, 7, 5, 3, 9, 6, 4, 8, 7, 2, 5, 9, 3,
];

pub const ERROR_DATA: &[u64] = &[
    0, 0, 1, 0, 3, 0, 0, 2, 0, 0, 0, 5, 0, 0, 1, 0, 0, 0, 0, 2,
    0, 0, 0, 1, 0, 0, 0, 0, 3, 0, 0, 0, 1, 0, 0, 0, 0, 2, 0, 0,
];

pub fn render_sparkline(
    title: &str,
    data: &[u64],
    color: tui_pantry::ratatui::style::Color,
    area: Rect,
    buf: &mut Buffer,
) {
    Sparkline::default()
        .data(data)
        .style(Style::default().fg(color))
        .block(
            Block::bordered()
                .title(format!(" {title} "))
                .title_style(Style::default().fg(crate::TEXT))
                .border_style(Style::default().fg(crate::BORDER)),
        )
        .render(area, buf);
}

struct SparklineDefault;

impl Ingredient for SparklineDefault {
    fn group(&self) -> &str { "Sparkline" }
    fn name(&self) -> &str { "Throughput" }
    fn source(&self) -> &str { "widget_preview::widgets" }
    fn description(&self) -> &str { "Inline time-series chart for trending data" }
    fn props(&self) -> &[PropInfo] { SPARKLINE_PROPS }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        render_sparkline("Throughput", THROUGHPUT_DATA, crate::GREEN, area, buf);
    }
}

// ── BarChart ──

const BARCHART_PROPS: &[PropInfo] = &[
    PropInfo { name: "data", ty: "&[(&str, u64)]", description: "Label-value pairs for each bar" },
    PropInfo { name: "bar_width", ty: "u16", description: "Character width of each bar" },
    PropInfo { name: "bar_gap", ty: "u16", description: "Gap between bars" },
];

pub const REGION_DATA: &[(&str, u64)] = &[
    ("us-east", 42),
    ("us-west", 38),
    ("eu-west", 55),
    ("ap-south", 29),
];

pub fn render_barchart(area: Rect, buf: &mut Buffer) {
    BarChart::default()
        .data(REGION_DATA)
        .bar_width(7)
        .bar_gap(2)
        .bar_style(Style::default().fg(crate::BLUE))
        .value_style(Style::default().fg(crate::TEXT))
        .label_style(Style::default().fg(crate::TEXT_DIM))
        .block(
            Block::bordered()
                .title(" Requests/s ")
                .title_style(Style::default().fg(crate::TEXT))
                .border_style(Style::default().fg(crate::BORDER)),
        )
        .render(area, buf);
}

struct BarChartDefault;

impl Ingredient for BarChartDefault {
    fn group(&self) -> &str { "BarChart" }
    fn name(&self) -> &str { "Regional" }
    fn source(&self) -> &str { "widget_preview::widgets" }
    fn description(&self) -> &str { "Vertical bar chart comparing values across categories" }
    fn props(&self) -> &[PropInfo] { BARCHART_PROPS }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        render_barchart(area, buf);
    }
}

// ── Table ──

const TABLE_PROPS: &[PropInfo] = &[
    PropInfo { name: "rows", ty: "Vec<ServiceRow>", description: "Data rows with name, region, cpu, requests, and status" },
    PropInfo { name: "header", ty: "Row", description: "Column header labels" },
    PropInfo { name: "widths", ty: "&[Constraint]", description: "Column width constraints" },
];

pub struct ServiceRow {
    pub name: &'static str,
    pub region: &'static str,
    pub cpu: u8,
    pub requests: u64,
    pub status: &'static str,
}

pub fn sample_services() -> Vec<ServiceRow> {
    vec![
        ServiceRow { name: "api-gateway",    region: "us-east", cpu: 34, requests: 1420, status: "Healthy" },
        ServiceRow { name: "auth-svc",       region: "us-east", cpu: 71, requests: 890,  status: "Healthy" },
        ServiceRow { name: "data-pipeline",  region: "eu-west", cpu: 88, requests: 2100, status: "Degraded" },
        ServiceRow { name: "ml-inference",   region: "ap-south", cpu: 45, requests: 340,  status: "Healthy" },
        ServiceRow { name: "cache-layer",    region: "us-west", cpu: 22, requests: 5600, status: "Healthy" },
        ServiceRow { name: "queue-worker",   region: "eu-west", cpu: 92, requests: 780,  status: "Critical" },
    ]
}

fn status_color(status: &str) -> tui_pantry::ratatui::style::Color {
    match status {
        "Healthy" => crate::GREEN,
        "Degraded" => crate::YELLOW,
        "Critical" => crate::RED,
        _ => crate::TEXT_DIM,
    }
}

pub fn render_table(services: &[ServiceRow], area: Rect, buf: &mut Buffer) {
    let header = Row::new(["Service", "Region", "CPU", "Req/s", "Status"])
        .style(Style::default().fg(crate::TEXT).add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = services
        .iter()
        .map(|s| {
            Row::new([
                Cell::from(s.name).style(Style::default().fg(crate::TEXT)),
                Cell::from(s.region).style(Style::default().fg(crate::TEXT_DIM)),
                Cell::from(format!("{}%", s.cpu)).style(Style::default().fg(gauge_color(s.cpu as f64 / 100.0))),
                Cell::from(s.requests.to_string()).style(Style::default().fg(crate::TEXT_DIM)),
                Cell::from(s.status).style(Style::default().fg(status_color(s.status))),
            ])
        })
        .collect();

    Table::new(
        rows,
        [
            Constraint::Min(14),
            Constraint::Min(10),
            Constraint::Min(6),
            Constraint::Min(8),
            Constraint::Min(10),
        ],
    )
    .header(header)
    .block(
        Block::bordered()
            .title(" Services ")
            .title_style(Style::default().fg(crate::TEXT))
            .border_style(Style::default().fg(crate::BORDER)),
    )
    .render(area, buf);
}

struct ServiceTableDefault;

impl Ingredient for ServiceTableDefault {
    fn group(&self) -> &str { "Table" }
    fn name(&self) -> &str { "Service List" }
    fn source(&self) -> &str { "widget_preview::widgets" }
    fn description(&self) -> &str { "Multi-column table with status-colored cells" }
    fn props(&self) -> &[PropInfo] { TABLE_PROPS }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        render_table(&sample_services(), area, buf);
    }
}
