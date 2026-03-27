use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::Span,
    widgets::{Axis, Block, Chart as RatatuiChart, Dataset, GraphType, Widget},
};
use tui_pantry::Ingredient;

use crate::styles::{MOCHA, palette::accent};

pub mod ingredient {
    use super::*;

    pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
        vec![Box::new(ChartLine), Box::new(ChartScatter)]
    }
}

// ── Line ──

struct ChartLine;

const CPU_USER: [(f64, f64); 10] = [
    (0.0, 12.0),
    (1.0, 28.0),
    (2.0, 35.0),
    (3.0, 42.0),
    (4.0, 38.0),
    (5.0, 55.0),
    (6.0, 48.0),
    (7.0, 62.0),
    (8.0, 58.0),
    (9.0, 45.0),
];

const CPU_SYSTEM: [(f64, f64); 10] = [
    (0.0, 5.0),
    (1.0, 8.0),
    (2.0, 12.0),
    (3.0, 15.0),
    (4.0, 10.0),
    (5.0, 18.0),
    (6.0, 22.0),
    (7.0, 20.0),
    (8.0, 16.0),
    (9.0, 14.0),
];

impl Ingredient for ChartLine {
    fn section(&self) -> Option<&str> {
        Some("Charts")
    }

    fn group(&self) -> &str {
        "Chart"
    }
    fn name(&self) -> &str {
        "Line"
    }
    fn source(&self) -> &str {
        "ratatui::widgets::Chart"
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let datasets = vec![
            Dataset::default()
                .name("user")
                .graph_type(GraphType::Line)
                .style(Style::default().fg(accent::BLUE))
                .data(&CPU_USER),
            Dataset::default()
                .name("system")
                .graph_type(GraphType::Line)
                .style(Style::default().fg(accent::RED))
                .data(&CPU_SYSTEM),
        ];

        let x_labels =
            ["0s", "5s", "9s"].map(|s| Span::styled(s, Style::default().fg(MOCHA.text_dim)));
        let y_labels =
            ["0%", "50%", "100%"].map(|s| Span::styled(s, Style::default().fg(MOCHA.text_dim)));

        RatatuiChart::new(datasets)
            .block(
                Block::bordered()
                    .title(" CPU Usage ")
                    .title_style(Style::default().fg(MOCHA.text))
                    .border_style(Style::default().fg(MOCHA.border)),
            )
            .style(Style::default().bg(MOCHA.surface))
            .x_axis(
                Axis::default()
                    .title(Span::styled("time", Style::default().fg(MOCHA.text_dim)))
                    .bounds([0.0, 9.0])
                    .labels(x_labels),
            )
            .y_axis(
                Axis::default()
                    .title(Span::styled("%", Style::default().fg(MOCHA.text_dim)))
                    .bounds([0.0, 100.0])
                    .labels(y_labels),
            )
            .render(area, buf);
    }
}

// ── Scatter ──

struct ChartScatter;

const SCATTER_A: [(f64, f64); 12] = [
    (1.0, 22.0),
    (2.5, 45.0),
    (3.0, 38.0),
    (4.2, 60.0),
    (5.5, 52.0),
    (6.0, 70.0),
    (7.3, 48.0),
    (8.0, 65.0),
    (8.8, 75.0),
    (9.5, 55.0),
    (10.0, 80.0),
    (11.0, 42.0),
];

const SCATTER_B: [(f64, f64); 10] = [
    (1.5, 35.0),
    (2.0, 18.0),
    (3.8, 50.0),
    (4.5, 28.0),
    (5.0, 42.0),
    (6.5, 33.0),
    (7.0, 58.0),
    (8.5, 40.0),
    (9.0, 68.0),
    (10.5, 55.0),
];

impl Ingredient for ChartScatter {
    fn section(&self) -> Option<&str> {
        Some("Charts")
    }

    fn group(&self) -> &str {
        "Chart"
    }
    fn name(&self) -> &str {
        "Scatter"
    }
    fn source(&self) -> &str {
        "ratatui::widgets::Chart"
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let datasets = vec![
            Dataset::default()
                .name("series A")
                .graph_type(GraphType::Scatter)
                .style(Style::default().fg(accent::GREEN))
                .data(&SCATTER_A),
            Dataset::default()
                .name("series B")
                .graph_type(GraphType::Scatter)
                .style(
                    Style::default()
                        .fg(accent::MAUVE)
                        .add_modifier(Modifier::BOLD),
                )
                .data(&SCATTER_B),
        ];

        let x_labels =
            ["0", "6", "12"].map(|s| Span::styled(s, Style::default().fg(MOCHA.text_dim)));
        let y_labels =
            ["0", "50", "100"].map(|s| Span::styled(s, Style::default().fg(MOCHA.text_dim)));

        RatatuiChart::new(datasets)
            .block(
                Block::bordered()
                    .title(" Scatter Plot ")
                    .title_style(Style::default().fg(MOCHA.text))
                    .border_style(Style::default().fg(MOCHA.border)),
            )
            .style(Style::default().bg(MOCHA.surface))
            .x_axis(Axis::default().bounds([0.0, 12.0]).labels(x_labels))
            .y_axis(Axis::default().bounds([0.0, 100.0]).labels(y_labels))
            .render(area, buf);
    }
}
