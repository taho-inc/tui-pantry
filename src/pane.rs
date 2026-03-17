use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Widget},
};

use crate::theme::PantryTheme;
use crate::Ingredient;

/// Chrome wrapper that renders a titled border around an ingredient.
pub struct Pane<'a> {
    title: &'a str,
    ingredient: &'a dyn Ingredient,
    focused: bool,
    theme: &'a PantryTheme,
}

impl<'a> Pane<'a> {
    pub fn new(
        title: &'a str,
        ingredient: &'a dyn Ingredient,
        focused: bool,
        theme: &'a PantryTheme,
    ) -> Self {
        Self {
            title,
            ingredient,
            focused,
            theme,
        }
    }
}

impl Widget for Pane<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border_color = if self.focused {
            self.theme.accent
        } else {
            Color::DarkGray
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(format!(" {} ", self.title))
            .title_style(Style::default().fg(self.theme.text_dim));

        let inner = block.inner(area);
        block.render(area, buf);
        self.ingredient.render(inner, buf);
    }
}
