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
        vec![
            Box::new(KeyValueDefault),
            Box::new(KeyValueDense),
        ]
    }
}

struct Entry {
    label: &'static str,
    value: &'static str,
}

fn render_entries(entries: &[Entry], spacing: u16, area: Rect, buf: &mut Buffer) {
    let label_width = entries.iter().map(|e| e.label.len()).max().unwrap_or(0);

    let rows = Layout::vertical(vec![Constraint::Length(spacing); entries.len()]).split(area);

    for (entry, row) in entries.iter().zip(rows.iter()) {
        Line::from(vec![
            Span::styled(
                format!("{:>width$}", entry.label, width = label_width),
                Style::default().fg(MOCHA.text_dim),
            ),
            Span::raw("  "),
            Span::styled(
                entry.value,
                Style::default().fg(MOCHA.text).add_modifier(Modifier::BOLD),
            ),
        ])
        .render(Rect { height: 1, ..*row }, buf);
    }
}

const DESCRIPTION: &str = "Right-aligned labels with bold values, spaced for scannability";

const PROPS: &[PropInfo] = &[
    PropInfo { name: "label_width", ty: "usize", description: "Derived from longest label for alignment" },
    PropInfo { name: "spacing", ty: "u16", description: "Vertical gap between entries" },
];

// ── Default ──

struct KeyValueDefault;

const SAMPLE_ENTRIES: &[Entry] = &[
    Entry { label: "Node ID", value: "taho-7f3a9b" },
    Entry { label: "Region", value: "us-east-1" },
    Entry { label: "Uptime", value: "14d 7h 32m" },
    Entry { label: "Peers", value: "23" },
    Entry { label: "Version", value: "0.4.1-rc.2" },
];

impl Ingredient for KeyValueDefault {
    fn group(&self) -> &str { "Key Value" }
    fn name(&self) -> &str { "Default" }
    fn source(&self) -> &str { "example_pantry::widgets::key_value" }
    fn description(&self) -> &str { DESCRIPTION }
    fn props(&self) -> &[PropInfo] { PROPS }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        render_entries(SAMPLE_ENTRIES, 2, area, buf);
    }
}

// ── Dense ──

struct KeyValueDense;

impl Ingredient for KeyValueDense {
    fn group(&self) -> &str { "Key Value" }
    fn name(&self) -> &str { "Dense" }
    fn source(&self) -> &str { "example_pantry::widgets::key_value" }
    fn description(&self) -> &str { DESCRIPTION }
    fn props(&self) -> &[PropInfo] { PROPS }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        render_entries(SAMPLE_ENTRIES, 1, area, buf);
    }
}
