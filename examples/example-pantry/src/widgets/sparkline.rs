use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Style,
    widgets::{Block, Sparkline as RatatuiSparkline, Widget},
};
use tui_pantry::Ingredient;

use crate::styles::{palette::accent, MOCHA};

pub mod ingredient {
    use super::*;

    pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
        vec![Box::new(SparklineDefault), Box::new(SparklineSparse)]
    }
}

fn render_sparkline(
    title: &str,
    data: &[u64],
    color: ratatui::style::Color,
    area: Rect,
    buf: &mut Buffer,
) {
    let [_, row, _] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(5),
        Constraint::Fill(1),
    ])
    .areas(area);

    RatatuiSparkline::default()
        .data(data)
        .style(Style::default().fg(color).bg(MOCHA.surface))
        .block(
            Block::bordered()
                .title(format!(" {title} "))
                .title_style(Style::default().fg(MOCHA.text))
                .border_style(Style::default().fg(MOCHA.border)),
        )
        .render(row, buf);
}

// ── Default ──

struct SparklineDefault;

impl Ingredient for SparklineDefault {
    fn group(&self) -> &str {
        "Sparkline"
    }
    fn name(&self) -> &str {
        "Default"
    }
    fn source(&self) -> &str {
        "ratatui::widgets::Sparkline"
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let data = [
            4, 7, 3, 8, 6, 9, 2, 5, 8, 3, 7, 4, 6, 9, 5, 3, 7, 8, 2, 6, 4, 9, 5, 7, 3, 6, 8, 4, 7,
            5, 3, 9, 6, 4, 8, 7, 2, 5, 9, 3, 7, 4, 6, 8, 5, 3, 7, 9, 2, 6, 4, 8, 5, 7, 3, 6, 9, 4,
            7, 5, 8, 3, 6, 9, 4, 7, 2, 5, 8, 6, 3, 7, 9, 4, 5, 8, 6, 3, 7, 9, 4, 6, 8, 5, 3, 7, 9,
            2, 6, 4, 8, 5, 7, 3, 6, 9, 4, 7, 5, 8,
        ];
        render_sparkline("Throughput", &data, accent::GREEN, area, buf);
    }
}

// ── Sparse ──

struct SparklineSparse;

impl Ingredient for SparklineSparse {
    fn group(&self) -> &str {
        "Sparkline"
    }
    fn name(&self) -> &str {
        "Sparse"
    }
    fn source(&self) -> &str {
        "ratatui::widgets::Sparkline"
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let data = [
            0, 0, 1, 0, 3, 0, 0, 2, 0, 0, 0, 5, 0, 0, 1, 0, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 0, 3,
            0, 0, 0, 1, 0, 0, 0, 0, 2, 0, 0, 0, 0, 4, 0, 0, 1, 0, 0, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0,
            3, 0, 0, 0, 0, 0, 1, 0, 0, 0, 5, 0, 0, 0, 1, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 3, 0, 0, 1,
            0, 0, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0,
        ];
        render_sparkline("Errors", &data, accent::RED, area, buf);
    }
}
