use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    layout::{Constraint, Layout, Margin, Rect},
    style::Style,
    symbols::Marker,
    text::Line,
    widgets::{
        Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, StatefulWidget, Widget,
        canvas::{Canvas, Circle, Map, MapResolution},
    },
};
use tui_pantry::Ingredient;

use crate::styles::{palette::accent, MOCHA};

pub mod ingredient {
    use super::*;

    pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
        vec![Box::new(ExplorerDefault::new())]
    }
}

// ── Sample log data ──

const LOG_LINES: &[&str] = &[
    "[INFO]  peer 12D3…a4f2 connected via QUIC (us-east)",
    "[INFO]  peer 12D3…b7c1 connected via WebSocket (eu-west)",
    "[INFO]  content PUT abc123 → 3 replicas",
    "[DEBUG] swarm state: Isolated → Quorum (3 peers)",
    "[INFO]  peer 12D3…d9e8 connected via QUIC (ap-south)",
    "[WARN]  peer 12D3…f1a3 latency spike: 340ms",
    "[INFO]  content GET def456 → cache hit",
    "[INFO]  scene timeline: 42 events replayed",
    "[DEBUG] HLC drift: +12ms from peer 12D3…b7c1",
    "[INFO]  peer 12D3…c5d2 connected via WebSocket (us-west)",
    "[ERROR] content PUT ghi789 failed: quorum unavailable",
    "[INFO]  connection rebalance: 5 → 4 peers",
    "[INFO]  peer 12D3…e3f0 disconnected (timeout)",
    "[DEBUG] fabric envelope: origin=12D3…a4f2 follows=[3,7]",
    "[INFO]  content GET jkl012 → fetched from peer 12D3…d9e8",
    "[WARN]  disk usage at 78% on content store",
    "[INFO]  mDNS discovery: 2 new peers on LAN",
    "[INFO]  peer 12D3…g6h4 connected via QUIC (eu-west)",
    "[DEBUG] statelet sync: NumberwangStatelet merged",
    "[INFO]  swarm health: 6 peers, 142ms avg latency",
];

fn log_line_style(line: &str) -> Style {
    if line.contains("[ERROR]") {
        Style::default().fg(MOCHA.critical)
    } else if line.contains("[WARN]") {
        Style::default().fg(MOCHA.warn)
    } else if line.contains("[DEBUG]") {
        Style::default().fg(MOCHA.text_dim)
    } else {
        Style::default().fg(MOCHA.text)
    }
}

// ── Peer locations for map overlay ──

const PEER_LOCATIONS: &[(f64, f64)] = &[
    (-74.0, 40.7),   // us-east (New York)
    (-122.4, 37.8),  // us-west (San Francisco)
    (-0.1, 51.5),    // eu-west (London)
    (77.2, 28.6),    // ap-south (Delhi)
    (139.7, 35.7),   // ap-east (Tokyo)
    (2.3, 48.9),     // eu-west (Paris)
];

// ── Explorer Default (interactive) ──

struct ExplorerDefault {
    scroll_position: usize,
}

impl ExplorerDefault {
    fn new() -> Self {
        Self { scroll_position: 0 }
    }
}

impl Ingredient for ExplorerDefault {
    fn tab(&self) -> &str { "Views" }
    fn group(&self) -> &str { "Explorer" }
    fn name(&self) -> &str { "Default" }
    fn source(&self) -> &str { "example_pantry::views::explorer" }
    fn interactive(&self) -> bool { true }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let [map_col, log_col] =
            Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)])
                .areas(area);

        // ── Map with peer markers ──
        Canvas::default()
            .block(
                Block::bordered()
                    .title(" Peer Network ")
                    .title_style(Style::default().fg(MOCHA.text))
                    .border_style(Style::default().fg(MOCHA.border)),
            )
            .marker(Marker::Braille)
            .x_bounds([-180.0, 180.0])
            .y_bounds([-60.0, 80.0])
            .paint(|ctx| {
                ctx.draw(&Map {
                    resolution: MapResolution::Low,
                    color: MOCHA.border,
                });
                for &(x, y) in PEER_LOCATIONS {
                    ctx.draw(&Circle {
                        x,
                        y,
                        radius: 3.0,
                        color: accent::GREEN,
                    });
                }
            })
            .background_color(MOCHA.surface)
            .render(map_col, buf);

        // ── Scrollable log with scrollbar ──
        let lines: Vec<Line> = LOG_LINES
            .iter()
            .map(|s| Line::from(*s).style(log_line_style(s)))
            .collect();

        Paragraph::new(lines)
            .block(
                Block::bordered()
                    .title(" Event Log ")
                    .title_style(Style::default().fg(MOCHA.text))
                    .border_style(Style::default().fg(MOCHA.border)),
            )
            .style(Style::default().bg(MOCHA.surface))
            .scroll((self.scroll_position as u16, 0))
            .render(log_col, buf);

        let mut scrollbar_state = ScrollbarState::new(LOG_LINES.len())
            .position(self.scroll_position);

        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .thumb_style(Style::default().fg(MOCHA.accent))
            .track_style(Style::default().fg(MOCHA.border))
            .render(log_col.inner(Margin::new(0, 1)), buf, &mut scrollbar_state);
    }

    fn handle_key(&mut self, code: KeyCode) -> bool {
        let max = LOG_LINES.len().saturating_sub(1);
        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.scroll_position = self.scroll_position.saturating_sub(1);
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.scroll_position = (self.scroll_position + 1).min(max);
                true
            }
            _ => false,
        }
    }
}
