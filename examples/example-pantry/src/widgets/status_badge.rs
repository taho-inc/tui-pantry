use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};
use tui_pantry::{Ingredient, PropInfo};

use crate::styles::MOCHA;

pub mod ingredient {
    use super::*;

    pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
        vec![Box::new(StatusBadgeInline), Box::new(StatusBadgePill)]
    }
}

struct Status {
    label: &'static str,
    color: ratatui::style::Color,
}

const DESCRIPTION: &str = "Colored dot + label for at-a-glance status";

const PROPS: &[PropInfo] = &[
    PropInfo {
        name: "color",
        ty: "Color",
        description: "Dot and label color mapped from state",
    },
    PropInfo {
        name: "label",
        ty: "&str",
        description: "Status text",
    },
];

const STATUSES: &[Status] = &[
    Status {
        label: "Healthy",
        color: MOCHA.ok,
    },
    Status {
        label: "Degraded",
        color: MOCHA.warn,
    },
    Status {
        label: "Offline",
        color: MOCHA.critical,
    },
    Status {
        label: "Unknown",
        color: MOCHA.text_disabled,
    },
];

// ── Inline ──

struct StatusBadgeInline;

impl Ingredient for StatusBadgeInline {
    fn section(&self) -> Option<&str> {
        Some("Chrome")
    }

    fn group(&self) -> &str {
        "Status Badge"
    }
    fn name(&self) -> &str {
        "Inline"
    }
    fn source(&self) -> &str {
        "example_pantry::widgets::status_badge"
    }
    fn description(&self) -> &str {
        DESCRIPTION
    }
    fn props(&self) -> &[PropInfo] {
        PROPS
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let rows = Layout::vertical(vec![Constraint::Length(1); STATUSES.len()]).split(area);

        for (status, row) in STATUSES.iter().zip(rows.iter()) {
            Line::from(vec![
                Span::styled("● ", Style::default().fg(status.color)),
                Span::styled(status.label, Style::default().fg(status.color)),
            ])
            .render(Rect { height: 1, ..*row }, buf);
        }
    }
}

// ── Pill ──

struct StatusBadgePill;

impl Ingredient for StatusBadgePill {
    fn section(&self) -> Option<&str> {
        Some("Chrome")
    }

    fn group(&self) -> &str {
        "Status Badge"
    }
    fn name(&self) -> &str {
        "Pill"
    }
    fn source(&self) -> &str {
        "example_pantry::widgets::status_badge"
    }
    fn description(&self) -> &str {
        DESCRIPTION
    }
    fn props(&self) -> &[PropInfo] {
        PROPS
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let rows = Layout::vertical(vec![Constraint::Length(2); STATUSES.len()]).split(area);

        for (status, row) in STATUSES.iter().zip(rows.iter()) {
            let pill = format!(" {} ", status.label);
            Line::from(Span::styled(
                pill,
                Style::default()
                    .bg(status.color)
                    .fg(MOCHA.surface)
                    .add_modifier(Modifier::BOLD),
            ))
            .render(Rect { height: 1, ..*row }, buf);
        }
    }
}
