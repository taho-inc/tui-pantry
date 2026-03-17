use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Style,
    widgets::{BarChart, Block, Sparkline, Widget},
};
use tui_pantry::{Ingredient, layout::render_centered};

use crate::styles::{palette::accent, MOCHA};

pub mod ingredient {
    use super::*;

    pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
        vec![Box::new(ActivityFeedDefault)]
    }
}

// ── Default ──

struct ActivityFeedDefault;

impl Ingredient for ActivityFeedDefault {
    fn tab(&self) -> &str { "Panes" }
    fn group(&self) -> &str { "Activity Feed" }
    fn name(&self) -> &str { "Default" }
    fn source(&self) -> &str { "example_pantry::panes::activity_feed" }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        render_centered(FeedWidget, Some(Constraint::Max(80)), Some(Constraint::Max(12)), area, buf);
    }
}

struct FeedWidget;

impl Widget for FeedWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [chart_col, spark_col] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .areas(area);

        render_barchart(chart_col, buf);
        render_sparklines(spark_col, buf);
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
        ], accent::GREEN),
        ("Errors", &[
            0, 0, 1, 0, 3, 0, 0, 2, 0, 0, 0, 5, 0, 0, 1, 0, 0, 0, 0, 2,
            0, 0, 0, 1, 0, 0, 0, 0, 3, 0, 0, 0, 1, 0, 0, 0, 0, 2, 0, 0,
        ], accent::RED),
    ];

    let rows = Layout::vertical(vec![Constraint::Ratio(1, series.len() as u32); series.len()])
        .split(area);

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
