use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    layout::{Constraint, Rect},
    style::{Modifier, Style},
    widgets::{Block, Cell, Row, StatefulWidget, Table as RatatuiTable, TableState},
};
use tui_pantry::{Ingredient, PropInfo};

use crate::styles::MOCHA;

pub mod ingredient {
    use super::*;

    pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
        vec![Box::new(TableDefault::new()), Box::new(TableEmpty)]
    }
}

struct SampleRow {
    name: &'static str,
    kind: &'static str,
    version: &'static str,
    status: &'static str,
}

fn sample_rows() -> Vec<SampleRow> {
    vec![
        SampleRow {
            name: "Paragraph",
            kind: "Widget",
            version: "0.30",
            status: "Stable",
        },
        SampleRow {
            name: "List",
            kind: "Widget",
            version: "0.30",
            status: "Stable",
        },
        SampleRow {
            name: "Table",
            kind: "Widget",
            version: "0.30",
            status: "Stable",
        },
        SampleRow {
            name: "BarChart",
            kind: "Widget",
            version: "0.30",
            status: "Stable",
        },
        SampleRow {
            name: "Canvas",
            kind: "Widget",
            version: "0.30",
            status: "Unstable",
        },
        SampleRow {
            name: "Calendar",
            kind: "Widget",
            version: "0.30",
            status: "Unstable",
        },
        SampleRow {
            name: "Sparkline",
            kind: "Widget",
            version: "0.30",
            status: "Stable",
        },
        SampleRow {
            name: "Gauge",
            kind: "Widget",
            version: "0.30",
            status: "Stable",
        },
    ]
}

fn status_color(status: &str) -> ratatui::style::Color {
    match status {
        "Stable" => MOCHA.ok,
        "Unstable" => MOCHA.warn,
        _ => MOCHA.text_dim,
    }
}

const DESCRIPTION: &str = "Row-selectable data table with column constraints";

const PROPS: &[PropInfo] = &[
    PropInfo {
        name: "header",
        ty: "Row",
        description: "Fixed header row with bottom margin",
    },
    PropInfo {
        name: "widths",
        ty: "&[Constraint]",
        description: "Column width constraints",
    },
    PropInfo {
        name: "block",
        ty: "Block",
        description: "Surrounding border and title",
    },
    PropInfo {
        name: "row_highlight_style",
        ty: "Style",
        description: "Style applied to the selected row",
    },
    PropInfo {
        name: "highlight_symbol",
        ty: "&str",
        description: "Prefix glyph for the selected row",
    },
];

// ── Default (interactive) ──

struct TableDefault {
    rows: Vec<SampleRow>,
    state: TableState,
}

impl TableDefault {
    fn new() -> Self {
        let rows = sample_rows();
        let mut state = TableState::default();
        state.select(Some(0));
        Self { rows, state }
    }
}

impl Ingredient for TableDefault {
    fn group(&self) -> &str {
        "Table"
    }
    fn name(&self) -> &str {
        "Default"
    }
    fn source(&self) -> &str {
        "ratatui::widgets::Table"
    }
    fn description(&self) -> &str {
        DESCRIPTION
    }
    fn interactive(&self) -> bool {
        true
    }
    fn props(&self) -> &[PropInfo] {
        PROPS
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let header = Row::new(["Name", "Kind", "Version", "Status"])
            .style(Style::default().fg(MOCHA.text).add_modifier(Modifier::BOLD))
            .bottom_margin(1);

        let rows: Vec<Row> = self
            .rows
            .iter()
            .map(|r| {
                Row::new([
                    Cell::from(r.name).style(Style::default().fg(MOCHA.text)),
                    Cell::from(r.kind).style(Style::default().fg(MOCHA.text_dim)),
                    Cell::from(r.version).style(Style::default().fg(MOCHA.text_dim)),
                    Cell::from(r.status).style(Style::default().fg(status_color(r.status))),
                ])
            })
            .collect();

        let table = RatatuiTable::new(
            rows,
            [
                Constraint::Min(12),
                Constraint::Min(8),
                Constraint::Min(8),
                Constraint::Min(10),
            ],
        )
        .header(header)
        .block(
            Block::bordered()
                .title(" Widgets ")
                .title_style(Style::default().fg(MOCHA.text))
                .border_style(Style::default().fg(MOCHA.border)),
        )
        .style(Style::default().bg(MOCHA.surface))
        .row_highlight_style(
            Style::default()
                .bg(MOCHA.surface_raised)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▸ ");

        let mut state = self.state;
        StatefulWidget::render(table, area, buf, &mut state);
    }

    fn handle_key(&mut self, code: KeyCode) -> bool {
        let len = self.rows.len();
        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                let i = self.state.selected().unwrap_or(0);
                self.state
                    .select(Some(if i == 0 { len.saturating_sub(1) } else { i - 1 }));
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let i = self.state.selected().unwrap_or(0);
                self.state
                    .select(Some(if i >= len - 1 { 0 } else { i + 1 }));
                true
            }
            _ => false,
        }
    }
}

// ── Empty ──

struct TableEmpty;

impl Ingredient for TableEmpty {
    fn group(&self) -> &str {
        "Table"
    }
    fn name(&self) -> &str {
        "Empty"
    }
    fn source(&self) -> &str {
        "ratatui::widgets::Table"
    }
    fn description(&self) -> &str {
        DESCRIPTION
    }
    fn props(&self) -> &[PropInfo] {
        PROPS
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let header = Row::new(["Name", "Kind", "Version", "Status"])
            .style(Style::default().fg(MOCHA.text).add_modifier(Modifier::BOLD))
            .bottom_margin(1);

        let table = RatatuiTable::new(
            Vec::<Row>::new(),
            [
                Constraint::Min(12),
                Constraint::Min(8),
                Constraint::Min(8),
                Constraint::Min(10),
            ],
        )
        .header(header)
        .block(
            Block::bordered()
                .title(" Empty Table ")
                .title_style(Style::default().fg(MOCHA.text))
                .border_style(Style::default().fg(MOCHA.border)),
        )
        .style(Style::default().bg(MOCHA.surface));

        ratatui::widgets::Widget::render(table, area, buf);
    }
}
