use std::io;
use std::time::Duration;

use ratatui::{
    crossterm::event::{
        self, Event, KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEvent,
        MouseEventKind,
    },
    layout::Position,
    DefaultTerminal,
};

use crate::Ingredient;
use crate::color_depth::{ColorDepth, quantize_buffer};
use crate::nav::NavTree;
use crate::theme::PantryTheme;
use crate::ui;

pub(crate) const TAB_LABELS: &[&str] = &["Widgets", "Views", "Styles"];

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Focus {
    Sidebar,
    Preview,
    Fullscreen,
}

pub struct App {
    pub ingredients: Vec<Box<dyn Ingredient>>,
    pub(crate) theme: PantryTheme,
    pub(crate) active_tab: usize,
    pub(crate) navs: Vec<NavTree>,
    pub(crate) focus: Focus,
    pub(crate) color_depth: ColorDepth,
    running: bool,
}

impl App {
    pub fn new(ingredients: Vec<Box<dyn Ingredient>>, theme: PantryTheme) -> Self {
        let navs: Vec<NavTree> = TAB_LABELS
            .iter()
            .map(|tab| NavTree::build(&ingredients, tab))
            .collect();

        Self {
            ingredients,
            theme,
            active_tab: 0,
            navs,
            focus: Focus::Sidebar,
            color_depth: ColorDepth::default(),
            running: true,
        }
    }

    pub fn nav(&self) -> &NavTree {
        &self.navs[self.active_tab]
    }

    pub fn nav_mut(&mut self) -> &mut NavTree {
        &mut self.navs[self.active_tab]
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        while self.running {
            let regions = ui::Regions::from_terminal(terminal.size()?.into());
            self.nav_mut()
                .scroll_into_view(regions.sidebar.height.saturating_sub(1) as usize);

            let depth = self.color_depth;
            terminal.draw(|frame| {
                ui::render(&self, frame.area(), frame.buffer_mut(), &regions);
                if depth != ColorDepth::TrueColor {
                    quantize_buffer(frame.buffer_mut(), depth);
                }
            })?;

            // ~30 fps poll; handles both keyboard and mouse input
            if event::poll(Duration::from_millis(33))? {
                match event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        self.handle_key(key.code, key.modifiers);
                    }
                    Event::Mouse(mouse) => {
                        self.handle_mouse(mouse, &regions);
                    }
                    _ => {}
                }
            }
        }

        Ok(())
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
            KeyCode::Char('c') => self.color_depth = self.color_depth.cycle(),
            KeyCode::Up | KeyCode::Char('k') => self.nav_mut().move_up(),
            KeyCode::Down | KeyCode::Char('j') => self.nav_mut().move_down(),
            KeyCode::Right | KeyCode::Char('l') => self.nav_mut().expand(),
            KeyCode::Left | KeyCode::Char('h') => self.nav_mut().collapse(),
            KeyCode::Enter => self.enter_or_toggle(),
            KeyCode::Char('f') if self.nav().selected_ingredient().is_some() => {
                self.focus = Focus::Fullscreen;
            }

            // Tab switching
            KeyCode::Char('1') => self.active_tab = 0,
            KeyCode::Char('2') => self.active_tab = 1,
            KeyCode::Char('3') => self.active_tab = 2,
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

    /// Dispatch mouse events: sidebar clicks, tab clicks, scroll wheel.
    fn handle_mouse(&mut self, mouse: MouseEvent, regions: &ui::Regions) {
        let pos = Position::new(mouse.column, mouse.row);
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if regions.sidebar.contains(pos) {
                    self.click_sidebar(mouse.row, regions);
                } else if let Some(tab) = regions.tab_at(mouse.column, mouse.row) {
                    self.active_tab = tab;
                }
            }
            MouseEventKind::ScrollUp if regions.sidebar.contains(pos) => {
                self.nav_mut().move_up();
            }
            MouseEventKind::ScrollDown if regions.sidebar.contains(pos) => {
                self.nav_mut().move_down();
            }
            _ => {}
        }
    }

    /// Map a sidebar click row to a nav entry. Always returns focus to sidebar.
    fn click_sidebar(&mut self, row: u16, regions: &ui::Regions) {
        self.focus = Focus::Sidebar;
        let entry_row = row.saturating_sub(regions.sidebar.y + 1) as usize;
        let visible_index = entry_row + self.nav().scroll_offset;
        self.nav_mut().move_to(visible_index);
    }

    /// Enter focuses interactive variants; toggles groups otherwise.
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
