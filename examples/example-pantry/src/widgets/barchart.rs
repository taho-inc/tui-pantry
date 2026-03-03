use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{BarChart as RatatuiBarChart, Block, Widget},
};
use tui_pantry::Ingredient;

use crate::styles::{palette::accent, MOCHA};

pub mod ingredient {
    use super::*;

    pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
        vec![
            Box::new(BarChartDefault),
            Box::new(BarChartHighLoad),
        ]
    }
}

// ── Default ──

struct BarChartDefault;

impl Ingredient for BarChartDefault {
    fn group(&self) -> &str { "Bar Chart" }
    fn name(&self) -> &str { "Default" }
    fn source(&self) -> &str { "ratatui::widgets::BarChart" }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let data = [
            ("us-east", 42),
            ("us-west", 38),
            ("eu-west", 55),
            ("ap-south", 29),
            ("ap-east", 47),
        ];

        RatatuiBarChart::default()
            .data(&data)
            .bar_width(7)
            .bar_gap(2)
            .bar_style(Style::default().fg(accent::BLUE))
            .value_style(Style::default().fg(MOCHA.text))
            .label_style(Style::default().fg(MOCHA.text_dim))
            .block(
                Block::bordered()
                    .title(" Jobs per Region ")
                    .title_style(Style::default().fg(MOCHA.text))
                    .border_style(Style::default().fg(MOCHA.border)),
            )
            .style(Style::default().bg(MOCHA.surface))
            .render(area, buf);
    }
}

// ── High Load ──

struct BarChartHighLoad;

impl Ingredient for BarChartHighLoad {
    fn group(&self) -> &str { "Bar Chart" }
    fn name(&self) -> &str { "High Load" }
    fn source(&self) -> &str { "ratatui::widgets::BarChart" }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let data = [
            ("us-east", 89),
            ("us-west", 92),
            ("eu-west", 95),
            ("ap-south", 78),
            ("ap-east", 88),
        ];

        RatatuiBarChart::default()
            .data(&data)
            .bar_width(7)
            .bar_gap(2)
            .bar_style(Style::default().fg(accent::RED))
            .value_style(Style::default().fg(MOCHA.text))
            .label_style(Style::default().fg(MOCHA.text_dim))
            .block(
                Block::bordered()
                    .title(" Jobs per Region (overloaded) ")
                    .title_style(Style::default().fg(MOCHA.text))
                    .border_style(Style::default().fg(MOCHA.border)),
            )
            .style(Style::default().bg(MOCHA.surface))
            .render(area, buf);
    }
}
