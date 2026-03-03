use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Widget},
};

use crate::Ingredient;

/// Chrome wrapper that renders a titled border around an ingredient.
pub struct Pane<'a> {
    title: &'a str,
    ingredient: &'a dyn Ingredient,
    focused: bool,
}

impl<'a> Pane<'a> {
    pub fn new(title: &'a str, ingredient: &'a dyn Ingredient, focused: bool) -> Self {
        Self { title, ingredient, focused }
    }
}

impl Widget for Pane<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border_color = if self.focused {
            Color::Rgb(120, 52, 245)
        } else {
            Color::DarkGray
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(format!(" {} ", self.title))
            .title_style(Style::default().fg(Color::Gray));

        let inner = block.inner(area);
        block.render(area, buf);
        self.ingredient.render(inner, buf);
    }
}
