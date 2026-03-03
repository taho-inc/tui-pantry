use crate::Ingredient;

/// Entry in the flattened navigation list.
#[derive(Debug)]
pub(crate) enum NavEntry {
    Group { name: String, expanded: bool },
    Variant { group_idx: usize, ingredient_idx: usize },
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

        let cursor = if groups.is_empty() { 0 } else { 1.min(Self::total_visible(&expanded, &group_items).saturating_sub(1)) };

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
            && !self.expanded[gi] {
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
