use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    layout::Rect,
    style::{Modifier, Style},
    widgets::{
        Block, HighlightSpacing, List as RatatuiList, ListItem, ListState, StatefulWidget, Widget,
    },
};
use tui_pantry::Ingredient;

use crate::styles::MOCHA;

pub mod ingredient {
    use super::*;

    pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
        vec![
            Box::new(ListDefault::new()),
            Box::new(ListEmpty),
        ]
    }
}

fn sample_items() -> Vec<&'static str> {
    vec![
        "Paragraph",
        "List",
        "Table",
        "Tabs",
        "Gauge",
        "BarChart",
        "Sparkline",
        "Block",
        "Canvas",
        "Chart",
    ]
}

// ── Default (interactive) ──

struct ListDefault {
    items: Vec<&'static str>,
    state: ListState,
}

impl ListDefault {
    fn new() -> Self {
        let items = sample_items();
        let mut state = ListState::default();
        state.select(Some(0));
        Self { items, state }
    }
}

impl Ingredient for ListDefault {
    fn group(&self) -> &str { "List" }
    fn name(&self) -> &str { "Default" }
    fn source(&self) -> &str { "ratatui::widgets::List" }
    fn interactive(&self) -> bool { true }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let items: Vec<ListItem> = self
            .items
            .iter()
            .map(|s| ListItem::new(*s).style(Style::default().fg(MOCHA.text)))
            .collect();

        let list = RatatuiList::new(items)
            .block(
                Block::bordered()
                    .title(" Widgets ")
                    .title_style(Style::default().fg(MOCHA.text))
                    .border_style(Style::default().fg(MOCHA.border)),
            )
            .style(Style::default().bg(MOCHA.surface))
            .highlight_style(
                Style::default()
                    .fg(MOCHA.text)
                    .bg(MOCHA.surface_raised)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▸ ")
            .highlight_spacing(HighlightSpacing::Always);

        // Clone state for rendering (StatefulWidget needs &mut)
        let mut state = self.state;
        StatefulWidget::render(list, area, buf, &mut state);
    }

    fn handle_key(&mut self, code: KeyCode) -> bool {
        let len = self.items.len();
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

// ── Empty ──

struct ListEmpty;

impl Ingredient for ListEmpty {
    fn group(&self) -> &str { "List" }
    fn name(&self) -> &str { "Empty" }
    fn source(&self) -> &str { "ratatui::widgets::List" }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let list = RatatuiList::new(Vec::<ListItem>::new())
            .block(
                Block::bordered()
                    .title(" Empty List ")
                    .title_style(Style::default().fg(MOCHA.text))
                    .border_style(Style::default().fg(MOCHA.border)),
            )
            .style(Style::default().bg(MOCHA.surface));

        Widget::render(list, area, buf);
    }
}
