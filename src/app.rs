use std::io;
use std::time::Duration;

use ratatui::{
    DefaultTerminal,
    crossterm::event::{
        self, Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    },
    layout::{Position, Rect},
    style::Color,
};

use crate::Ingredient;
use crate::color_depth::{ColorDepth, quantize_buffer};
use crate::nav::NavTree;
use crate::theme::{PantryTheme, PreviewBackgrounds, ThemePair};
use crate::ui;

pub(crate) const TAB_LABELS: &[&str] = &["Widgets", "Panes", "Views", "Styles"];

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Focus {
    Sidebar,
    Preview,
    Fullscreen,
}

/// Active scrollbar thumb drag.
struct ScrollbarDrag {
    target: DragTarget,
    area: Rect,
    content_max: usize,
}

#[derive(Clone, Copy)]
enum DragTarget {
    Sidebar,
    Gallery,
}

pub struct App {
    pub ingredients: Vec<Box<dyn Ingredient>>,
    pub(crate) themes: ThemePair,
    pub(crate) dark_mode: bool,
    pub(crate) preview_backgrounds: PreviewBackgrounds,
    pub(crate) preview_bg_index: Option<usize>,
    pub(crate) active_tab: usize,
    pub(crate) navs: Vec<NavTree>,
    pub(crate) focus: Focus,
    pub(crate) color_depth: ColorDepth,
    /// Gallery view scroll offset.
    pub(crate) gallery_scroll: usize,
    scrollbar_drag: Option<ScrollbarDrag>,
    running: bool,
}

impl App {
    pub fn new(
        ingredients: Vec<Box<dyn Ingredient>>,
        themes: ThemePair,
        preview_backgrounds: PreviewBackgrounds,
    ) -> Self {
        let navs: Vec<NavTree> = TAB_LABELS
            .iter()
            .map(|tab| NavTree::build(&ingredients, tab))
            .collect();

        let dark_mode = themes.start_dark();

        let preview_bg_index = if preview_backgrounds.is_empty() {
            None
        } else {
            Some(0)
        };

        Self {
            ingredients,
            themes,
            dark_mode,
            preview_backgrounds,
            preview_bg_index,
            active_tab: 0,
            navs,
            focus: Focus::Sidebar,
            color_depth: ColorDepth::default(),
            gallery_scroll: 0,
            scrollbar_drag: None,
            running: true,
        }
    }

    pub(crate) fn theme(&self) -> &PantryTheme {
        self.themes.get(self.dark_mode)
    }

    /// Current preview background, if configured.
    pub(crate) fn preview_bg(&self) -> Option<(&str, Color)> {
        self.preview_bg_index
            .and_then(|i| self.preview_backgrounds.get(i))
    }

    fn cycle_preview_bg(&mut self) {
        if let Some(idx) = &mut self.preview_bg_index {
            *idx = (*idx + 1) % self.preview_backgrounds.len();
        }
    }

    pub fn nav(&self) -> &NavTree {
        &self.navs[self.active_tab]
    }

