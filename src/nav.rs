use crate::Ingredient;

/// Internal section data built during `NavTree::build`.
#[derive(Debug)]
struct SectionData {
    name: String,
    widget_indices: Vec<usize>,
}

/// Entry in the flattened navigation list.
#[derive(Debug)]
pub(crate) enum NavEntry {
    Section {
        section_idx: usize,
        name: String,
        expanded: bool,
    },
    Widget {
        widget_idx: usize,
        name: String,
        expanded: bool,
        /// True when this widget lives inside a section (indented one extra level).
        sectioned: bool,
    },
    Variant {
        widget_idx: usize,
        ingredient_idx: usize,
        /// True when the parent widget lives inside a section.
        sectioned: bool,
    },
}

/// Flattened navigation tree with optional section grouping.
///
/// When ingredients declare `section()`, the hierarchy is
/// Section → Widget → Variant. Without sections, it collapses to
/// Widget → Variant (backward compatible).
pub(crate) struct NavTree {
    sections: Vec<SectionData>,
    section_expanded: Vec<bool>,

    /// Widget names in discovery order.
    pub widgets: Vec<String>,
    /// Ingredient indices grouped by widget.
    pub widget_items: Vec<Vec<usize>>,
    /// Expansion state per widget.
    pub widget_expanded: Vec<bool>,

    /// Widget indices not assigned to any section.
    orphan_widgets: Vec<usize>,

    /// Cursor position in the visible list.
    pub cursor: usize,
    /// First visible entry in the sidebar viewport.
    pub scroll_offset: usize,
}

impl NavTree {
    /// Build the nav tree from ingredients matching a specific tab.
    ///
    /// Sections are derived from `Ingredient::section()`. Widgets whose
    /// ingredients return `None` appear as unsectioned below all sections.
    pub fn build(ingredients: &[Box<dyn Ingredient>], tab: &str) -> Self {
        let mut widgets: Vec<String> = Vec::new();
        let mut widget_items: Vec<Vec<usize>> = Vec::new();

        for (i, ingredient) in ingredients.iter().enumerate() {
            if ingredient.tab() != tab {
                continue;
            }

            let widget = ingredient.group().to_string();

            if let Some(pos) = widgets.iter().position(|w| *w == widget) {
                widget_items[pos].push(i);
            } else {
                widgets.push(widget);
                widget_items.push(vec![i]);
            }
        }

        let mut section_names: Vec<String> = Vec::new();
        let mut section_widgets: Vec<Vec<usize>> = Vec::new();
        let mut orphan_widgets: Vec<usize> = Vec::new();

        for (wi, items) in widget_items.iter().enumerate() {
            let section = items.first().and_then(|&ii| ingredients[ii].section());

            if let Some(name) = section {
                if let Some(si) = section_names.iter().position(|s| s == name) {
                    section_widgets[si].push(wi);
                } else {
                    section_names.push(name.to_string());
                    section_widgets.push(vec![wi]);
                }
            } else {
                orphan_widgets.push(wi);
            }
        }

        let section_data: Vec<SectionData> = section_names
            .into_iter()
            .zip(section_widgets)
            .map(|(name, widget_indices)| SectionData {
                name,
                widget_indices,
            })
            .collect();

        let widget_expanded = vec![true; widgets.len()];
        let section_expanded = vec![true; section_data.len()];

        let mut tree = Self {
            sections: section_data,
            section_expanded,
            widgets,
            widget_items,
            widget_expanded,
            orphan_widgets,
            cursor: 0,
            scroll_offset: 0,
        };

        tree.cursor = tree
            .visible()
            .iter()
            .position(|e| matches!(e, NavEntry::Variant { .. }))
            .unwrap_or(0);

        tree
    }

    fn has_sections(&self) -> bool {
        !self.sections.is_empty()
    }

    /// Flatten visible entries based on expansion state.
    pub fn visible(&self) -> Vec<NavEntry> {
        let mut entries = Vec::new();

        if self.has_sections() {
            for (si, section) in self.sections.iter().enumerate() {
                entries.push(NavEntry::Section {
                    section_idx: si,
                    name: section.name.clone(),
                    expanded: self.section_expanded[si],
                });

                if self.section_expanded[si] {
                    for &wi in &section.widget_indices {
                        self.push_widget_entries(&mut entries, wi, true);
                    }
                }
            }

            for &wi in &self.orphan_widgets {
                self.push_widget_entries(&mut entries, wi, false);
            }
        } else {
            for wi in 0..self.widgets.len() {
                self.push_widget_entries(&mut entries, wi, false);
            }
        }

        entries
    }

