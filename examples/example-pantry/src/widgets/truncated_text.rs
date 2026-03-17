use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Widget,
};
use tui_pantry::{Ingredient, PropInfo};

use crate::styles::MOCHA;

pub mod ingredient {
    use super::*;

    pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
        vec![Box::new(TruncatedSingleLine), Box::new(TruncatedMultiLine)]
    }
}

const DESCRIPTION: &str = "Ellipsis truncation for overflow text";

const PROPS: &[PropInfo] = &[
    PropInfo {
        name: "max_width",
        ty: "usize",
        description: "Character limit before truncation",
    },
    PropInfo {
        name: "ellipsis",
        ty: "char",
        description: "Suffix character when truncated (…)",
    },
];

fn truncate_line(text: &str, max_width: usize) -> Line<'_> {
    if text.len() <= max_width {
        Line::from(Span::styled(text, Style::default().fg(MOCHA.text)))
    } else {
        let cut = max_width.saturating_sub(1);
        Line::from(vec![
            Span::styled(&text[..cut], Style::default().fg(MOCHA.text)),
            Span::styled("…", Style::default().fg(MOCHA.text_disabled)),
        ])
    }
}

// ── Single Line ──

struct TruncatedSingleLine;

const LONG_LINE: &str = "The Fabric is a self-forming peer-to-peer network that streams, compiles, and runs AI/ML workloads across distributed nodes with content-addressed storage";

impl Ingredient for TruncatedSingleLine {
    fn group(&self) -> &str {
        "Truncated Text"
    }
    fn name(&self) -> &str {
        "Single Line"
    }
    fn source(&self) -> &str {
        "example_pantry::widgets::truncated_text"
    }
    fn description(&self) -> &str {
        DESCRIPTION
    }
    fn props(&self) -> &[PropInfo] {
        PROPS
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        if area.height == 0 {
            return;
        }
        truncate_line(LONG_LINE, area.width as usize).render(Rect { height: 1, ..area }, buf);
    }
}

// ── Multi Line ──

struct TruncatedMultiLine;

const LINES: &[&str] = &[
    "Node taho-7f3a9b: 23 peers connected, quorum active, inference queue depth 3",
    "Content exchange: 1,247 blocks cached, 98.2% hit rate, 12 pending fetches from swarm",
    "Statelet 'network': tracking 47 nodes across 3 regions with hybrid clock sync",
    "Statelet 'ram': 2.1 GB allocated, 1.8 GB used, last GC 4m ago, no pressure",
    "Transport: QUIC primary (42 streams), WebSocket fallback (5 browser nodes)",
];

impl Ingredient for TruncatedMultiLine {
    fn group(&self) -> &str {
        "Truncated Text"
    }
    fn name(&self) -> &str {
        "Multi Line"
    }
    fn source(&self) -> &str {
        "example_pantry::widgets::truncated_text"
    }
    fn description(&self) -> &str {
        DESCRIPTION
    }
    fn props(&self) -> &[PropInfo] {
        PROPS
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let width = area.width as usize;
        for (i, line) in LINES.iter().enumerate() {
            let y = area.y + (i as u16) * 2;
            if y >= area.bottom() {
                break;
            }
            truncate_line(line, width).render(
                Rect {
                    y,
                    height: 1,
                    ..area
                },
                buf,
            );
        }
    }
}