    pub fn nav_mut(&mut self) -> &mut NavTree {
        &mut self.navs[self.active_tab]
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        let mut dirty = true;
        let mut regions = ui::Regions::from_terminal(terminal.size()?.into());

        while self.running {
            if dirty {
                regions = ui::Regions::from_terminal(terminal.size()?.into());
                self.nav_mut()
                    .scroll_into_view(regions.sidebar.height.saturating_sub(1) as usize);

                let depth = self.color_depth;
                terminal.draw(|frame| {
                    let draw_regions = ui::Regions::from_terminal(frame.area());
                    ui::render(&self, frame.area(), frame.buffer_mut(), &draw_regions);
                    if depth != ColorDepth::TrueColor {
                        quantize_buffer(frame.buffer_mut(), depth);
                    }
                })?;
                dirty = false;
            }

            let timeout = if self.selected_is_animated() {
                Duration::from_millis(33)
            } else {
                Duration::from_secs(1)
            };
            let has_event = event::poll(timeout)?;

            if has_event {
                match event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        self.handle_key(key.code, key.modifiers);
                        dirty = true;
                    }
                    Event::Mouse(mouse) => {
                        self.handle_mouse(mouse, &regions);
                        dirty = true;
                    }
                    Event::Resize(..) => dirty = true,
                    _ => {}
                }
            } else {
                dirty = true;
            }
        }

        Ok(())
    }

    fn selected_is_animated(&self) -> bool {
        self.nav()
            .selected_ingredient()
            .is_some_and(|idx| self.ingredients[idx].animated())
    }

    fn handle_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        match self.focus {
            Focus::Sidebar => self.handle_sidebar_key(code, modifiers),
            Focus::Preview => self.handle_preview_key(code),
            Focus::Fullscreen => self.handle_fullscreen_key(code),
        }
    }

    fn handle_sidebar_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => self.running = false,
            KeyCode::Char('b') => self.cycle_preview_bg(),
            KeyCode::Char('c') => self.color_depth = self.color_depth.cycle(),
            KeyCode::Char('t') => self.dark_mode = !self.dark_mode,
            KeyCode::Up | KeyCode::Char('k') => {
                self.nav_mut().move_up();
                self.gallery_scroll = 0;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.nav_mut().move_down();
                self.gallery_scroll = 0;
            }
            KeyCode::Right | KeyCode::Char('l') => self.nav_mut().expand(),
            KeyCode::Left | KeyCode::Char('h') => self.nav_mut().collapse(),
            KeyCode::Enter => self.enter_or_toggle(),
            KeyCode::Char('f') if self.nav().selected_ingredient().is_some() => {
                self.focus = Focus::Fullscreen;
            }

            KeyCode::Char('1') => self.active_tab = 0,
            KeyCode::Char('2') => self.active_tab = 1,
            KeyCode::Char('3') => self.active_tab = 2,
            KeyCode::Char('4') => self.active_tab = 3,
            KeyCode::Tab if modifiers.contains(KeyModifiers::SHIFT) => {
                self.active_tab = (self.active_tab + TAB_LABELS.len() - 1) % TAB_LABELS.len();
            }
            KeyCode::Tab => {
                self.active_tab = (self.active_tab + 1) % TAB_LABELS.len();
            }
            KeyCode::BackTab => {
                self.active_tab = (self.active_tab + TAB_LABELS.len() - 1) % TAB_LABELS.len();
            }

            _ => {}
        }
    }

    fn handle_preview_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Esc => {
                self.focus = Focus::Sidebar;
                return;
            }
            KeyCode::Char('f') => {
                self.focus = Focus::Fullscreen;
                return;
            }
            _ => {}
        }

        if let Some(idx) = self.nav().selected_ingredient() {
            self.ingredients[idx].handle_key(code);
        }
    }

    fn handle_fullscreen_key(&mut self, code: KeyCode) {
        if matches!(code, KeyCode::Esc | KeyCode::Char('f')) {
            self.focus = Focus::Sidebar;
            return;
        }

        if let Some(idx) = self.nav().selected_ingredient() {
            self.ingredients[idx].handle_key(code);
        }
    }

    fn handle_mouse(&mut self, mouse: MouseEvent, regions: &ui::Regions) {
        let pos = Position::new(mouse.column, mouse.row);

        // Scrollbar drag continuation — must come before other handlers.
        if let MouseEventKind::Drag(MouseButton::Left) = mouse.kind
            && let Some(drag) = &self.scrollbar_drag
        {
            let value = ui::scrollbar_position_from_row(drag.area, drag.content_max, mouse.row);

            match drag.target {
                DragTarget::Sidebar => {
                    self.nav_mut().move_to(value);
                }
                DragTarget::Gallery => {
                    self.gallery_scroll = value;
                }
            }

            return;
        }

        if let MouseEventKind::Up(MouseButton::Left) = mouse.kind {
            self.scrollbar_drag = None;
        }

        match self.focus {
            Focus::Preview | Focus::Fullscreen => {
                let ingredient_area = if self.focus == Focus::Fullscreen {
                    regions.fullscreen_area()
                } else {
                    ui::ingredient_area(regions, &self.ingredients, self.nav())
                };

                if let Some(idx) = self.nav().selected_ingredient()
                    && self.ingredients[idx].interactive()
                    && ingredient_area.contains(pos)
                {
                    self.ingredients[idx].handle_mouse(mouse, ingredient_area);
                    return;
                }
            }
            Focus::Sidebar => {}
        }

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // Check scrollbar clicks before general area clicks.
                if self.handle_scrollbar_click(mouse, regions) {
                    return;
                }

                if regions.sidebar.contains(pos) {
                    self.click_sidebar(mouse.row, regions);
                } else if let Some(tab) = regions.tab_at(mouse.column, mouse.row) {
                    self.active_tab = tab;
                }
            }

            MouseEventKind::ScrollUp if regions.sidebar.contains(pos) => {
                self.nav_mut().move_up();
                self.gallery_scroll = 0;
            }
            MouseEventKind::ScrollDown if regions.sidebar.contains(pos) => {
                self.nav_mut().move_down();
                self.gallery_scroll = 0;
            }
            MouseEventKind::ScrollUp if regions.preview.contains(pos) => {
                self.gallery_scroll = self.gallery_scroll.saturating_sub(3);
            }
            MouseEventKind::ScrollDown if regions.preview.contains(pos) => {
                self.gallery_scroll += 3;
            }

            _ => {}
        }
    }

    /// Handle a left-click on a scrollbar. Returns true if consumed.
    fn handle_scrollbar_click(&mut self, mouse: MouseEvent, regions: &ui::Regions) -> bool {
        let col = mouse.column;
        let row = mouse.row;

        // Sidebar scrollbar.
        if let Some(area) = ui::sidebar_scrollbar_area(regions, self.nav())
            && col == area.right() - 1
            && row >= area.y
            && row < area.bottom()
        {
            let entries = self.nav().visible().len();
            let viewport = regions.sidebar.height.saturating_sub(1) as usize;
            let page = viewport.max(1);

            return self.dispatch_scrollbar_hit(
                area,
                entries.saturating_sub(1),
                self.nav().cursor,
                row,
                page,
                DragTarget::Sidebar,
            );
        }

        // Gallery scrollbar.
        if let Some((area, max_scroll)) = ui::gallery_scrollbar_info(regions, self.nav())
            && col == area.right() - 1
            && row >= area.y
            && row < area.bottom()
        {
            let page = area.height.saturating_sub(2) as usize;

            return self.dispatch_scrollbar_hit(
                area,
                max_scroll,
                self.gallery_scroll,
                row,
                page,
                DragTarget::Gallery,
            );
        }

        false
    }

    /// Process a scrollbar hit (arrow, track, or thumb) and update state.
    fn dispatch_scrollbar_hit(
        &mut self,
        area: Rect,
        content_max: usize,
        position: usize,
        row: u16,
        page: usize,
        target: DragTarget,
    ) -> bool {
        let Some(hit) = ui::scrollbar_hit_test(area, content_max + 1, position, row) else {
            return false;
        };

        let new_pos = match hit {
            ui::ScrollbarHit::UpArrow => position.saturating_sub(1),
            ui::ScrollbarHit::DownArrow => (position + 1).min(content_max),
            ui::ScrollbarHit::Above => position.saturating_sub(page),
            ui::ScrollbarHit::Below => (position + page).min(content_max),
            ui::ScrollbarHit::Thumb => {
                self.scrollbar_drag = Some(ScrollbarDrag {
                    target,
                    area,
                    content_max,
                });
                return true;
            }
        };

        match target {
            DragTarget::Sidebar => {
                self.nav_mut().move_to(new_pos);
            }
            DragTarget::Gallery => {
                self.gallery_scroll = new_pos;
            }
        }

        true
    }

    fn click_sidebar(&mut self, row: u16, regions: &ui::Regions) {
        self.focus = Focus::Sidebar;
        self.gallery_scroll = 0;
        let entry_row = row.saturating_sub(regions.sidebar.y + 1) as usize;
        let visible_index = entry_row + self.nav().scroll_offset;
        self.nav_mut().move_to(visible_index);
        self.nav_mut().toggle_or_enter();
    }

    fn enter_or_toggle(&mut self) {
        if let Some(idx) = self.nav().selected_ingredient()
            && self.ingredients[idx].interactive()
        {
            self.focus = Focus::Preview;
            return;
        }
        self.nav_mut().toggle_or_enter();
    }
}
