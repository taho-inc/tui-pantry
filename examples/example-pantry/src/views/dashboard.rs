use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    widgets::{
        BarChart, Block, Cell, Gauge, Row, Sparkline, StatefulWidget, Table, TableState, Widget,
    },
};
use tui_pantry::Ingredient;

use crate::styles::{palette::accent, MOCHA};

pub mod ingredient {
    use super::*;

    pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
        vec![
            Box::new(DashboardDefault::new()),
            Box::new(DashboardEmpty),
        ]
    }
}

// ── Sample data ──

struct ServiceRow {
    name: &'static str,
    region: &'static str,
    cpu: u8,
    requests: u64,
    status: &'static str,
}

fn sample_services() -> Vec<ServiceRow> {
    vec![
        ServiceRow { name: "api-gateway", region: "us-east", cpu: 34, requests: 1420, status: "Healthy" },
        ServiceRow { name: "auth-svc", region: "us-east", cpu: 71, requests: 890, status: "Healthy" },
        ServiceRow { name: "data-pipeline", region: "eu-west", cpu: 88, requests: 2100, status: "Degraded" },
        ServiceRow { name: "ml-inference", region: "ap-south", cpu: 45, requests: 340, status: "Healthy" },
        ServiceRow { name: "cache-layer", region: "us-west", cpu: 22, requests: 5600, status: "Healthy" },
        ServiceRow { name: "queue-worker", region: "eu-west", cpu: 92, requests: 780, status: "Critical" },
    ]
}

fn status_color(status: &str) -> ratatui::style::Color {
    match status {
        "Healthy" => MOCHA.ok,
        "Degraded" => MOCHA.warn,
        "Critical" => MOCHA.critical,
        _ => MOCHA.text_dim,
    }
}

// ── Shared rendering ──

fn render_gauges(area: Rect, buf: &mut Buffer) {
    let gauges: &[(&str, f64)] = &[("CPU", 0.58), ("Memory", 0.73), ("Disk", 0.41)];
    let cols = Layout::horizontal(vec![Constraint::Ratio(1, 3); 3]).split(area);

    for ((label, ratio), col) in gauges.iter().zip(cols.iter()) {
        let color = MOCHA.ratio_color(*ratio as f32);
        Gauge::default()
            .block(
                Block::bordered()
                    .title(format!(" {label} "))
                    .title_style(Style::default().fg(MOCHA.text))
                    .border_style(Style::default().fg(MOCHA.border)),
            )
            .gauge_style(Style::default().fg(color).bg(MOCHA.surface_raised))
            .ratio(*ratio)
            .label(format!("{:.0}%", ratio * 100.0))
            .render(*col, buf);
    }
}

fn render_barchart(area: Rect, buf: &mut Buffer) {
    let data = [
        ("us-east", 42),
        ("us-west", 38),
        ("eu-west", 55),
        ("ap-south", 29),
    ];

    BarChart::default()
        .data(&data)
        .bar_width(7)
        .bar_gap(2)
        .bar_style(Style::default().fg(accent::BLUE))
        .value_style(Style::default().fg(MOCHA.text))
        .label_style(Style::default().fg(MOCHA.text_dim))
        .block(
            Block::bordered()
                .title(" Requests/s ")
                .title_style(Style::default().fg(MOCHA.text))
                .border_style(Style::default().fg(MOCHA.border)),
        )
        .style(Style::default().bg(MOCHA.surface))
        .render(area, buf);
}

fn render_sparklines(area: Rect, buf: &mut Buffer) {
    let series: &[(&str, &[u64], ratatui::style::Color)] = &[
        ("Throughput", &[
            4, 7, 3, 8, 6, 9, 2, 5, 8, 3, 7, 4, 6, 9, 5, 3, 7, 8, 2, 6,
            4, 9, 5, 7, 3, 6, 8, 4, 7, 5, 3, 9, 6, 4, 8, 7, 2, 5, 9, 3,
            7, 4, 6, 8, 5, 3, 7, 9, 2, 6, 4, 8, 5, 7, 3, 6, 9, 4, 7, 5,
        ], accent::GREEN),
        ("Errors", &[
            0, 0, 1, 0, 3, 0, 0, 2, 0, 0, 0, 5, 0, 0, 1, 0, 0, 0, 0, 2,
            0, 0, 0, 1, 0, 0, 0, 0, 3, 0, 0, 0, 1, 0, 0, 0, 0, 2, 0, 0,
            0, 0, 4, 0, 0, 1, 0, 0, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 3, 0,
        ], accent::RED),
    ];

    let rows = Layout::vertical(vec![Constraint::Length(4); series.len()]).split(area);

    for ((title, data, color), row) in series.iter().zip(rows.iter()) {
        Sparkline::default()
            .data(*data)
            .style(Style::default().fg(*color).bg(MOCHA.surface))
            .block(
                Block::bordered()
                    .title(format!(" {title} "))
                    .title_style(Style::default().fg(MOCHA.text))
                    .border_style(Style::default().fg(MOCHA.border)),
            )
            .render(*row, buf);
    }
}

