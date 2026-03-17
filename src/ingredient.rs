use ratatui::crossterm::event::{KeyCode, MouseEvent, MouseEventKind};
use ratatui::layout::Rect;

/// Describes a single configurable property of a widget.
pub struct PropInfo {
    pub name: &'static str,
    pub ty: &'static str,
    pub description: &'static str,
}

/// A renderable widget variant for the pantry harness.
///
/// Each ingredient represents one "story" — a specific configuration
/// of a widget with mock data. Ingredients are grouped by widget name
/// in the navigation tree.
pub trait Ingredient: Send + Sync {
    /// Top-level tab this ingredient belongs to (e.g., "Widgets", "Views", "Styles").
    fn tab(&self) -> &str {
        "Widgets"
    }

    /// Widget group name displayed as a tree parent (e.g., "Node Table").
    fn group(&self) -> &str;

    /// Variant name displayed as a tree child (e.g., "Default").
    fn name(&self) -> &str;

    /// Module path for source attribution (e.g., "taho_tui::widgets::node_table").
    fn source(&self) -> &str;

    /// One-line description of the widget's purpose.
    fn description(&self) -> &str {
        ""
    }

    /// Configurable properties exposed by the widget.
    fn props(&self) -> &[PropInfo] {
        &[]
    }

    /// Render this ingredient's widget into the given area.
    fn render(&self, area: Rect, buf: &mut ratatui::buffer::Buffer);

    /// Whether this ingredient needs periodic redraws (e.g. animations).
    fn animated(&self) -> bool {
        false
    }

    /// Whether this ingredient accepts keyboard and mouse input when focused.
    fn interactive(&self) -> bool {
        false
    }

    /// Handle a key press while the preview pane is focused.
    /// Returns true if the key was consumed.
    fn handle_key(&mut self, _code: KeyCode) -> bool {
        false
    }

    /// Handle a mouse event while the preview pane is focused.
    ///
    /// `area` is the ingredient's render area (inside the pane border),
    /// matching what was passed to `render()`. Mouse coordinates are absolute
    /// terminal positions — use `area.contains(Position::new(col, row))` to
    /// hit-test.
    fn handle_mouse(&mut self, _event: MouseEvent, _area: Rect) -> bool {
        false
    }
}

/// Returns true if the mouse event is a left-click.
pub fn is_click(event: &MouseEvent) -> bool {
    matches!(
        event.kind,
        MouseEventKind::Down(ratatui::crossterm::event::MouseButton::Left)
    )
}