    fn push_widget_entries(&self, entries: &mut Vec<NavEntry>, wi: usize, sectioned: bool) {
        entries.push(NavEntry::Widget {
            widget_idx: wi,
            name: self.widgets[wi].clone(),
            expanded: self.widget_expanded[wi],
            sectioned,
        });

        if self.widget_expanded[wi] {
            for &ii in &self.widget_items[wi] {
                entries.push(NavEntry::Variant {
                    widget_idx: wi,
                    ingredient_idx: ii,
                    sectioned,
                });
            }
        }
    }

    /// The currently selected ingredient index, if cursor is on a variant.
    pub fn selected_ingredient(&self) -> Option<usize> {
        let entries = self.visible();
        match entries.get(self.cursor) {
            Some(NavEntry::Variant { ingredient_idx, .. }) => Some(*ingredient_idx),
            _ => None,
        }
    }

    /// All ingredient indices for the widget at cursor, if cursor is on a widget header.
    pub fn selected_widget_items(&self) -> Option<(&str, &[usize])> {
        let entries = self.visible();
        match entries.get(self.cursor) {
            Some(NavEntry::Widget { widget_idx, .. }) => {
                Some((&self.widgets[*widget_idx], &self.widget_items[*widget_idx]))
            }
            _ => None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.widgets.is_empty()
    }

    pub fn move_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_down(&mut self) {
        let max = self.visible().len().saturating_sub(1);
        if self.cursor < max {
            self.cursor += 1;
        }
    }

    /// Jump cursor directly to a visible entry index, clamped to valid range.
    pub fn move_to(&mut self, index: usize) {
        let max = self.visible().len().saturating_sub(1);
        self.cursor = index.min(max);
    }

    /// Toggle expand/collapse if cursor is on a section or widget.
    pub fn toggle_or_enter(&mut self) {
        let entries = self.visible();
        match entries.get(self.cursor) {
            Some(NavEntry::Section { section_idx, .. }) => {
                self.section_expanded[*section_idx] = !self.section_expanded[*section_idx];
            }
            Some(NavEntry::Widget { widget_idx, .. }) => {
                self.widget_expanded[*widget_idx] = !self.widget_expanded[*widget_idx];
            }
            _ => {}
        }
    }

    /// Expand the node at cursor (right arrow).
    pub fn expand(&mut self) {
        let entries = self.visible();
        match entries.get(self.cursor) {
            Some(NavEntry::Section { section_idx, .. }) if !self.section_expanded[*section_idx] => {
                self.section_expanded[*section_idx] = true;
            }
            Some(NavEntry::Widget { widget_idx, .. }) if !self.widget_expanded[*widget_idx] => {
                self.widget_expanded[*widget_idx] = true;
            }
            _ => {}
        }
    }

    /// Collapse the node at cursor, or jump to parent (left arrow).
    ///
    /// - Section → collapse it
    /// - Expanded widget → collapse it
    /// - Collapsed widget inside a section → collapse parent section, jump there
    /// - Variant → collapse parent widget, jump there
    pub fn collapse(&mut self) {
        let entries = self.visible();

        match entries.get(self.cursor) {
            Some(NavEntry::Section { section_idx, .. }) => {
                self.section_expanded[*section_idx] = false;
            }

            Some(NavEntry::Widget { widget_idx, .. }) => {
                let wi = *widget_idx;

                if self.widget_expanded[wi] {
                    self.widget_expanded[wi] = false;
                } else if let Some(si) = self.section_of_widget(wi) {
                    self.section_expanded[si] = false;
                    self.cursor = self.position_of_section(si);
                }
            }

            Some(NavEntry::Variant { widget_idx, .. }) => {
                let wi = *widget_idx;
                self.widget_expanded[wi] = false;
                self.cursor = self.position_of_widget(wi);
            }

            None => {}
        }
    }

    /// Adjust scroll so the cursor stays within the viewport.
    pub fn scroll_into_view(&mut self, viewport_height: usize) {
        if viewport_height == 0 {
            return;
        }

        let total = self.visible().len();
        let max_offset = total.saturating_sub(viewport_height);
        self.scroll_offset = self.scroll_offset.min(max_offset);

        if self.cursor < self.scroll_offset {
            self.scroll_offset = self.cursor;
        } else if self.cursor >= self.scroll_offset + viewport_height {
            self.scroll_offset = self.cursor - viewport_height + 1;
        }
    }

    /// Which section (if any) contains this widget.
    fn section_of_widget(&self, widget_idx: usize) -> Option<usize> {
        self.sections
            .iter()
            .position(|s| s.widget_indices.contains(&widget_idx))
    }

    /// Visible-list position of a section header.
    fn position_of_section(&self, target_si: usize) -> usize {
        let mut pos = 0;

        for (si, section) in self.sections.iter().enumerate() {
            if si == target_si {
                return pos;
            }

            pos += 1;

            if self.section_expanded[si] {
                for &wi in &section.widget_indices {
                    pos += 1;
                    if self.widget_expanded[wi] {
                        pos += self.widget_items[wi].len();
                    }
                }
            }
        }

        pos
    }

    /// Visible-list position of a widget header.
    fn position_of_widget(&self, target_wi: usize) -> usize {
        let mut pos = 0;

        if self.has_sections() {
            for (si, section) in self.sections.iter().enumerate() {
                pos += 1;

                if self.section_expanded[si] {
                    for &wi in &section.widget_indices {
                        if wi == target_wi {
                            return pos;
                        }

                        pos += 1;

                        if self.widget_expanded[wi] {
                            pos += self.widget_items[wi].len();
                        }
                    }
                }
            }

            for &wi in &self.orphan_widgets {
                if wi == target_wi {
                    return pos;
                }

                pos += 1;

                if self.widget_expanded[wi] {
                    pos += self.widget_items[wi].len();
                }
            }
        } else {
            for wi in 0..self.widgets.len() {
                if wi == target_wi {
                    return pos;
                }

                pos += 1;

                if self.widget_expanded[wi] {
                    pos += self.widget_items[wi].len();
                }
            }
        }

        pos
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{buffer::Buffer, layout::Rect};

    struct Stub {
        tab: &'static str,
        section: Option<&'static str>,
        group: &'static str,
        name: &'static str,
    }

    impl Stub {
        fn new(group: &'static str, name: &'static str) -> Self {
            Self {
                tab: "Widgets",
                section: None,
                group,
                name,
            }
        }

        fn tab(mut self, tab: &'static str) -> Self {
            self.tab = tab;
            self
        }

        fn section(mut self, section: &'static str) -> Self {
            self.section = Some(section);
            self
        }
    }

    impl Ingredient for Stub {
        fn tab(&self) -> &str {
            self.tab
        }
        fn section(&self) -> Option<&str> {
            self.section
        }
        fn group(&self) -> &str {
            self.group
        }
        fn name(&self) -> &str {
            self.name
        }
        fn source(&self) -> &str {
            ""
        }
        fn render(&self, _area: Rect, _buf: &mut Buffer) {}
    }

    fn boxed(s: Stub) -> Box<dyn Ingredient> {
        Box::new(s)
    }

    fn sample_ingredients() -> Vec<Box<dyn Ingredient>> {
        vec![
            boxed(Stub::new("Table", "Default")),
            boxed(Stub::new("Table", "Striped")),
            boxed(Stub::new("Chart", "Line")),
            boxed(Stub::new("Chart", "Bar")),
            boxed(Stub::new("Chart", "Scatter")),
        ]
    }

    // -- build ----------------------------------------------------------------

    #[test]
    fn build_groups_by_widget_name() {
        let items = sample_ingredients();
        let nav = NavTree::build(&items, "Widgets");

        assert_eq!(nav.widgets, vec!["Table", "Chart"]);
        assert_eq!(nav.widget_items[0], vec![0, 1]);
        assert_eq!(nav.widget_items[1], vec![2, 3, 4]);
    }

    #[test]
    fn build_filters_by_tab() {
        let items = vec![
            boxed(Stub::new("Table", "Default")),
            boxed(Stub::new("Chart", "Line").tab("Styles")),
        ];
        let nav = NavTree::build(&items, "Widgets");

        assert_eq!(nav.widgets, vec!["Table"]);
        assert_eq!(nav.widget_items[0], vec![0]);
    }

    #[test]
    fn build_empty_tab() {
        let items = sample_ingredients();
        let nav = NavTree::build(&items, "Views");

        assert!(nav.is_empty());
        assert_eq!(nav.cursor, 0);
    }

    #[test]
    fn build_cursor_starts_on_first_variant() {
        let items = sample_ingredients();
        let nav = NavTree::build(&items, "Widgets");

        assert_eq!(nav.cursor, 1);
        assert_eq!(nav.selected_ingredient(), Some(0));
    }

    // -- visible --------------------------------------------------------------

    #[test]
    fn visible_all_expanded() {
        let items = sample_ingredients();
        let nav = NavTree::build(&items, "Widgets");
        let vis = nav.visible();

        // Table(widget) + 2 variants + Chart(widget) + 3 variants = 7
        assert_eq!(vis.len(), 7);
        assert!(
            matches!(vis[0], NavEntry::Widget { ref name, expanded: true, .. } if name == "Table")
        );
        assert!(matches!(
            vis[1],
            NavEntry::Variant {
                ingredient_idx: 0,
                ..
            }
        ));
        assert!(matches!(
            vis[2],
            NavEntry::Variant {
                ingredient_idx: 1,
                ..
            }
        ));
        assert!(
            matches!(vis[3], NavEntry::Widget { ref name, expanded: true, .. } if name == "Chart")
        );
    }

    #[test]
    fn visible_collapsed_hides_children() {
        let items = sample_ingredients();
        let mut nav = NavTree::build(&items, "Widgets");
        nav.widget_expanded[0] = false;

        let vis = nav.visible();
        // Table(collapsed) + Chart(widget) + 3 variants = 5
        assert_eq!(vis.len(), 5);
        assert!(matches!(
            vis[0],
            NavEntry::Widget {
                expanded: false,
                ..
            }
        ));
        assert!(matches!(vis[1], NavEntry::Widget { ref name, .. } if name == "Chart"));
    }

    // -- navigation -----------------------------------------------------------

    #[test]
    fn move_to_clamps() {
        let items = sample_ingredients();
        let mut nav = NavTree::build(&items, "Widgets");
        nav.move_to(999);
        assert_eq!(nav.cursor, nav.visible().len() - 1);
    }

    #[test]
    fn selected_ingredient_on_widget_returns_none() {
        let items = sample_ingredients();
        let mut nav = NavTree::build(&items, "Widgets");
        nav.cursor = 0;
        assert_eq!(nav.selected_ingredient(), None);
    }

    #[test]
    fn selected_widget_items_on_widget_header() {
        let items = sample_ingredients();
        let mut nav = NavTree::build(&items, "Widgets");
        nav.cursor = 0; // Table widget header

        let (name, indices) = nav.selected_widget_items().unwrap();
        assert_eq!(name, "Table");
        assert_eq!(indices, &[0, 1]);
    }

    #[test]
    fn selected_widget_items_on_variant_returns_none() {
        let items = sample_ingredients();
        let nav = NavTree::build(&items, "Widgets");
        // cursor starts on first variant
        assert!(nav.selected_widget_items().is_none());
    }

    #[test]
    fn selected_ingredient_on_variant_returns_index() {
        let items = sample_ingredients();
        let mut nav = NavTree::build(&items, "Widgets");
        nav.cursor = 4; // Chart > "Line" (ingredient_idx=2)
        assert_eq!(nav.selected_ingredient(), Some(2));
    }

    // -- expand / collapse ----------------------------------------------------

    #[test]
    fn expand_on_collapsed_widget() {
        let items = sample_ingredients();
        let mut nav = NavTree::build(&items, "Widgets");
        nav.widget_expanded[0] = false;
        nav.cursor = 0;
        nav.expand();
        assert!(nav.widget_expanded[0]);
    }

    #[test]
    fn collapse_from_variant_jumps_to_parent() {
        let items = sample_ingredients();
        let mut nav = NavTree::build(&items, "Widgets");
        nav.cursor = 1;
        nav.collapse();

        assert!(!nav.widget_expanded[0]);
        assert_eq!(nav.cursor, 0);
    }

    #[test]
    fn collapse_from_second_widget_variant() {
        let items = sample_ingredients();
        let mut nav = NavTree::build(&items, "Widgets");
        nav.cursor = 5; // Chart > "Bar" (second variant)
        nav.collapse();

        assert!(!nav.widget_expanded[1]);
        // Table(expanded) + 2 variants = 3; Chart at position 3
        assert_eq!(nav.cursor, 3);
    }

    #[test]
    fn toggle_or_enter_on_widget_toggles() {
        let items = sample_ingredients();
        let mut nav = NavTree::build(&items, "Widgets");
        nav.cursor = 0;
        assert!(nav.widget_expanded[0]);
        nav.toggle_or_enter();
        assert!(!nav.widget_expanded[0]);
        nav.toggle_or_enter();
        assert!(nav.widget_expanded[0]);
    }

    #[test]
    fn toggle_or_enter_on_variant_is_noop() {
        let items = sample_ingredients();
        let mut nav = NavTree::build(&items, "Widgets");
        nav.cursor = 1;
        let expanded_before = nav.widget_expanded.clone();
        nav.toggle_or_enter();
        assert_eq!(nav.widget_expanded, expanded_before);
    }

    // -- scroll ---------------------------------------------------------------

    #[test]
    fn scroll_into_view_scrolls_down() {
        let items = sample_ingredients();
        let mut nav = NavTree::build(&items, "Widgets");
        nav.cursor = 6;
        nav.scroll_offset = 0;
        nav.scroll_into_view(3);
        assert_eq!(nav.scroll_offset, 4);
    }

    #[test]
    fn scroll_into_view_scrolls_up() {
        let items = sample_ingredients();
        let mut nav = NavTree::build(&items, "Widgets");
        nav.cursor = 1;
        nav.scroll_offset = 3;
        nav.scroll_into_view(3);
        assert_eq!(nav.scroll_offset, 1);
    }

    #[test]
    fn scroll_into_view_clamps_offset_past_end() {
        let items = sample_ingredients();
        let mut nav = NavTree::build(&items, "Widgets");
        nav.scroll_offset = 100;
        nav.scroll_into_view(3);
        assert!(nav.scroll_offset <= 4);
    }

    #[test]
    fn scroll_into_view_zero_viewport_is_noop() {
        let items = sample_ingredients();
        let mut nav = NavTree::build(&items, "Widgets");
        let before = nav.scroll_offset;
        nav.scroll_into_view(0);
        assert_eq!(nav.scroll_offset, before);
    }

    // -- sections -------------------------------------------------------------

    fn sectioned_ingredients() -> Vec<Box<dyn Ingredient>> {
        vec![
            boxed(Stub::new("Block", "Default").section("Core")),
            boxed(Stub::new("Paragraph", "Default").section("Core")),
            boxed(Stub::new("Pie Chart", "Default").section("Charts")),
            boxed(Stub::new("Pie Chart", "Exploded").section("Charts")),
            boxed(Stub::new("Banner", "Default")),
        ]
    }

    #[test]
    fn sections_build_assigns_widgets() {
        let items = sectioned_ingredients();
        let nav = NavTree::build(&items, "Widgets");

        assert_eq!(nav.sections.len(), 2);
        assert_eq!(nav.sections[0].name, "Core");
        assert_eq!(nav.sections[0].widget_indices, vec![0, 1]);
        assert_eq!(nav.sections[1].name, "Charts");
        assert_eq!(nav.sections[1].widget_indices, vec![2]);
        assert_eq!(nav.orphan_widgets, vec![3]);
    }

    #[test]
    fn sections_visible_three_levels() {
        let items = sectioned_ingredients();
        let nav = NavTree::build(&items, "Widgets");
        let vis = nav.visible();

        // Core(section) + Block(widget) + Default(variant)
        //   + Paragraph(widget) + Default(variant)
        // Charts(section) + Pie Chart(widget) + Default + Exploded
        // Banner(orphan widget) + Default
        assert_eq!(vis.len(), 11);
        assert!(matches!(vis[0], NavEntry::Section { ref name, .. } if name == "Core"));
        assert!(matches!(vis[1], NavEntry::Widget { ref name, .. } if name == "Block"));
        assert!(matches!(
            vis[2],
            NavEntry::Variant {
                ingredient_idx: 0,
                ..
            }
        ));
        assert!(matches!(vis[3], NavEntry::Widget { ref name, .. } if name == "Paragraph"));
        assert!(matches!(
            vis[4],
            NavEntry::Variant {
                ingredient_idx: 1,
                ..
            }
        ));
        assert!(matches!(vis[5], NavEntry::Section { ref name, .. } if name == "Charts"));
        assert!(matches!(vis[6], NavEntry::Widget { ref name, .. } if name == "Pie Chart"));
        assert!(matches!(
            vis[7],
            NavEntry::Variant {
                ingredient_idx: 2,
                ..
            }
        ));
        assert!(matches!(
            vis[8],
            NavEntry::Variant {
                ingredient_idx: 3,
                ..
            }
        ));
        assert!(matches!(vis[9], NavEntry::Widget { ref name, .. } if name == "Banner"));
        assert!(matches!(
            vis[10],
            NavEntry::Variant {
                ingredient_idx: 4,
                ..
            }
        ));
    }

    #[test]
    fn sections_cursor_starts_on_first_variant() {
        let items = sectioned_ingredients();
        let nav = NavTree::build(&items, "Widgets");

        assert_eq!(nav.cursor, 2);
        assert_eq!(nav.selected_ingredient(), Some(0));
    }

    #[test]
    fn sections_collapse_section_hides_children() {
        let items = sectioned_ingredients();
        let mut nav = NavTree::build(&items, "Widgets");
        nav.section_expanded[0] = false;
        let vis = nav.visible();

        // Core(collapsed) + Charts(section) + Pie Chart + 2 variants + Banner + Default = 7
        assert_eq!(vis.len(), 7);
        assert!(matches!(
            vis[0],
            NavEntry::Section {
                expanded: false,
                ..
            }
        ));
        assert!(matches!(vis[1], NavEntry::Section { ref name, .. } if name == "Charts"));
    }

    #[test]
    fn sections_collapse_from_variant_jumps_to_widget() {
        let items = sectioned_ingredients();
        let mut nav = NavTree::build(&items, "Widgets");
        nav.cursor = 2; // Core > Block > Default
        nav.collapse();

        assert!(!nav.widget_expanded[0]);
        assert_eq!(nav.cursor, 1);
    }

    #[test]
    fn sections_collapse_collapsed_widget_jumps_to_section() {
        let items = sectioned_ingredients();
        let mut nav = NavTree::build(&items, "Widgets");

        nav.widget_expanded[0] = false;
        nav.cursor = 1; // Block widget header (collapsed)
        nav.collapse();

        assert!(!nav.section_expanded[0]);
        assert_eq!(nav.cursor, 0);
    }

    #[test]
    fn sections_expand_collapsed_section() {
        let items = sectioned_ingredients();
        let mut nav = NavTree::build(&items, "Widgets");
        nav.section_expanded[0] = false;
        nav.cursor = 0;
        nav.expand();

        assert!(nav.section_expanded[0]);
    }

    #[test]
    fn sections_toggle_section() {
        let items = sectioned_ingredients();
        let mut nav = NavTree::build(&items, "Widgets");
        nav.cursor = 0;
        assert!(nav.section_expanded[0]);
        nav.toggle_or_enter();
        assert!(!nav.section_expanded[0]);
        nav.toggle_or_enter();
        assert!(nav.section_expanded[0]);
    }

    #[test]
    fn sections_orphan_widgets_appear_below() {
        let items = sectioned_ingredients();
        let nav = NavTree::build(&items, "Widgets");
        let vis = nav.visible();

        let banner_pos = vis
            .iter()
            .position(|e| matches!(e, NavEntry::Widget { name, .. } if name == "Banner"))
            .unwrap();
        let last_section_pos = vis
            .iter()
            .rposition(|e| matches!(e, NavEntry::Section { .. }))
            .unwrap();

        assert!(banner_pos > last_section_pos);
    }

    #[test]
    fn sections_no_sections_when_none_declared() {
        let items = vec![boxed(Stub::new("Banner", "Default"))];
        let nav = NavTree::build(&items, "Widgets");

        assert!(nav.sections.is_empty());
        assert_eq!(nav.orphan_widgets, vec![0]);
    }
}
