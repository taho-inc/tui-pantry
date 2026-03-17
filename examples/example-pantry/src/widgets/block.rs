use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block as RatatuiBlock, Borders, Padding, Widget},
};
use tui_pantry::{Ingredient, PropInfo};

use crate::styles::MOCHA;

pub mod ingredient {
    use super::*;

    pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
        vec![
            Box::new(BlockPlain),
            Box::new(BlockRounded),
            Box::new(BlockDouble),
            Box::new(BlockTitled),
        ]
    }
}

const DESCRIPTION: &str = "Container with border and optional title";

const PROPS: &[PropInfo] = &[
    PropInfo { name: "borders", ty: "Borders", description: "Which edges to draw" },
    PropInfo { name: "border_type", ty: "BorderType", description: "Line style: Plain, Rounded, Double, Thick" },
    PropInfo { name: "border_style", ty: "Style", description: "Border line color and modifiers" },
    PropInfo { name: "title", ty: "Line", description: "Text rendered on the top border" },
    PropInfo { name: "title_style", ty: "Style", description: "Title text color and modifiers" },
    PropInfo { name: "padding", ty: "Padding", description: "Inner spacing between border and content" },
    PropInfo { name: "style", ty: "Style", description: "Background and default foreground" },
];

fn base_style() -> Style {
    Style::default().bg(MOCHA.surface)
}

fn border_style() -> Style {
    Style::default().fg(MOCHA.border)
}

// ── Plain ──

struct BlockPlain;

impl Ingredient for BlockPlain {
    fn group(&self) -> &str { "Block" }
    fn name(&self) -> &str { "Plain" }
    fn source(&self) -> &str { "ratatui::widgets::Block" }
    fn description(&self) -> &str { DESCRIPTION }
    fn props(&self) -> &[PropInfo] { PROPS }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        RatatuiBlock::default()
            .borders(Borders::ALL)
            .border_style(border_style())
            .style(base_style())
            .render(area, buf);
    }
}

// ── Rounded ──

struct BlockRounded;

impl Ingredient for BlockRounded {
    fn group(&self) -> &str { "Block" }
    fn name(&self) -> &str { "Rounded" }
    fn source(&self) -> &str { "ratatui::widgets::Block" }
    fn description(&self) -> &str { DESCRIPTION }
    fn props(&self) -> &[PropInfo] { PROPS }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        RatatuiBlock::bordered()
            .border_style(border_style())
            .style(base_style())
            .render(area, buf);
    }
}

// ── Double ──

struct BlockDouble;

impl Ingredient for BlockDouble {
    fn group(&self) -> &str { "Block" }
    fn name(&self) -> &str { "Double" }
    fn source(&self) -> &str { "ratatui::widgets::Block" }
    fn description(&self) -> &str { DESCRIPTION }
    fn props(&self) -> &[PropInfo] { PROPS }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        RatatuiBlock::default()
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Double)
            .border_style(border_style())
            .style(base_style())
            .render(area, buf);
    }
}

// ── Titled ──

struct BlockTitled;

impl Ingredient for BlockTitled {
    fn group(&self) -> &str { "Block" }
    fn name(&self) -> &str { "Titled + Padding" }
    fn source(&self) -> &str { "ratatui::widgets::Block" }
    fn description(&self) -> &str { DESCRIPTION }
    fn props(&self) -> &[PropInfo] { PROPS }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        RatatuiBlock::bordered()
            .title(" Panel Title ")
            .title_style(Style::default().fg(MOCHA.text))
            .border_style(Style::default().fg(MOCHA.accent))
            .padding(Padding::uniform(1))
            .style(base_style())
            .render(area, buf);
    }
}
