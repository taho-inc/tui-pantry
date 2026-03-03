use ratatui::crossterm::event::KeyCode;

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
    fn tab(&self) -> &str { "Widgets" }

    /// Widget group name displayed as a tree parent (e.g., "Node Table").
    fn group(&self) -> &str;

    /// Variant name displayed as a tree child (e.g., "Default").
    fn name(&self) -> &str;

    /// Module path for source attribution (e.g., "taho_tui::widgets::node_table").
    fn source(&self) -> &str;

    /// One-line description of the widget's purpose.
    fn description(&self) -> &str { "" }

    /// Configurable properties exposed by the widget.
    fn props(&self) -> &[PropInfo] { &[] }

    /// Render this ingredient's widget into the given area.
    fn render(&self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer);

    /// Whether this ingredient accepts keyboard input when the preview pane is focused.
    fn interactive(&self) -> bool { false }

    /// Handle a key press while the preview pane is focused.
    /// Returns true if the key was consumed.
    fn handle_key(&mut self, _code: KeyCode) -> bool { false }
}
