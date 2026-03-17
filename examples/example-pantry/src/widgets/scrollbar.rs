use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    layout::{Margin, Rect},
    style::Style,
    text::Line,
    widgets::{
        Block, Paragraph, Scrollbar as RatatuiScrollbar, ScrollbarOrientation, ScrollbarState,
        StatefulWidget, Widget, Wrap,
    },
};
use tui_pantry::Ingredient;

use crate::styles::MOCHA;

pub mod ingredient {
    use super::*;

    pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
        vec![
            Box::new(ScrollbarVertical::new()),
            Box::new(ScrollbarHorizontal::new()),
        ]
    }
}

const SAMPLE_TEXT: &[&str] = &[
    "Line 1: The quick brown fox jumps over the lazy dog.",
    "Line 2: Pack my box with five dozen liquor jugs.",
    "Line 3: How vexingly quick daft zebras jump.",
    "Line 4: The five boxing wizards jump quickly.",
    "Line 5: Jinxed wizards pluck ivy from the big quilt.",
    "Line 6: Crazy Frederick bought many very exquisite opal jewels.",
    "Line 7: We promptly judged antique ivory buckles for the next prize.",
    "Line 8: A mad boxer shot a quick, gloved jab to the jaw of his dizzy opponent.",
    "Line 9: Jaded zombies acted quaintly but kept driving their oxen forward.",
    "Line 10: The job requires extra pluck and zeal from every young wage earner.",
    "Line 11: Few quips galvanized the mock jury box.",
    "Line 12: Quick zephyrs blow, vexing daft Jim.",
    "Line 13: Two driven jocks help fax my big quiz.",
    "Line 14: The lazy major was fixing Cupid's broken quiver.",
    "Line 15: Sixty zippers were quickly picked from the woven jute bag.",
];

// ── Vertical (interactive) ──

struct ScrollbarVertical {
    position: usize,
}

impl ScrollbarVertical {
    fn new() -> Self {
        Self { position: 0 }
    }
}

impl Ingredient for ScrollbarVertical {
    fn group(&self) -> &str {
        "Scrollbar"
    }
    fn name(&self) -> &str {
        "Vertical"
    }
    fn source(&self) -> &str {
        "ratatui::widgets::Scrollbar"
    }
    fn interactive(&self) -> bool {
        true
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title(" Scrollable Text ")
            .title_style(Style::default().fg(MOCHA.text))
            .border_style(Style::default().fg(MOCHA.border));

        let lines: Vec<Line> = SAMPLE_TEXT
            .iter()
            .map(|s| Line::from(*s).style(Style::default().fg(MOCHA.text)))
            .collect();

        Paragraph::new(lines)
            .block(block)
            .style(Style::default().bg(MOCHA.surface))
            .scroll((self.position as u16, 0))
            .render(area, buf);

        let mut scrollbar_state = ScrollbarState::new(SAMPLE_TEXT.len()).position(self.position);

        RatatuiScrollbar::new(ScrollbarOrientation::VerticalRight)
            .thumb_style(Style::default().fg(MOCHA.accent))
            .track_style(Style::default().fg(MOCHA.border))
            .render(area.inner(Margin::new(0, 1)), buf, &mut scrollbar_state);
    }

    fn handle_key(&mut self, code: KeyCode) -> bool {
        let max = SAMPLE_TEXT.len().saturating_sub(1);
        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.position = self.position.saturating_sub(1);
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.position = (self.position + 1).min(max);
                true
            }
            _ => false,
        }
    }
}

// ── Horizontal ──

struct ScrollbarHorizontal {
    position: usize,
}

impl ScrollbarHorizontal {
    fn new() -> Self {
        Self { position: 0 }
    }
}

impl Ingredient for ScrollbarHorizontal {
    fn group(&self) -> &str {
        "Scrollbar"
    }
    fn name(&self) -> &str {
        "Horizontal"
    }
    fn source(&self) -> &str {
        "ratatui::widgets::Scrollbar"
    }
    fn interactive(&self) -> bool {
        true
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let wide_line = "← This line is very long and demonstrates horizontal scrolling. \
            It keeps going and going to show how the scrollbar tracks the horizontal position. \
            Almost there… and done! →";

        let block = Block::bordered()
            .title(" Wide Content ")
            .title_style(Style::default().fg(MOCHA.text))
            .border_style(Style::default().fg(MOCHA.border));

        Paragraph::new(Line::from(wide_line).style(Style::default().fg(MOCHA.text)))
            .block(block)
            .style(Style::default().bg(MOCHA.surface))
            .wrap(Wrap { trim: false })
            .scroll((0, self.position as u16))
            .render(area, buf);

        let mut scrollbar_state = ScrollbarState::new(wide_line.len()).position(self.position);

        RatatuiScrollbar::new(ScrollbarOrientation::HorizontalBottom)
            .thumb_style(Style::default().fg(MOCHA.accent))
            .track_style(Style::default().fg(MOCHA.border))
            .render(area.inner(Margin::new(1, 0)), buf, &mut scrollbar_state);
    }

    fn handle_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Left | KeyCode::Char('h') => {
                self.position = self.position.saturating_sub(2);
                true
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.position = (self.position + 2).min(200);
                true
            }
            _ => false,
        }
    }
}
