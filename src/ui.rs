use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

use crate::Pane;
use crate::ingredient::PropInfo;

use crate::app::{App, Focus, TAB_LABELS};
use crate::nav::NavEntry;
use crate::swatch::GradientSwatch;
use crate::theme::PantryTheme;

const SIDEBAR_WIDTH: u16 = 28;
const BOTTOM_BAR_HEIGHT: u16 = 1;
const TOP_BAR_HEIGHT: u16 = 1;

pub(crate) fn render(app: &App, area: Rect, buf: &mut Buffer) {
    let theme = &app.theme;

    GradientSwatch::new(theme.gradient_left, theme.gradient_right).render(area, buf);

    let inner = area.inner(Margin { vertical: 1, horizontal: 2 });

    let [top_bar, main_area, bottom] = Layout::vertical([
        Constraint::Length(TOP_BAR_HEIGHT),
        Constraint::Min(0),
        Constraint::Length(BOTTOM_BAR_HEIGHT),
    ])
    .areas(inner);

    let [sidebar, preview] = Layout::horizontal([
        Constraint::Length(SIDEBAR_WIDTH),
        Constraint::Min(0),
    ])
    .areas(main_area);

    Clear.render(inner, buf);
    Block::new()
        .style(Style::new().bg(theme.panel_bg))
        .render(inner, buf);

    let focused = app.focus == Focus::Preview;

    render_top_bar(app, theme, top_bar, buf);
    render_sidebar(app, theme, sidebar, buf);
    render_preview(app, theme, preview, focused, buf);
    render_bottom_bar(theme, focused, bottom, buf);
}

fn render_top_bar(app: &App, theme: &PantryTheme, area: Rect, buf: &mut Buffer) {
    let app_name = Span::styled(
        " tui-pantry ",
        Style::new().fg(theme.accent).add_modifier(Modifier::BOLD),
    );

    let mut tab_spans: Vec<Span> = Vec::new();
    for (i, label) in TAB_LABELS.iter().enumerate() {
        if i > 0 {
            tab_spans.push(Span::styled(" · ", Style::new().fg(theme.border)));
        }

        let style = if i == app.active_tab {
            Style::new().fg(theme.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(theme.text_dim)
        };

        tab_spans.push(Span::styled(*label, style));
    }
    tab_spans.push(Span::raw(" "));

    let tabs_width: u16 = tab_spans.iter().map(|s| s.width() as u16).sum();

    let [title_area, tabs_area] = Layout::horizontal([
        Constraint::Min(0),
        Constraint::Length(tabs_width),
    ])
    .areas(area);

    buf.set_line(title_area.x, title_area.y, &Line::from(vec![app_name]), title_area.width);
    buf.set_line(tabs_area.x, tabs_area.y, &Line::from(tab_spans), tabs_area.width);
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

    let header_line = Line::from(vec![
        Span::styled(
            format!(" {tab_label} "),
            Style::default().fg(theme.text_dim).add_modifier(Modifier::BOLD),
        ),
    ]);

    buf.set_line(inner.x, inner.y, &header_line, inner.width);

    if nav.is_empty() {
        if inner.height > 2 {
            let empty_msg = Line::from(Span::styled("  (empty)", Style::default().fg(theme.text_dim)));
            buf.set_line(inner.x, inner.y + 2, &empty_msg, inner.width);
        }
        return;
    }

    let entries = nav.visible();
    let selected_ingredient = nav.selected_ingredient();
    let offset = nav.scroll_offset;

    for (i, entry) in entries.iter().enumerate().skip(offset) {
        let y = inner.y + 1 + (i - offset) as u16;
        if y >= inner.y + inner.height {
            break;
        }

        let is_cursor = i == nav.cursor;

        match entry {
            NavEntry::Group { name, expanded } => {
                let caret = if *expanded { "▼" } else { "▶" };
                let style = if is_cursor {
                    Style::default().fg(theme.accent).bg(theme.cursor_bg)
                } else {
                    Style::default().fg(theme.text)
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

            NavEntry::Variant { ingredient_idx, .. } => {
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

                let line = Line::from(vec![
                    Span::styled("   ◆ ", marker_style),
                    Span::styled(ingredient.name(), style),
                ]);

                buf.set_line(inner.x, y, &line, inner.width);

                if is_cursor {
                    fill_bg(buf, inner.x, y, inner.width, theme.cursor_bg);
                }
            }
        }
    }
}

fn render_preview(app: &App, theme: &PantryTheme, area: Rect, focused: bool, buf: &mut Buffer) {
    if let Some(idx) = app.nav().selected_ingredient() {
        let ingredient = &app.ingredients[idx];

        let description = ingredient.description();
        let props = ingredient.props();
        let doc_height = doc_panel_height(description, props);

        let [header_area, body] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .areas(area);

        let breadcrumb = Line::from(vec![
            Span::styled(format!(" {} ", ingredient.group()), Style::default().fg(theme.text_dim)),
            Span::styled("› ", Style::default().fg(theme.border)),
            Span::styled(ingredient.name(), Style::default().fg(theme.text)),
            Span::raw("  "),
            Span::styled(ingredient.source(), Style::default().fg(theme.text_dim)),
        ]);

        buf.set_line(header_area.x, header_area.y, &breadcrumb, header_area.width);

        if doc_height > 0 {
            let max_doc = (body.height * 2 / 5).max(4);
            let clamped = doc_height.min(max_doc);

            let [canvas, doc_area] = Layout::vertical([
                Constraint::Min(3),
                Constraint::Length(clamped),
            ])
            .areas(body);

            let pane = Pane::new(ingredient.name(), ingredient.as_ref(), focused, theme);
            pane.render(canvas, buf);
            render_doc_panel(theme, description, props, doc_area, buf);
        } else {
            let pane = Pane::new(ingredient.name(), ingredient.as_ref(), focused, theme);
            pane.render(body, buf);
        }
    } else if app.nav().is_empty() && TAB_LABELS[app.active_tab] == "Styles" {
        render_stylesheet_prompt(theme, area, buf);
    } else {
        let empty = Paragraph::new("Select an ingredient from the sidebar")
            .style(Style::default().fg(theme.text_dim));

        empty.render(area, buf);
    }
}

/// Height needed to render the doc panel content.
fn doc_panel_height(description: &str, props: &[PropInfo]) -> u16 {
    if description.is_empty() && props.is_empty() {
        return 0;
    }
    // 1 separator + 1 description (if present) + 1 blank + 1 header + N props
    let desc_lines: u16 = if description.is_empty() { 0 } else { 2 };
    let props_lines: u16 = if props.is_empty() { 0 } else { 1 + props.len() as u16 };
    1 + desc_lines + props_lines
}

fn render_doc_panel(
    theme: &PantryTheme,
    description: &str,
    props: &[PropInfo],
    area: Rect,
    buf: &mut Buffer,
) {
    let accent = Style::default().fg(Color::Rgb(232, 164, 90));
    let dim = Style::default().fg(theme.text_dim);
    let text = Style::default().fg(Color::Gray);

    // Separator line
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
        // Column widths: find max name and type lengths
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
                Span::styled(format!("{:<name_w$}", prop.name), Style::default().fg(theme.text)),
                Span::styled("  ", dim),
                Span::styled(format!("{:<ty_w$}", prop.ty), Style::default().fg(Color::Rgb(140, 140, 200))),
                Span::styled("  ", dim),
                Span::styled(prop.description, text),
            ]);
            buf.set_line(x, y, &line, w);
            y += 1;
        }
    }
}

