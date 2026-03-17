use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Tabs as RatatuiTabs, Widget},
};
use tui_pantry::Ingredient;

use crate::styles::MOCHA;

pub mod ingredient {
    use super::*;

    pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
        vec![
            Box::new(TabsDefault),
            Box::new(TabsMany),
            Box::new(TabsSingle),
        ]
    }
}

fn render_tabs(titles: &[&str], selected: usize, area: Rect, buf: &mut Buffer) {
    let [row] = Layout::vertical([Constraint::Length(3)]).areas(area);

    RatatuiTabs::new(titles.iter().copied())
        .block(Block::bordered().border_style(Style::default().fg(MOCHA.border)))
        .style(Style::default().fg(MOCHA.text_dim).bg(MOCHA.surface))
        .highlight_style(
            Style::default()
                .fg(MOCHA.accent)
                .add_modifier(Modifier::BOLD),
        )
        .select(selected)
        .render(row, buf);
}

// ── Default ──

struct TabsDefault;

impl Ingredient for TabsDefault {
    fn group(&self) -> &str {
        "Tabs"
    }
    fn name(&self) -> &str {
        "Default"
    }
    fn source(&self) -> &str {
        "ratatui::widgets::Tabs"
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        render_tabs(&["Overview", "Details", "Settings"], 0, area, buf);
    }
}

// ── Many ──

struct TabsMany;

impl Ingredient for TabsMany {
    fn group(&self) -> &str {
        "Tabs"
    }
    fn name(&self) -> &str {
        "Many Tabs"
    }
    fn source(&self) -> &str {
        "ratatui::widgets::Tabs"
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        render_tabs(
            &[
                "Home",
                "Nodes",
                "Network",
                "Content",
                "Inference",
                "Events",
                "Config",
            ],
            2,
            area,
            buf,
        );
    }
}

// ── Single ──

struct TabsSingle;

impl Ingredient for TabsSingle {
    fn group(&self) -> &str {
        "Tabs"
    }
    fn name(&self) -> &str {
        "Single Tab"
    }
    fn source(&self) -> &str {
        "ratatui::widgets::Tabs"
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        render_tabs(&["Dashboard"], 0, area, buf);
    }
}
