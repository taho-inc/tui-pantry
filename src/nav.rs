use crate::Ingredient;

/// Entry in the flattened navigation list.
#[derive(Debug)]
pub(crate) enum NavEntry {
    Group {
        name: String,
        expanded: bool,
    },
    Variant {
        group_idx: usize,
        ingredient_idx: usize,
    },
}

/// Flattened navigation tree built from grouped ingredients.
pub(crate) struct NavTree {
    /// Group names in display order.
    pub groups: Vec<String>,
    /// Ingredient indices grouped by group name.
    pub group_items: Vec<Vec<usize>>,
    /// Expansion state per group.
    pub expanded: Vec<bool>,
    /// Cursor position in the visible list.
    pub cursor: usize,
    /// First visible entry in the sidebar viewport.
    pub scroll_offset: usize,
}

impl NavTree {
    /// Build the nav tree from ingredients matching a specific tab.
    ///
    /// Ingredient indices reference the original global vec, so preview
    /// lookups stay correct across tabs.
    pub fn build(ingredients: &[Box<dyn Ingredient>], tab: &str) -> Self {
        let mut groups: Vec<String> = Vec::new();
        let mut group_items: Vec<Vec<usize>> = Vec::new();

        for (i, ingredient) in ingredients.iter().enumerate() {
            if ingredient.tab() != tab {
                continue;
            }

            let group = ingredient.group().to_string();
            if let Some(pos) = groups.iter().position(|g| *g == group) {
                group_items[pos].push(i);
            } else {
                groups.push(group);
                group_items.push(vec![i]);
            }
        }

        let expanded = vec![true; groups.len()];

        let cursor = if groups.is_empty() {
            0
        } else {
            1.min(Self::total_visible(&expanded, &group_items).saturating_sub(1))
        };

        Self {
            groups,
            group_items,
            expanded,
            cursor,
            scroll_offset: 0,
        }
    }

    fn total_visible(expanded: &[bool], group_items: &[Vec<usize>]) -> usize {
        expanded.iter().enumerate().fold(0, |acc, (gi, &exp)| {
            acc + 1 + if exp { group_items[gi].len() } else { 0 }
        })
    }

    /// Flatten visible entries based on expansion state.
    pub fn visible(&self) -> Vec<NavEntry> {
        let mut entries = Vec::new();
        for (gi, group) in self.groups.iter().enumerate() {
            entries.push(NavEntry::Group {
                name: group.clone(),
                expanded: self.expanded[gi],
            });
            if self.expanded[gi] {
                for &ii in &self.group_items[gi] {
                    entries.push(NavEntry::Variant {
                        group_idx: gi,
                        ingredient_idx: ii,
                    });
                }
            }
        }
        entries
    }

