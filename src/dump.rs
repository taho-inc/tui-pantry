use std::fmt::Write as _;
use std::io::{self, Write};

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier};

use crate::Ingredient;

const DEFAULT_WIDTH: u16 = 80;
const DEFAULT_HEIGHT: u16 = 24;

pub(crate) enum HeadlessAction {
    Dump(DumpArgs),
    List,
}

pub(crate) struct DumpArgs {
    pub group: String,
    pub variant: Option<String>,
    pub width: u16,
    pub height: u16,
}

/// Check env args for `--dump` or `--list` before entering the TUI.
pub(crate) fn parse_headless_args() -> Option<HeadlessAction> {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--list") {
        return Some(HeadlessAction::List);
    }

    let pos = args.iter().position(|a| a == "--dump")?;
    let group = args.get(pos + 1)?.clone();

    let mut variant = None;
    let mut width = DEFAULT_WIDTH;
    let mut height = DEFAULT_HEIGHT;

    let mut i = pos + 2;
    while i < args.len() {
        match args[i].as_str() {
            "--variant" => {
                variant = args.get(i + 1).cloned();
                i += 2;
            }
            "--size" => {
                if let Some(size_str) = args.get(i + 1)
                    && let Some((w, h)) = parse_size(size_str)
                {
                    width = w;
                    height = h;
                }
                i += 2;
            }
            _ => i += 1,
        }
    }

    Some(HeadlessAction::Dump(DumpArgs {
        group,
        variant,
        width,
        height,
    }))
}

fn parse_size(s: &str) -> Option<(u16, u16)> {
    let (w, h) = s.split_once('x')?;
    Some((w.parse().ok()?, h.parse().ok()?))
}

// ---------------------------------------------------------------------------
// List
// ---------------------------------------------------------------------------

pub(crate) fn list(ingredients: &[Box<dyn Ingredient>]) -> io::Result<()> {
    let stdout = io::stdout();
    let mut out = stdout.lock();

    for ing in ingredients {
        writeln!(out, "{}/{}", ing.group(), ing.name())?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Dump
// ---------------------------------------------------------------------------

pub(crate) fn dump(ingredients: &[Box<dyn Ingredient>], args: &DumpArgs) -> io::Result<()> {
    let matches: Vec<&dyn Ingredient> = ingredients
        .iter()
        .map(AsRef::as_ref)
        .filter(|ing| ing.group() == args.group)
        .filter(|ing| {
            args.variant
                .as_ref()
                .is_none_or(|v| ing.name() == v.as_str())
        })
        .collect();

    if matches.is_empty() {
        let mut groups: Vec<&str> = ingredients.iter().map(|i| i.group()).collect();
        groups.sort();
        groups.dedup();

        eprintln!("no ingredient matching group {:?}", args.group);
        if !groups.is_empty() {
            eprintln!("available: {}", groups.join(", "));
        }

        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "ingredient not found",
        ));
    }

    let stdout = io::stdout();
    let mut out = stdout.lock();

    for (i, ingredient) in matches.iter().enumerate() {
        if i > 0 {
            writeln!(out)?;
        }

        if matches.len() > 1 {
            writeln!(out, "--- {} ---", ingredient.name())?;
        }

        let ansi = render_to_ansi(*ingredient, args.width, args.height);
        out.write_all(ansi.as_bytes())?;
    }

    Ok(())
}

fn render_to_ansi(ingredient: &dyn Ingredient, width: u16, height: u16) -> String {
    let area = Rect::new(0, 0, width, height);
    let mut buf = Buffer::empty(area);
    ingredient.render(area, &mut buf);
    buffer_to_ansi(&buf)
}

// ---------------------------------------------------------------------------
// Buffer → ANSI
// ---------------------------------------------------------------------------

