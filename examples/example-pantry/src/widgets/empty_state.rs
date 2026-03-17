use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};
use tui_pantry::{Ingredient, PropInfo, layout::render_centered};

use crate::styles::MOCHA;

pub mod ingredient {
    use super::*;

    pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
        vec![
            Box::new(EmptyStateNoData),
            Box::new(EmptyStateLoading),
            Box::new(EmptyStateError),
        ]
    }
}

const DESCRIPTION: &str = "Placeholder for panels awaiting data";

const PROPS: &[PropInfo] = &[
    PropInfo {
        name: "icon",
        ty: "&str",
        description: "Symbol or emoji above the title",
    },
    PropInfo {
        name: "title",
        ty: "&str",
        description: "Primary message",
    },
    PropInfo {
        name: "hint",
        ty: "&str",
        description: "Secondary guidance text",
    },
];

struct EmptyContent {
    icon: &'static str,
    title: &'static str,
    hint: &'static str,
    title_color: ratatui::style::Color,
}

struct EmptyWidget(&'static EmptyContent);

impl Widget for EmptyWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let c = self.0;
        if area.height < 3 {
            return;
        }

        let icon_y = area.y;
        let title_y = area.y + 1;
        let hint_y = area.y + 2;

        Line::from(Span::styled(c.icon, Style::default().fg(c.title_color)))
            .centered()
            .render(
                Rect {
                    y: icon_y,
                    height: 1,
                    ..area
                },
                buf,
            );

        Line::from(Span::styled(
            c.title,
            Style::default()
                .fg(c.title_color)
                .add_modifier(Modifier::BOLD),
        ))
        .centered()
        .render(
            Rect {
                y: title_y,
                height: 1,
                ..area
            },
            buf,
        );

        Line::from(Span::styled(
            c.hint,
            Style::default().fg(MOCHA.text_disabled),
        ))
        .centered()
        .render(
            Rect {
                y: hint_y,
                height: 1,
                ..area
            },
            buf,
        );
    }
}

// ── No Data ──

struct EmptyStateNoData;

const NO_DATA: EmptyContent = EmptyContent {
    icon: "∅",
    title: "No data yet",
    hint: "Data will appear once peers connect",
    title_color: MOCHA.text_dim,
};

impl Ingredient for EmptyStateNoData {
    fn group(&self) -> &str {
        "Empty State"
    }
    fn name(&self) -> &str {
        "No Data"
    }
    fn source(&self) -> &str {
        "example_pantry::widgets::empty_state"
    }
    fn description(&self) -> &str {
        DESCRIPTION
    }
    fn props(&self) -> &[PropInfo] {
        PROPS
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        render_centered(
            EmptyWidget(&NO_DATA),
            None,
            Some(Constraint::Length(3)),
            area,
            buf,
        );
    }
}

// ── Loading ──

struct EmptyStateLoading;

const LOADING: EmptyContent = EmptyContent {
    icon: "⟳",
    title: "Loading…",
    hint: "Fetching state from the network",
    title_color: MOCHA.info,
};

impl Ingredient for EmptyStateLoading {
    fn group(&self) -> &str {
        "Empty State"
    }
    fn name(&self) -> &str {
        "Loading"
    }
    fn source(&self) -> &str {
        "example_pantry::widgets::empty_state"
    }
    fn description(&self) -> &str {
        DESCRIPTION
    }
    fn props(&self) -> &[PropInfo] {
        PROPS
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        render_centered(
            EmptyWidget(&LOADING),
            None,
            Some(Constraint::Length(3)),
            area,
            buf,
        );
    }
}

// ── Error ──

struct EmptyStateError;

const ERROR: EmptyContent = EmptyContent {
    icon: "✗",
    title: "Connection failed",
    hint: "Check network settings and retry",
    title_color: MOCHA.critical,
};

impl Ingredient for EmptyStateError {
    fn group(&self) -> &str {
        "Empty State"
    }
    fn name(&self) -> &str {
        "Error"
    }
    fn source(&self) -> &str {
        "example_pantry::widgets::empty_state"
    }
    fn description(&self) -> &str {
        DESCRIPTION
    }
    fn props(&self) -> &[PropInfo] {
        PROPS
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        render_centered(
            EmptyWidget(&ERROR),
            None,
            Some(Constraint::Length(3)),
            area,
            buf,
        );
    }
}