fn render_table(services: &[ServiceRow], state: &mut TableState, area: Rect, buf: &mut Buffer) {
    let header = Row::new(["Service", "Region", "CPU", "Req/s", "Status"])
        .style(Style::default().fg(MOCHA.text).add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = services
        .iter()
        .map(|s| {
            let cpu_color = MOCHA.percent_color(s.cpu);
            Row::new([
                Cell::from(s.name).style(Style::default().fg(MOCHA.text)),
                Cell::from(s.region).style(Style::default().fg(MOCHA.text_dim)),
                Cell::from(format!("{}%", s.cpu)).style(Style::default().fg(cpu_color)),
                Cell::from(s.requests.to_string()).style(Style::default().fg(MOCHA.text_dim)),
                Cell::from(s.status).style(Style::default().fg(status_color(s.status))),
            ])
        })
        .collect();

    let table = Table::new(
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
            .title_style(Style::default().fg(MOCHA.text))
            .border_style(Style::default().fg(MOCHA.border)),
    )
    .style(Style::default().bg(MOCHA.surface))
    .row_highlight_style(
        Style::default()
            .bg(MOCHA.surface_raised)
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("▸ ");

    StatefulWidget::render(table, area, buf, state);
}

// ── Dashboard Default (interactive) ──

struct DashboardDefault {
    services: Vec<ServiceRow>,
    state: TableState,
}

impl DashboardDefault {
    fn new() -> Self {
        let services = sample_services();
        let mut state = TableState::default();
        state.select(Some(0));
        Self { services, state }
    }
}

impl Ingredient for DashboardDefault {
    fn tab(&self) -> &str { "Views" }
    fn group(&self) -> &str { "Dashboard" }
    fn name(&self) -> &str { "Default" }
    fn source(&self) -> &str { "example_pantry::views::dashboard" }
    fn interactive(&self) -> bool { true }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        // Top row: gauges
        // Middle row: barchart | sparklines
        // Bottom: services table
        let [gauge_row, chart_row, table_row] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(10),
            Constraint::Min(8),
        ])
        .areas(area);

        render_gauges(gauge_row, buf);

        let [chart_col, spark_col] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(chart_row);

        render_barchart(chart_col, buf);
        render_sparklines(spark_col, buf);

        let mut state = self.state;
        render_table(&self.services, &mut state, table_row, buf);
    }

    fn handle_key(&mut self, code: KeyCode) -> bool {
        let len = self.services.len();
        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                let i = self.state.selected().unwrap_or(0);
                self.state.select(Some(if i == 0 { len.saturating_sub(1) } else { i - 1 }));
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let i = self.state.selected().unwrap_or(0);
                self.state.select(Some(if i >= len - 1 { 0 } else { i + 1 }));
                true
            }
            _ => false,
        }
    }
}

// ── Dashboard Empty ──

struct DashboardEmpty;

impl Ingredient for DashboardEmpty {
    fn tab(&self) -> &str { "Views" }
    fn group(&self) -> &str { "Dashboard" }
    fn name(&self) -> &str { "Empty" }
    fn source(&self) -> &str { "example_pantry::views::dashboard" }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let [gauge_row, chart_row, table_row] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(10),
            Constraint::Min(8),
        ])
        .areas(area);

        let cols = Layout::horizontal(vec![Constraint::Ratio(1, 3); 3]).split(gauge_row);
        for (label, col) in ["CPU", "Memory", "Disk"].iter().zip(cols.iter()) {
            Gauge::default()
                .block(
                    Block::bordered()
                        .title(format!(" {label} "))
                        .title_style(Style::default().fg(MOCHA.text))
                        .border_style(Style::default().fg(MOCHA.border)),
                )
                .gauge_style(Style::default().fg(MOCHA.text_disabled).bg(MOCHA.surface_raised))
                .ratio(0.0)
                .label("—")
                .render(*col, buf);
        }

        let [chart_col, spark_col] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(chart_row);

        BarChart::default()
            .data(&[])
            .block(
                Block::bordered()
                    .title(" Requests/s ")
                    .title_style(Style::default().fg(MOCHA.text))
                    .border_style(Style::default().fg(MOCHA.border)),
            )
            .style(Style::default().bg(MOCHA.surface))
            .render(chart_col, buf);

        Sparkline::default()
            .data(&[] as &[u64])
            .style(Style::default().bg(MOCHA.surface))
            .block(
                Block::bordered()
                    .title(" Throughput ")
                    .title_style(Style::default().fg(MOCHA.text))
                    .border_style(Style::default().fg(MOCHA.border)),
            )
            .render(spark_col, buf);

        let header = Row::new(["Service", "Region", "CPU", "Req/s", "Status"])
            .style(Style::default().fg(MOCHA.text).add_modifier(Modifier::BOLD));

        let table = Table::new(
            Vec::<Row>::new(),
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
                .title_style(Style::default().fg(MOCHA.text))
                .border_style(Style::default().fg(MOCHA.border)),
        )
        .style(Style::default().bg(MOCHA.surface));

        Widget::render(table, table_row, buf);
    }
}