fn buffer_to_ansi(buf: &Buffer) -> String {
    let mut out = String::new();
    let mut prev = ratatui::style::Style::default();

    for y in 0..buf.area.height {
        if y > 0 {
            out.push_str("\x1b[0m\n");
            prev = ratatui::style::Style::default();
        }

        for x in 0..buf.area.width {
            let cell = &buf[(x, y)];
            let style = cell.style();

            if style != prev {
                out.push_str("\x1b[0m");
                write_style(&mut out, &style);
                prev = style;
            }

            out.push_str(cell.symbol());
        }
    }

    out.push_str("\x1b[0m\n");
    out
}

fn write_style(out: &mut String, style: &ratatui::style::Style) {
    if let Some(fg) = style.fg {
        write_color(out, fg, 30);
    }

    if let Some(bg) = style.bg {
        write_color(out, bg, 40);
    }

    let m = style.add_modifier;

    if m.contains(Modifier::BOLD) {
        out.push_str("\x1b[1m");
    }
    if m.contains(Modifier::DIM) {
        out.push_str("\x1b[2m");
    }
    if m.contains(Modifier::ITALIC) {
        out.push_str("\x1b[3m");
    }
    if m.contains(Modifier::UNDERLINED) {
        out.push_str("\x1b[4m");
    }
    if m.contains(Modifier::REVERSED) {
        out.push_str("\x1b[7m");
    }
    if m.contains(Modifier::CROSSED_OUT) {
        out.push_str("\x1b[9m");
    }
}

/// Write a single ANSI color escape. `base` is 30 (fg) or 40 (bg).
fn write_color(out: &mut String, color: Color, base: u8) {
    match color {
        Color::Indexed(i) => {
            let _ = write!(out, "\x1b[{};5;{i}m", base + 8);
        }
        Color::Rgb(r, g, b) => {
            let _ = write!(out, "\x1b[{};2;{r};{g};{b}m", base + 8);
        }
        named => {
            let _ = write!(out, "\x1b[{}m", base + named_offset(named));
        }
    }
}

fn named_offset(color: Color) -> u8 {
    match color {
        Color::Black => 0,
        Color::Red => 1,
        Color::Green => 2,
        Color::Yellow => 3,
        Color::Blue => 4,
        Color::Magenta => 5,
        Color::Cyan => 6,
        Color::Gray => 7,
        Color::Reset => 9,
        Color::DarkGray => 60,
        Color::LightRed => 61,
        Color::LightGreen => 62,
        Color::LightYellow => 63,
        Color::LightBlue => 64,
        Color::LightMagenta => 65,
        Color::LightCyan => 66,
        Color::White => 67,
        _ => 9, // unreachable for named colors; fall back to reset
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Style;

    #[test]
    fn buffer_to_ansi_empty() {
        let buf = Buffer::empty(Rect::new(0, 0, 3, 1));
        let ansi = buffer_to_ansi(&buf);
        // 3 spaces + reset + newline
        assert!(ansi.contains("   "));
        assert!(ansi.ends_with("\x1b[0m\n"));
    }

    #[test]
    fn buffer_to_ansi_colored_cell() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 1, 1));
        buf[(0, 0)].set_symbol("X");
        buf[(0, 0)].set_style(Style::new().fg(Color::Red));
        let ansi = buffer_to_ansi(&buf);
        assert!(ansi.contains("\x1b[31m"));
        assert!(ansi.contains("X"));
    }

    #[test]
    fn buffer_to_ansi_rgb() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 1, 1));
        buf[(0, 0)].set_symbol("@");
        buf[(0, 0)].set_style(Style::new().fg(Color::Rgb(100, 200, 50)));
        let ansi = buffer_to_ansi(&buf);
        assert!(ansi.contains("\x1b[38;2;100;200;50m"));
    }

    #[test]
    fn parse_size_valid() {
        assert_eq!(parse_size("40x10"), Some((40, 10)));
    }

    #[test]
    fn parse_size_invalid() {
        assert_eq!(parse_size("abc"), None);
        assert_eq!(parse_size("40"), None);
    }
}
