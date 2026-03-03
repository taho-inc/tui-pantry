use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::Widget,
};

/// Render a widget centered within the given area.
///
/// Each constraint controls its axis; equal fill regions absorb the
/// remainder. Pass `None` for an axis to skip centering on that axis.
pub fn render_centered(
    widget: impl Widget,
    width: Option<Constraint>,
    height: Option<Constraint>,
    area: Rect,
    buf: &mut Buffer,
) {
    let h_area = match width {
        Some(w) => {
            let [_, center, _] =
                Layout::horizontal([Constraint::Fill(1), w, Constraint::Fill(1)]).areas(area);
            center
        }
        None => area,
    };
    let center = match height {
        Some(h) => {
            let [_, center, _] =
                Layout::vertical([Constraint::Fill(1), h, Constraint::Fill(1)]).areas(h_area);
            center
        }
        None => h_area,
    };
    widget.render(center, buf);
}
