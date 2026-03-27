use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
        StatefulWidget, Widget,
    },
};

use crate::Pane;
use crate::ingredient::PropInfo;

use crate::app::{App, Focus, TAB_LABELS};
use crate::nav::NavEntry;
use crate::theme::PantryTheme;

const SIDEBAR_WIDTH: u16 = 28;
const BOTTOM_BAR_HEIGHT: u16 = 2;
const TOP_BAR_HEIGHT: u16 = 3;

const TAB_INDICATOR: &str = "▸";

/// `│  ▸ Label  │` → border(1) + pad(3) + label + pad(3) + border(1)
fn tab_box_width(label: &str) -> u16 {
    label.len() as u16 + 8
}

/// Hit-testable layout regions, computed once from terminal size.
pub(crate) struct Regions {
    pub terminal: Rect,
    pub top_bar: Rect,
    pub sidebar: Rect,
    pub preview: Rect,
    pub bottom_bar: Rect,
}

impl Regions {
    pub fn from_terminal(area: Rect) -> Self {
        let [top_bar, main_area, bottom_bar] = Layout::vertical([
            Constraint::Length(TOP_BAR_HEIGHT),
            Constraint::Min(0),
            Constraint::Length(BOTTOM_BAR_HEIGHT),
        ])
        .areas(area);
        let [sidebar, _gap, preview] = Layout::horizontal([
            Constraint::Length(SIDEBAR_WIDTH),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .areas(main_area);
        Self {
            terminal: area,
            top_bar,
            sidebar,
            preview,
            bottom_bar,
        }
    }

    pub fn fullscreen_area(&self) -> Rect {
        self.terminal
    }

    /// Which tab index (if any) is at the given terminal coordinate.
    pub fn tab_at(&self, col: u16, row: u16) -> Option<usize> {
        let label_row = self.top_bar.y + self.top_bar.height - 2;
        if row != label_row {
            return None;
        }

        let tabs_x = self.top_bar.x + 1;

        let mut x = tabs_x;
        for (i, label) in TAB_LABELS.iter().enumerate() {
            let w = tab_box_width(label);
            if col >= x && col < x + w {
                return Some(i);
            }
            x += w;
        }

        None
    }
}

pub(crate) fn render(app: &App, area: Rect, buf: &mut Buffer, regions: &Regions) {
    let theme = app.theme();

    if app.focus == Focus::Fullscreen {
        if let Some(idx) = app.nav().selected_ingredient() {
            let bg = app.preview_bg().map_or(theme.panel_bg, |(_, c)| c);
            Clear.render(area, buf);
            Block::new().style(Style::new().bg(bg)).render(area, buf);
            app.ingredients[idx].render(area, buf);
        }
        return;
    }

    Block::new()
        .style(Style::new().bg(theme.panel_bg))
        .render(area, buf);

    let focused = app.focus == Focus::Preview;

    render_top_bar(app, theme, regions.top_bar, buf);
    render_sidebar(app, theme, regions.sidebar, buf);
    render_preview(app, theme, regions.preview, focused, buf);
    render_bottom_bar(app, theme, regions.bottom_bar, buf);
}

fn render_top_bar(app: &App, theme: &PantryTheme, area: Rect, buf: &mut Buffer) {
    if area.height < 3 || area.width == 0 {
        return;
    }

    let rounded = symbols::border::ROUNDED;
    let bs = Style::default().fg(theme.border);

    let top_y = area.y;
    let mid_y = area.y + 1;
    let bot_y = area.y + 2;

    for x in area.x..area.right() {
        buf[(x, bot_y)]
            .set_symbol(rounded.horizontal_top)
            .set_style(bs);
    }

    // Title flush-right on the label row.
    let title = format!("TUI PANTRY v{}", env!("CARGO_PKG_VERSION"));
    let title_style = Style::new().fg(theme.accent).add_modifier(Modifier::BOLD);
    let title_x = area.right().saturating_sub(title.len() as u16 + 1);

    for (j, ch) in title.chars().enumerate() {
        let cx = title_x + j as u16;
        if cx < area.right() {
            buf[(cx, mid_y)].set_char(ch).set_style(title_style);
        }
    }

    // Tab boxes flush-left, 1 cell margin.
    let mut x = area.x + 1;

    for (i, label) in TAB_LABELS.iter().enumerate() {
        let w = tab_box_width(label);
        if x + w > area.right() {
            break;
        }

        let active = i == app.active_tab;
        let text_style = if active {
            Style::default().fg(theme.text)
        } else {
            Style::default().fg(theme.text_dim)
        };

        let left_x = x;
        let right_x = x + w - 1;

        // Top border
        buf[(left_x, top_y)]
            .set_symbol(rounded.top_left)
            .set_style(bs);

        for cx in (left_x + 1)..right_x {
            buf[(cx, top_y)]
                .set_symbol(rounded.horizontal_top)
                .set_style(bs);
        }

        buf[(right_x, top_y)]
            .set_symbol(rounded.top_right)
            .set_style(bs);

        // Label row
        buf[(left_x, mid_y)]
            .set_symbol(rounded.vertical_left)
            .set_style(bs);

        buf[(right_x, mid_y)]
            .set_symbol(rounded.vertical_right)
            .set_style(bs);

        let label_x = left_x + 4;
        let label_style = if active {
            let bold = text_style.add_modifier(Modifier::BOLD);
            buf[(label_x - 2, mid_y)]
                .set_symbol(TAB_INDICATOR)
                .set_style(bold);
            bold
        } else {
            text_style
        };

        for (j, ch) in label.chars().enumerate() {
            let cx = label_x + j as u16;
            if cx >= right_x {
                break;
            }
            buf[(cx, mid_y)].set_char(ch).set_style(label_style);
        }

        // Bottom junction
        if active {
            buf[(left_x, bot_y)]
                .set_symbol(rounded.bottom_right)
                .set_style(bs);

            buf[(right_x, bot_y)]
                .set_symbol(rounded.bottom_left)
                .set_style(bs);

            for cx in (left_x + 1)..right_x {
                buf[(cx, bot_y)].set_symbol(" ").set_style(bs);
            }
        } else {
            buf[(left_x, bot_y)].set_symbol("┴").set_style(bs);
            buf[(right_x, bot_y)].set_symbol("┴").set_style(bs);
        }

        x += w;
    }
}

fn render_sidebar(app: &App, theme: &PantryTheme, area: Rect, buf: &mut Buffer) {
    let block = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(theme.text_dim));

    let inner = block.inner(area);
    block.render(area, buf);

    if inner.height < 2 {
        return;
    }

    let nav = app.nav();
    let tab_label = TAB_LABELS[app.active_tab].to_uppercase();

    let header_line = Line::from(vec![Span::styled(
        format!(" {tab_label} "),
        Style::default()
            .fg(theme.text_dim)
            .add_modifier(Modifier::BOLD),
    )]);

    buf.set_line(inner.x, inner.y, &header_line, inner.width);

    if nav.is_empty() {
        if inner.height > 2 {
            let empty_msg = Line::from(Span::styled(
                "  (empty)",
                Style::default().fg(theme.text_dim),
            ));
            buf.set_line(inner.x, inner.y + 2, &empty_msg, inner.width);
        }
        return;
    }

    let entries = nav.visible();
    let selected_ingredient = nav.selected_ingredient();
    let offset = nav.scroll_offset;
    let viewport_rows = inner.height.saturating_sub(1) as usize;

    for (i, entry) in entries.iter().enumerate().skip(offset) {
        let y = inner.y + 1 + (i - offset) as u16;
        if y >= inner.y + inner.height {
            break;
        }

        let is_cursor = i == nav.cursor;

        match entry {
            NavEntry::Section { name, expanded, .. } => {
                let caret = if *expanded { "▼" } else { "▶" };
                let style = if is_cursor {
                    Style::default()
                        .fg(theme.accent)
                        .bg(theme.cursor_bg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.text).add_modifier(Modifier::BOLD)
                };

                let line = Line::from(vec![
                    Span::styled(format!(" {caret} "), Style::default().fg(theme.text_dim)),
                    Span::styled(name.as_str(), style),
                ]);

                buf.set_line(inner.x, y, &line, inner.width);

                if is_cursor {
                    fill_bg(buf, inner.x, y, inner.width, theme.cursor_bg);
                }
            }

            NavEntry::Widget {
                name,
                expanded,
                sectioned,
                ..
            } => {
                let caret = if *expanded { "▼" } else { "▶" };
                let style = if is_cursor {
                    Style::default().fg(theme.accent).bg(theme.cursor_bg)
                } else {
                    Style::default().fg(theme.text)
                };

                let prefix = if *sectioned {
                    format!("   {caret} ")
                } else {
                    format!(" {caret} ")
                };

                let line = Line::from(vec![
                    Span::styled(prefix, Style::default().fg(theme.text_dim)),
                    Span::styled(name.as_str(), style),
                ]);

                buf.set_line(inner.x, y, &line, inner.width);

                if is_cursor {
                    fill_bg(buf, inner.x, y, inner.width, theme.cursor_bg);
                }
            }

            NavEntry::Variant {
                ingredient_idx,
                sectioned,
                ..
            } => {
                let ingredient = &app.ingredients[*ingredient_idx];
                let is_selected = selected_ingredient == Some(*ingredient_idx) && is_cursor;

                let style = if is_selected {
                    Style::default().fg(theme.accent)
                } else if is_cursor {
                    Style::default().fg(theme.text).bg(theme.cursor_bg)
                } else {
                    Style::default().fg(theme.text_dim)
                };

                let marker_style = if is_selected {
                    Style::default().fg(theme.accent)
                } else {
                    Style::default().fg(theme.border)
                };

                let prefix = if *sectioned { "     ◆ " } else { "   ◆ " };

                let line = Line::from(vec![
                    Span::styled(prefix, marker_style),
                    Span::styled(ingredient.name(), style),
                ]);

                buf.set_line(inner.x, y, &line, inner.width);

                if is_cursor {
                    fill_bg(buf, inner.x, y, inner.width, theme.cursor_bg);
                }
            }
        }
    }

    if entries.len() > viewport_rows {
        let scrollbar_area = area.inner(Margin {
            vertical: 1,
            horizontal: 0,
        });

        let mut state = ScrollbarState::new(entries.len()).position(nav.cursor);

        Scrollbar::new(ScrollbarOrientation::VerticalRight).render(scrollbar_area, buf, &mut state);
    }
}

fn render_preview(app: &App, theme: &PantryTheme, area: Rect, focused: bool, buf: &mut Buffer) {
    let preview_bg = app.preview_bg();
    let bg_color = preview_bg.map_or(theme.panel_bg, |(_, c)| c);
    let bg_label = preview_bg.map(|(name, _)| name);

    // Fill preview area with the selected background.
    if preview_bg.is_some() {
        fill_bg_area(buf, area, bg_color);
    }

    if let Some(idx) = app.nav().selected_ingredient() {
        render_single_ingredient(app, theme, area, focused, idx, bg_label, buf);
    } else if let Some((widget_name, items)) = app.nav().selected_widget_items() {
        render_gallery(app, theme, area, widget_name, items, bg_label, buf);
    } else if app.nav().is_empty() && TAB_LABELS[app.active_tab] == "Styles" {
        render_stylesheet_prompt(theme, area, buf);
    } else {
        let empty = Paragraph::new("Select an ingredient from the sidebar")
            .style(Style::default().fg(theme.text_dim));

        empty.render(area, buf);
    }
}

fn fill_bg_area(buf: &mut Buffer, area: Rect, color: Color) {
    for y in area.y..area.bottom() {
        for x in area.x..area.right() {
            buf[(x, y)].set_bg(color);
        }
    }
}

fn render_single_ingredient(
    app: &App,
    theme: &PantryTheme,
    area: Rect,
    focused: bool,
    idx: usize,
    bg_label: Option<&str>,
    buf: &mut Buffer,
) {
    let ingredient = &app.ingredients[idx];

    let description = ingredient.description();
    let props = ingredient.props();
    let doc_height = doc_panel_height(description, props);

    let [header_area, body] =
        Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(area);

    let mut spans = vec![
        Span::styled(
            format!(" {} ", ingredient.group()),
            Style::default().fg(theme.text_dim),
        ),
        Span::styled("› ", Style::default().fg(theme.border)),
        Span::styled(ingredient.name(), Style::default().fg(theme.text)),
        Span::raw("  "),
        Span::styled(ingredient.source(), Style::default().fg(theme.text_dim)),
    ];

    if let Some(label) = bg_label {
        spans.push(Span::styled(
            format!("  ▪ {label}"),
            Style::default().fg(theme.text_dim),
        ));
    }

    buf.set_line(
        header_area.x,
        header_area.y,
        &Line::from(spans),
        header_area.width,
    );

    if doc_height > 0 {
        let max_doc = (body.height * 2 / 5).max(4);
        let clamped = doc_height.min(max_doc);

        let [canvas, doc_area] =
            Layout::vertical([Constraint::Min(3), Constraint::Length(clamped)]).areas(body);

        let pane = Pane::new(ingredient.name(), ingredient.as_ref(), focused, theme);
        pane.render(canvas, buf);
        render_doc_panel(theme, description, props, doc_area, buf);
    } else {
        let pane = Pane::new(ingredient.name(), ingredient.as_ref(), focused, theme);
        pane.render(body, buf);
    }
}

const GALLERY_ITEM_HEIGHT: u16 = 14;
const GALLERY_MARGIN: u16 = 1;

/// Render all variants of a widget vertically with scroll.
fn render_gallery(
    app: &App,
    theme: &PantryTheme,
    area: Rect,
    widget_name: &str,
    items: &[usize],
    bg_label: Option<&str>,
    buf: &mut Buffer,
) {
    let [header_area, body] =
        Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(area);

    let count = items.len();
    let mut spans = vec![
        Span::styled(
            format!(" {} ", widget_name),
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  {count} variant{}", if count == 1 { "" } else { "s" }),
            Style::default().fg(theme.text_dim),
        ),
    ];

    if let Some(label) = bg_label {
        spans.push(Span::styled(
            format!("  ▪ {label}"),
            Style::default().fg(theme.text_dim),
        ));
    }

    buf.set_line(
        header_area.x,
        header_area.y,
        &Line::from(spans),
        header_area.width,
    );

    if body.height == 0 || items.is_empty() {
        return;
    }

    let step = GALLERY_ITEM_HEIGHT + GALLERY_MARGIN;
    let total_height = items.len() as u16 * step;
    let max_scroll = total_height.saturating_sub(body.height) as usize;
    let scroll = app.gallery_scroll.min(max_scroll);

    for (i, &idx) in items.iter().enumerate() {
        let item_top = (i as u16) * step;
        let item_bot = item_top + GALLERY_ITEM_HEIGHT;

        if item_bot <= scroll as u16 {
            continue;
        }

        let render_y = body.y as i32 + item_top as i32 - scroll as i32;
        if render_y >= body.bottom() as i32 {
            break;
        }

        let ingredient = &app.ingredients[idx];

        let visible_top = (render_y.max(body.y as i32)) as u16;
        let visible_bot =
            ((render_y + GALLERY_ITEM_HEIGHT as i32).min(body.bottom() as i32)) as u16;
        let visible_height = visible_bot.saturating_sub(visible_top);

        if visible_height == 0 {
            continue;
        }

        let item_area = Rect {
            x: body.x,
            y: visible_top,
            width: body.width.saturating_sub(1),
            height: visible_height,
        };

        // Pane borders require the full item height; partial items get a plain label.
        if render_y >= body.y as i32 && visible_height >= GALLERY_ITEM_HEIGHT {
            Pane::new(ingredient.name(), ingredient.as_ref(), false, theme).render(item_area, buf);
        } else {
            let label = Line::styled(
                format!(" {} ", ingredient.name()),
                Style::default().fg(theme.text_dim),
            );

            buf.set_line(item_area.x, item_area.y, &label, item_area.width);

            let inner = Rect {
                y: item_area.y + 1,
                height: item_area.height.saturating_sub(1),
                ..item_area
            };

            if inner.height > 0 {
                ingredient.render(inner, buf);
            }
        }
    }

    if total_height > body.height {
        let mut state = ScrollbarState::new(max_scroll).position(scroll);

        Scrollbar::new(ScrollbarOrientation::VerticalRight).render(body, buf, &mut state);
    }
}

/// Height needed to render the doc panel content.
fn doc_panel_height(description: &str, props: &[PropInfo]) -> u16 {
    if description.is_empty() && props.is_empty() {
        return 0;
    }
    // 1 separator + 1 description (if present) + 1 blank + 1 header + N props
    let desc_lines: u16 = if description.is_empty() { 0 } else { 2 };
    let props_lines: u16 = if props.is_empty() {
        0
    } else {
        1 + props.len() as u16
    };
    1 + desc_lines + props_lines
}

fn render_doc_panel(
    theme: &PantryTheme,
    description: &str,
    props: &[PropInfo],
    area: Rect,
    buf: &mut Buffer,
) {
    if area.is_empty() {
        return;
    }

    let accent = Style::default().fg(theme.doc_accent);
    let dim = Style::default().fg(theme.text_dim);
    let text = Style::default().fg(theme.doc_text);

    let sep = "─".repeat(area.width as usize);
    buf.set_line(area.x, area.y, &Line::styled(&*sep, dim), area.width);

    let mut y = area.y + 1;
    let x = area.x + 1;
    let w = area.width.saturating_sub(1);

    if !description.is_empty() && y < area.y + area.height {
        buf.set_line(x, y, &Line::styled(description, text), w);
        y += 2;
    }

    if !props.is_empty() && y < area.y + area.height {
        let name_w = props.iter().map(|p| p.name.len()).max().unwrap_or(0);
        let ty_w = props.iter().map(|p| p.ty.len()).max().unwrap_or(0);

        let header = Line::from(vec![
            Span::styled(format!("{:<name_w$}", "PROP"), accent),
            Span::styled("  ", dim),
            Span::styled(format!("{:<ty_w$}", "TYPE"), accent),
            Span::styled("  ", dim),
            Span::styled("DESCRIPTION", accent),
        ]);
        buf.set_line(x, y, &header, w);
        y += 1;

        for prop in props {
            if y >= area.y + area.height {
                break;
            }
            let line = Line::from(vec![
                Span::styled(
                    format!("{:<name_w$}", prop.name),
                    Style::default().fg(theme.text),
                ),
                Span::styled("  ", dim),
                Span::styled(
                    format!("{:<ty_w$}", prop.ty),
                    Style::default().fg(theme.doc_type),
                ),
                Span::styled("  ", dim),
                Span::styled(prop.description, text),
            ]);
            buf.set_line(x, y, &line, w);
            y += 1;
        }
    }
}

fn render_bottom_bar(app: &App, theme: &PantryTheme, area: Rect, buf: &mut Buffer) {
    let accent = Style::default().fg(theme.accent);
    let dim = Style::default().fg(theme.text_dim);

    let sep = "─".repeat(area.width as usize);
    buf.set_line(area.x, area.y, &Line::styled(&*sep, dim), area.width);
    let area = Rect {
        y: area.y + 1,
        height: area.height.saturating_sub(1),
        ..area
    };

    let hints = match app.focus {
        Focus::Preview => vec![
            Span::styled(" ↑↓", accent),
            Span::styled(" navigate  ", dim),
            Span::styled("␣", accent),
            Span::styled(" select  ", dim),
            Span::styled("click", accent),
            Span::styled(" interact  ", dim),
            Span::styled("f", accent),
            Span::styled(" fullscreen  ", dim),
            Span::styled("Esc", accent),
            Span::styled(" back", dim),
        ],
        Focus::Sidebar => {
            let mut spans = vec![
                Span::styled(" ↑↓", accent),
                Span::styled(" navigate  ", dim),
                Span::styled("→", accent),
                Span::styled(" expand  ", dim),
                Span::styled("←", accent),
                Span::styled(" collapse  ", dim),
                Span::styled("↵", accent),
                Span::styled(" select  ", dim),
            ];
            if app.nav().selected_ingredient().is_some() {
                spans.push(Span::styled("f", accent));
                spans.push(Span::styled(" fullscreen  ", dim));
            }
            spans.extend([
                Span::styled("1-4", accent),
                Span::styled(" tabs  ", dim),
                Span::styled("t", accent),
                Span::styled(
                    if app.dark_mode {
                        " light theme  "
                    } else {
                        " dark theme  "
                    },
                    dim,
                ),
            ]);

            if !app.preview_backgrounds.is_empty() {
                spans.push(Span::styled("b", accent));
                spans.push(Span::styled(" background  ", dim));
            }

            spans.extend([
                Span::styled("c", accent),
                Span::styled(" colors  ", dim),
                Span::styled("q", accent),
                Span::styled(" quit", dim),
            ]);
            spans
        }
        Focus::Fullscreen => return,
    };

    let depth_label = app.color_depth.label();
    let theme_label = if app.dark_mode { "dark" } else { "light" };
    let indicator = vec![
        Span::styled("● ", accent),
        Span::styled(theme_label, Style::default().fg(theme.indicator)),
        Span::styled(" · ", dim),
        Span::styled(depth_label, Style::default().fg(theme.indicator)),
        Span::raw(" "),
    ];
    let indicator_width: u16 = indicator.iter().map(|s| s.width() as u16).sum();

    let [hints_area, indicator_area] =
        Layout::horizontal([Constraint::Min(0), Constraint::Length(indicator_width)]).areas(area);

    buf.set_line(
        hints_area.x,
        hints_area.y,
        &Line::from(hints),
        hints_area.width,
    );
    buf.set_line(
        indicator_area.x,
        indicator_area.y,
        &Line::from(indicator),
        indicator_area.width,
    );
}

fn fill_bg(buf: &mut Buffer, x: u16, y: u16, width: u16, color: Color) {
    for dx in 0..width {
        buf[(x + dx, y)].set_bg(color);
    }
}

// ── Scrollbar hit-testing ──────────────────────────────────────────

/// Where a click landed on a vertical scrollbar.
pub(crate) enum ScrollbarHit {
    UpArrow,
    DownArrow,
    Above,
    Below,
    Thumb,
}

/// Sidebar scrollbar area, if content overflows.
pub(crate) fn sidebar_scrollbar_area(regions: &Regions, nav: &crate::nav::NavTree) -> Option<Rect> {
    let viewport = regions.sidebar.height.saturating_sub(1) as usize;

    if nav.visible().len() <= viewport {
        return None;
    }

    Some(regions.sidebar.inner(Margin {
        vertical: 1,
        horizontal: 0,
    }))
}

/// Gallery scrollbar area and max scroll value, if gallery content overflows.
pub(crate) fn gallery_scrollbar_info(
    regions: &Regions,
    nav: &crate::nav::NavTree,
) -> Option<(Rect, usize)> {
    let (_, items) = nav.selected_widget_items()?;

    let [_, body] =
        Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(regions.preview);

    let step = GALLERY_ITEM_HEIGHT + GALLERY_MARGIN;
    let total = items.len() as u16 * step;

    if total <= body.height {
        return None;
    }

    Some((body, total.saturating_sub(body.height) as usize))
}

/// Hit-test a row against a vertical scrollbar with arrows.
pub(crate) fn scrollbar_hit_test(
    area: Rect,
    content_length: usize,
    position: usize,
    row: u16,
) -> Option<ScrollbarHit> {
    if content_length <= 1 || area.height < 3 {
        return None;
    }

    if row == area.y {
        return Some(ScrollbarHit::UpArrow);
    }

    if row == area.bottom() - 1 {
        return Some(ScrollbarHit::DownArrow);
    }

    let track_top = area.y + 1;
    let track_height = area.height.saturating_sub(2);

    if row < track_top || row >= track_top + track_height {
        return None;
    }

    let thumb_row =
        track_top + (position as u32 * track_height as u32 / (content_length - 1) as u32) as u16;

    if row == thumb_row {
        Some(ScrollbarHit::Thumb)
    } else if row < thumb_row {
        Some(ScrollbarHit::Above)
    } else {
        Some(ScrollbarHit::Below)
    }
}

/// Map a mouse row to a scroll position within the scrollbar track.
pub(crate) fn scrollbar_position_from_row(area: Rect, content_max: usize, row: u16) -> usize {
    let track_top = area.y + 1;
    let track_height = area.height.saturating_sub(2) as usize;

    if track_height == 0 || content_max == 0 {
        return 0;
    }

    let offset = row.saturating_sub(track_top).min(track_height as u16 - 1) as usize;
    offset * content_max / (track_height - 1).max(1)
}

/// Compute the area where the ingredient renders inside the preview pane.
///
/// Replicates the layout from `render_preview`: header (1 line) → body → optional doc panel,
/// then the Pane border (1px each side).
pub(crate) fn ingredient_area(
    regions: &Regions,
    ingredients: &[Box<dyn crate::Ingredient>],
    nav: &crate::nav::NavTree,
) -> Rect {
    let Some(idx) = nav.selected_ingredient() else {
        return Rect::default();
    };
    let ingredient = &ingredients[idx];
    let doc_height = doc_panel_height(ingredient.description(), ingredient.props());

    let [_header, body] =
        Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(regions.preview);

    let canvas = if doc_height > 0 {
        let max_doc = (body.height * 2 / 5).max(4);
        let clamped = doc_height.min(max_doc);
        Layout::vertical([Constraint::Min(3), Constraint::Length(clamped)]).areas::<2>(body)[0]
    } else {
        body
    };

    // Pane adds a 1px border on all sides.
    canvas.inner(Margin {
        vertical: 1,
        horizontal: 1,
    })
}

fn render_stylesheet_prompt(theme: &PantryTheme, area: Rect, buf: &mut Buffer) {
    let dim = Style::new().fg(theme.text_dim);
    let text = Style::new().fg(theme.doc_text);
    let code = Style::new().fg(theme.text);

    let lines: &[Line] = &[
        Line::from(vec![
            Span::styled("  Add stylesheet sections to ", text),
            Span::styled("pantry.toml", code),
            Span::styled(":", text),
        ]),
        Line::default(),
        Line::from(Span::styled("     [colors.brand]", dim)),
        Line::from(Span::styled("     deep_purple = \"#2E1574\"", dim)),
        Line::from(Span::styled("     white = \"#FFFFFF\"", dim)),
        Line::default(),
        Line::from(Span::styled("     [typography]", dim)),
        Line::from(Span::styled(
            "     text = { color = \"#FFF\", description = \"Primary\" }",
            dim,
        )),
    ];

    for (i, line) in lines.iter().enumerate() {
        let y = area.y + 1 + i as u16;
        if y >= area.bottom() {
            break;
        }
        buf.set_line(area.x, y, line, area.width);
    }
}