fn render_bottom_bar(theme: &PantryTheme, focused: bool, area: Rect, buf: &mut Buffer) {
    let accent = Style::default().fg(theme.accent);
    let dim = Style::default().fg(theme.text_dim);

    let hints = if focused {
        Line::from(vec![
            Span::styled(" ↑↓", accent),
            Span::styled(" navigate  ", dim),
            Span::styled("Esc", accent),
            Span::styled(" back", dim),
        ])
    } else {
        Line::from(vec![
            Span::styled(" ↑↓", accent),
            Span::styled(" navigate  ", dim),
            Span::styled("→", accent),
            Span::styled(" expand  ", dim),
            Span::styled("←", accent),
            Span::styled(" collapse  ", dim),
            Span::styled("↵", accent),
            Span::styled(" select  ", dim),
            Span::styled("1-3", accent),
            Span::styled(" tabs  ", dim),
            Span::styled("q", accent),
            Span::styled(" quit", dim),
        ])
    };

    buf.set_line(area.x, area.y, &hints, area.width);
}

fn fill_bg(buf: &mut Buffer, x: u16, y: u16, width: u16, color: Color) {
    for dx in 0..width {
        buf[(x + dx, y)].set_bg(color);
    }
}

fn render_stylesheet_prompt(theme: &PantryTheme, area: Rect, buf: &mut Buffer) {
    let dim = Style::new().fg(theme.text_dim);
    let text = Style::new().fg(Color::Gray);
    let code = Style::new().fg(theme.text);

    let lines: &[Line] = &[
        Line::from(vec![
            Span::styled("  Create ", text),
            Span::styled("styles.toml", code),
            Span::styled(" next to your ", text),
            Span::styled("Cargo.toml", code),
            Span::styled(":", text),
        ]),
        Line::default(),
        Line::from(Span::styled("     [colors.brand]", dim)),
        Line::from(Span::styled("     deep_purple = \"#2E1574\"", dim)),
        Line::from(Span::styled("     white = \"#FFFFFF\"", dim)),
        Line::default(),
        Line::from(Span::styled("     [typography]", dim)),
        Line::from(Span::styled("     text = { color = \"#FFF\", description = \"Primary\" }", dim)),
    ];

    for (i, line) in lines.iter().enumerate() {
        let y = area.y + 1 + i as u16;
        if y >= area.bottom() {
            break;
        }
        buf.set_line(area.x, y, line, area.width);
    }
}
