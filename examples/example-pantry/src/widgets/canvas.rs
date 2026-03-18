use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    symbols::Marker,
    widgets::{
        canvas::{Canvas as RatatuiCanvas, Circle, Line, Map, MapResolution, Rectangle},
        Block, Widget,
    },
};
use tui_pantry::Ingredient;

use crate::styles::{palette::accent, MOCHA};

pub mod ingredient {
    use super::*;

    pub fn ingredients() -> Vec<Box<dyn Ingredient>> {
        vec![Box::new(CanvasShapes), Box::new(CanvasMap)]
    }
}

// ── Shapes ──

struct CanvasShapes;

impl Ingredient for CanvasShapes {
    fn group(&self) -> &str {
        "Canvas"
    }
    fn name(&self) -> &str {
        "Shapes"
    }
    fn source(&self) -> &str {
        "ratatui::widgets::canvas::Canvas"
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        RatatuiCanvas::default()
            .block(
                Block::bordered()
                    .title(" Canvas Shapes ")
                    .title_style(Style::default().fg(MOCHA.text))
                    .border_style(Style::default().fg(MOCHA.border)),
            )
            .marker(Marker::Braille)
            .x_bounds([0.0, 100.0])
            .y_bounds([0.0, 100.0])
            .paint(|ctx| {
                ctx.draw(&Circle {
                    x: 50.0,
                    y: 50.0,
                    radius: 30.0,
                    color: accent::BLUE,
                });
                ctx.draw(&Rectangle {
                    x: 10.0,
                    y: 10.0,
                    width: 30.0,
                    height: 30.0,
                    color: accent::GREEN,
                });
                ctx.draw(&Line {
                    x1: 0.0,
                    y1: 0.0,
                    x2: 100.0,
                    y2: 100.0,
                    color: accent::RED,
                });
                ctx.draw(&Line {
                    x1: 100.0,
                    y1: 0.0,
                    x2: 0.0,
                    y2: 100.0,
                    color: accent::MAUVE,
                });
            })
            .background_color(MOCHA.surface)
            .render(area, buf);
    }
}

// ── Map ──

struct CanvasMap;

impl Ingredient for CanvasMap {
    fn group(&self) -> &str {
        "Canvas"
    }
    fn name(&self) -> &str {
        "World Map"
    }
    fn source(&self) -> &str {
        "ratatui::widgets::canvas::Canvas"
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        RatatuiCanvas::default()
            .block(
                Block::bordered()
                    .title(" World Map ")
                    .title_style(Style::default().fg(MOCHA.text))
                    .border_style(Style::default().fg(MOCHA.border)),
            )
            .marker(Marker::Braille)
            .x_bounds([-180.0, 180.0])
            .y_bounds([-90.0, 90.0])
            .paint(|ctx| {
                ctx.draw(&Map {
                    resolution: MapResolution::Low,
                    color: accent::TEAL,
                });
            })
            .background_color(MOCHA.surface)
            .render(area, buf);
    }
}
