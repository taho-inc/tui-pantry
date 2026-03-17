use ratatui::{
    buffer::Buffer,
    crossterm::event::KeyCode,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};
use tui_pantry::{Ingredient, PropInfo};

use crate::styles::MOCHA;

pub mod ingredient {
    use super::*;

    pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
        vec![Box::new(LogStreamDefault::new()), Box::new(LogStreamQuiet)]
    }
}

#[derive(Clone, Copy)]
enum Severity {
    Info,
    Warn,
    Error,
}

impl Severity {
    fn label(self) -> &'static str {
        match self {
            Severity::Info => "INFO ",
            Severity::Warn => "WARN ",
            Severity::Error => "ERROR",
        }
    }

    fn color(self) -> ratatui::style::Color {
        match self {
            Severity::Info => MOCHA.text_dim,
            Severity::Warn => MOCHA.warn,
            Severity::Error => MOCHA.critical,
        }
    }
}

struct LogEntry {
    time: &'static str,
    severity: Severity,
    message: &'static str,
}

const ENTRIES: &[LogEntry] = &[
    LogEntry {
        time: "14:32:01",
        severity: Severity::Info,
        message: "peer connected: taho-9a2f",
    },
    LogEntry {
        time: "14:32:03",
        severity: Severity::Info,
        message: "scene sync started (3 statelets)",
    },
    LogEntry {
        time: "14:32:04",
        severity: Severity::Warn,
        message: "content fetch timeout: bafy...k7m2 (retry 1/3)",
    },
    LogEntry {
        time: "14:32:05",
        severity: Severity::Info,
        message: "quorum reached (4/5 peers)",
    },
    LogEntry {
        time: "14:32:07",
        severity: Severity::Error,
        message: "statelet 'ram' event apply failed: CausalGap",
    },
    LogEntry {
        time: "14:32:08",
        severity: Severity::Info,
        message: "content fetch succeeded: bafy...k7m2",
    },
    LogEntry {
        time: "14:32:10",
        severity: Severity::Info,
        message: "peer connected: taho-c1d8",
    },
    LogEntry {
        time: "14:32:12",
        severity: Severity::Warn,
        message: "high memory pressure: 87% used",
    },
    LogEntry {
        time: "14:32:14",
        severity: Severity::Info,
        message: "inference request queued (txt2img)",
    },
    LogEntry {
        time: "14:32:15",
        severity: Severity::Info,
        message: "inference complete: 340ms",
    },
    LogEntry {
        time: "14:32:18",
        severity: Severity::Info,
        message: "peer disconnected: taho-3e7b (timeout)",
    },
    LogEntry {
        time: "14:32:20",
        severity: Severity::Error,
        message: "swarm transport error: connection reset",
    },
    LogEntry {
        time: "14:32:22",
        severity: Severity::Info,
        message: "reconnecting to taho-3e7b...",
    },
    LogEntry {
        time: "14:32:24",
        severity: Severity::Info,
        message: "peer connected: taho-3e7b",
    },
];

const DESCRIPTION: &str = "Timestamped entries with severity coloring and scroll";

const PROPS: &[PropInfo] = &[
    PropInfo {
        name: "timestamp_width",
        ty: "u16",
        description: "Fixed gutter for time column",
    },
    PropInfo {
        name: "severity",
        ty: "Severity",
        description: "INFO/WARN/ERROR with mapped color",
    },
    PropInfo {
        name: "scroll",
        ty: "usize",
        description: "Vertical scroll offset",
    },
];

const QUIET_ENTRIES: &[LogEntry] = &[
    LogEntry {
        time: "14:32:01",
        severity: Severity::Info,
        message: "peer connected: taho-9a2f",
    },
    LogEntry {
        time: "14:32:05",
        severity: Severity::Info,
        message: "quorum reached (4/5 peers)",
    },
    LogEntry {
        time: "14:32:14",
        severity: Severity::Info,
        message: "inference request queued (txt2img)",
    },
    LogEntry {
        time: "14:32:15",
        severity: Severity::Info,
        message: "inference complete: 340ms",
    },
];

fn render_entry(entry: &LogEntry, area: Rect, buf: &mut Buffer) {
    Line::from(vec![
        Span::styled(entry.time, Style::default().fg(MOCHA.text_disabled)),
        Span::raw(" "),
        Span::styled(
            entry.severity.label(),
            Style::default()
                .fg(entry.severity.color())
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(entry.message, Style::default().fg(MOCHA.text)),
    ])
    .render(area, buf);
}

// ── Default (interactive, scrollable) ──

struct LogStreamDefault {
    scroll: usize,
}

impl LogStreamDefault {
    fn new() -> Self {
        Self { scroll: 0 }
    }
}

impl Ingredient for LogStreamDefault {
    fn group(&self) -> &str {
        "Log Stream"
    }
    fn name(&self) -> &str {
        "Default"
    }
    fn source(&self) -> &str {
        "example_pantry::widgets::log_stream"
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
        let visible = area.height as usize;
        let entries = &ENTRIES[self.scroll..ENTRIES.len().min(self.scroll + visible)];
        for (i, entry) in entries.iter().enumerate() {
            let y = area.y + i as u16;
            if y >= area.bottom() {
                break;
            }
            render_entry(
                entry,
                Rect {
                    y,
                    height: 1,
                    ..area
                },
                buf,
            );
        }
    }

    fn handle_key(&mut self, code: KeyCode) -> bool {
        let max = ENTRIES.len().saturating_sub(1);
        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.scroll = self.scroll.saturating_sub(1);
                true
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.scroll = (self.scroll + 1).min(max);
                true
            }
            _ => false,
        }
    }
}

// ── Quiet ──

struct LogStreamQuiet;

impl Ingredient for LogStreamQuiet {
    fn group(&self) -> &str {
        "Log Stream"
    }
    fn name(&self) -> &str {
        "Quiet"
    }
    fn source(&self) -> &str {
        "example_pantry::widgets::log_stream"
    }
    fn description(&self) -> &str {
        DESCRIPTION
    }
    fn props(&self) -> &[PropInfo] {
        PROPS
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        for (i, entry) in QUIET_ENTRIES.iter().enumerate() {
            let y = area.y + i as u16;
            if y >= area.bottom() {
                break;
            }
            render_entry(
                entry,
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