    /// The currently selected ingredient index, if cursor is on a variant.
    pub fn selected_ingredient(&self) -> Option<usize> {
        let entries = self.visible();
        match entries.get(self.cursor) {
            Some(NavEntry::Variant { ingredient_idx, .. }) => Some(*ingredient_idx),
            _ => None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.groups.is_empty()
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

    /// Toggle expand/collapse if cursor is on a group. Expand + enter first child if collapsing.
    pub fn toggle_or_enter(&mut self) {
        let entries = self.visible();
        if let Some(NavEntry::Group { .. }) = entries.get(self.cursor) {
            self.toggle_at_cursor();
        }
    }

    /// Expand the group at cursor (right arrow).
    pub fn expand(&mut self) {
        if let Some(gi) = self.group_at_cursor()
            && !self.expanded[gi]
        {
            self.expanded[gi] = true;
        }
    }

    /// Collapse the group at cursor, or the parent group if on a variant (left arrow).
    pub fn collapse(&mut self) {
        let entries = self.visible();
        match entries.get(self.cursor) {
            Some(NavEntry::Group { .. }) => {
                if let Some(gi) = self.group_at_cursor() {
                    self.expanded[gi] = false;
                }
            }
            Some(NavEntry::Variant { group_idx, .. }) => {
                let gi = *group_idx;
                self.expanded[gi] = false;
                // Move cursor to the group header
                let mut pos = 0;
                for g in 0..gi {
                    pos += 1;
                    if self.expanded[g] {
                        pos += self.group_items[g].len();
                    }
                }
                self.cursor = pos;
            }
            None => {}
        }
    }

    /// Adjust scroll so the cursor stays within the viewport.
    pub fn scroll_into_view(&mut self, viewport_height: usize) {
        if viewport_height == 0 {
            return;
        }
        // Clamp offset so we don't show blank space past the end.
        let total = self.visible().len();
        let max_offset = total.saturating_sub(viewport_height);
        self.scroll_offset = self.scroll_offset.min(max_offset);

        if self.cursor < self.scroll_offset {
            self.scroll_offset = self.cursor;
        } else if self.cursor >= self.scroll_offset + viewport_height {
            self.scroll_offset = self.cursor - viewport_height + 1;
        }
    }

    fn toggle_at_cursor(&mut self) {
        if let Some(gi) = self.group_at_cursor() {
            self.expanded[gi] = !self.expanded[gi];
        }
    }

    fn group_at_cursor(&self) -> Option<usize> {
        let entries = self.visible();
        match entries.get(self.cursor) {
            Some(NavEntry::Group { name, .. }) => self.groups.iter().position(|g| g == name),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{buffer::Buffer, layout::Rect};

    struct Stub {
        tab: &'static str,
        group: &'static str,
        name: &'static str,
    }

    impl Stub {
        fn new(group: &'static str, name: &'static str) -> Self {
            Self {
                tab: "Widgets",
                group,
                name,
            }
        }

        fn tab(mut self, tab: &'static str) -> Self {
            self.tab = tab;
            self
        }
    }

    impl Ingredient for Stub {
        fn tab(&self) -> &str {
            self.tab
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
    fn build_groups_by_group_name() {
        let items = sample_ingredients();
        let nav = NavTree::build(&items, "Widgets");

        assert_eq!(nav.groups, vec!["Table", "Chart"]);
        assert_eq!(nav.group_items[0], vec![0, 1]);
        assert_eq!(nav.group_items[1], vec![2, 3, 4]);
    }

    #[test]
    fn build_filters_by_tab() {
        let items = vec![
            boxed(Stub::new("Table", "Default")),
            boxed(Stub::new("Chart", "Line").tab("Styles")),
        ];
        let nav = NavTree::build(&items, "Widgets");

        assert_eq!(nav.groups, vec!["Table"]);
        assert_eq!(nav.group_items[0], vec![0]);
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

        // Cursor at 1 = first variant under first group
        assert_eq!(nav.cursor, 1);
        assert_eq!(nav.selected_ingredient(), Some(0));
    }

    // -- visible --------------------------------------------------------------

    #[test]
    fn visible_all_expanded() {
        let items = sample_ingredients();
        let nav = NavTree::build(&items, "Widgets");
        let vis = nav.visible();

        // Table(group) + 2 variants + Chart(group) + 3 variants = 7
        assert_eq!(vis.len(), 7);
        assert!(matches!(vis[0], NavEntry::Group { ref name, expanded: true } if name == "Table"));
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
        assert!(matches!(vis[3], NavEntry::Group { ref name, expanded: true } if name == "Chart"));
    }

    #[test]
    fn visible_collapsed_hides_children() {
        let items = sample_ingredients();
        let mut nav = NavTree::build(&items, "Widgets");
        nav.expanded[0] = false; // collapse Table

        let vis = nav.visible();
        // Table(group, collapsed) + Chart(group) + 3 variants = 5
        assert_eq!(vis.len(), 5);
        assert!(matches!(
            vis[0],
            NavEntry::Group {
                expanded: false,
                ..
            }
        ));
        assert!(matches!(vis[1], NavEntry::Group { ref name, .. } if name == "Chart"));
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
    fn selected_ingredient_on_group_returns_none() {
        let items = sample_ingredients();
        let mut nav = NavTree::build(&items, "Widgets");
        nav.cursor = 0; // group header
        assert_eq!(nav.selected_ingredient(), None);
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
    fn expand_on_collapsed_group() {
        let items = sample_ingredients();
        let mut nav = NavTree::build(&items, "Widgets");
        nav.expanded[0] = false;
        nav.cursor = 0;
        nav.expand();
        assert!(nav.expanded[0]);
    }

    #[test]
    fn collapse_from_variant_jumps_to_parent() {
        let items = sample_ingredients();
        let mut nav = NavTree::build(&items, "Widgets");
        nav.cursor = 1; // first variant of Table
        nav.collapse();

        assert!(!nav.expanded[0]); // Table collapsed
        assert_eq!(nav.cursor, 0); // cursor on Table group
    }

    #[test]
    fn collapse_from_second_group_variant() {
        let items = sample_ingredients();
        let mut nav = NavTree::build(&items, "Widgets");
        nav.cursor = 5; // Chart > "Bar" (second variant)
        nav.collapse();

        assert!(!nav.expanded[1]); // Chart collapsed
        // Table(expanded) + 2 variants = 3; Chart at position 3
        assert_eq!(nav.cursor, 3);
    }

    #[test]
    fn toggle_or_enter_on_group_toggles() {
        let items = sample_ingredients();
        let mut nav = NavTree::build(&items, "Widgets");
        nav.cursor = 0;
        assert!(nav.expanded[0]);
        nav.toggle_or_enter();
        assert!(!nav.expanded[0]);
        nav.toggle_or_enter();
        assert!(nav.expanded[0]);
    }

    #[test]
    fn toggle_or_enter_on_variant_is_noop() {
        let items = sample_ingredients();
        let mut nav = NavTree::build(&items, "Widgets");
        nav.cursor = 1; // variant
        let expanded_before = nav.expanded.clone();
        nav.toggle_or_enter();
        assert_eq!(nav.expanded, expanded_before);
    }

    // -- scroll ---------------------------------------------------------------

    #[test]
    fn scroll_into_view_scrolls_down() {
        let items = sample_ingredients();
        let mut nav = NavTree::build(&items, "Widgets");
        nav.cursor = 6; // last entry
        nav.scroll_offset = 0;
        nav.scroll_into_view(3);
        // cursor(6) should be visible: offset = 6 - 3 + 1 = 4
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
        // max_offset = 7 - 3 = 4
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
}
