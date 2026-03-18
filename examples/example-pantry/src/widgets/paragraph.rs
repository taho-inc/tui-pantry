use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph as RatatuiParagraph, Widget, Wrap},
};
use tui_pantry::Ingredient;

use crate::styles::{palette::accent, MOCHA};

pub mod ingredient {
    use super::*;

    pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
        vec![
            Box::new(ParagraphWrapped),
            Box::new(ParagraphAligned),
            Box::new(ParagraphStyled),
        ]
    }
}

const SAMPLE_TEXT: &str = "\
Ratatui is a Rust library for building terminal user interfaces. \
It provides a set of widgets and utilities to create rich, \
interactive applications that run in the terminal. \
The library emphasizes composability and simplicity.";

// ── Wrapped ──

struct ParagraphWrapped;

impl Ingredient for ParagraphWrapped {
    fn group(&self) -> &str {
        "Paragraph"
    }
    fn name(&self) -> &str {
        "Wrapped"
    }
    fn source(&self) -> &str {
        "ratatui::widgets::Paragraph"
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        RatatuiParagraph::new(SAMPLE_TEXT)
            .style(Style::default().fg(MOCHA.text).bg(MOCHA.surface))
            .block(
                Block::bordered()
                    .title(" Wrapped Text ")
                    .title_style(Style::default().fg(MOCHA.text))
                    .border_style(Style::default().fg(MOCHA.border)),
            )
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }
}

// ── Aligned ──

struct ParagraphAligned;

impl Ingredient for ParagraphAligned {
    fn group(&self) -> &str {
        "Paragraph"
    }
    fn name(&self) -> &str {
        "Center Aligned"
    }
    fn source(&self) -> &str {
        "ratatui::widgets::Paragraph"
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        RatatuiParagraph::new(SAMPLE_TEXT)
            .style(Style::default().fg(MOCHA.text).bg(MOCHA.surface))
            .block(
                Block::bordered()
                    .title(" Centered ")
                    .title_style(Style::default().fg(MOCHA.text))
                    .border_style(Style::default().fg(MOCHA.border)),
            )
            .alignment(ratatui::layout::Alignment::Center)
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }
}

// ── Styled Spans ──

struct ParagraphStyled;

impl Ingredient for ParagraphStyled {
    fn group(&self) -> &str {
        "Paragraph"
    }
    fn name(&self) -> &str {
        "Styled Spans"
    }
    fn source(&self) -> &str {
        "ratatui::widgets::Paragraph"
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let lines = vec![
            Line::from(vec![
                Span::styled(
                    "Ratatui",
                    Style::default()
                        .fg(accent::MAUVE)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" is a "),
                Span::styled(
                    "Rust",
                    Style::default()
                        .fg(accent::PEACH)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" library for building "),
                Span::styled("terminal UIs", Style::default().fg(accent::GREEN)),
                Span::raw("."),
            ]),
            Line::raw(""),
            Line::from(vec![
                Span::raw("Status: "),
                Span::styled(
                    "OK",
                    Style::default().fg(MOCHA.ok).add_modifier(Modifier::BOLD),
                ),
                Span::raw("  Warnings: "),
                Span::styled("2", Style::default().fg(MOCHA.warn)),
                Span::raw("  Errors: "),
                Span::styled("0", Style::default().fg(MOCHA.critical)),
            ]),
        ];

        RatatuiParagraph::new(lines)
            .style(Style::default().fg(MOCHA.text).bg(MOCHA.surface))
            .block(
                Block::bordered()
                    .title(" Styled Spans ")
                    .title_style(Style::default().fg(MOCHA.text))
                    .border_style(Style::default().fg(MOCHA.border)),
            )
            .render(area, buf);
    }
}
