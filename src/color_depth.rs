use ratatui::{buffer::Buffer, style::Color};

/// Terminal color depth tiers for emulation.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum ColorDepth {
    Mono,
    Ansi8,
    Ansi16,
    Indexed256,
    #[default]
    TrueColor,
}

impl ColorDepth {
    pub fn cycle(self) -> Self {
        match self {
            Self::TrueColor => Self::Indexed256,
            Self::Indexed256 => Self::Ansi16,
            Self::Ansi16 => Self::Ansi8,
            Self::Ansi8 => Self::Mono,
            Self::Mono => Self::TrueColor,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::TrueColor => "24-bit",
            Self::Indexed256 => "256",
            Self::Ansi16 => "16",
            Self::Ansi8 => "8",
            Self::Mono => "mono",
        }
    }
}

/// Rewrite every cell's fg/bg in the buffer to emulate the given depth.
pub(crate) fn quantize_buffer(buf: &mut Buffer, depth: ColorDepth) {
    for cell in &mut buf.content {
        let style = cell.style();
        let fg = style.fg.map(|c| quantize(c, depth));
        let bg = style.bg.map(|c| quantize(c, depth));
        cell.set_style(ratatui::style::Style { fg, bg, ..style });
    }
}

fn quantize(color: Color, depth: ColorDepth) -> Color {
    match depth {
        ColorDepth::TrueColor => color,
        ColorDepth::Indexed256 => to_indexed(color),
        ColorDepth::Ansi16 => to_ansi16(color),
        ColorDepth::Ansi8 => to_ansi8(color),
        ColorDepth::Mono => to_mono(color),
    }
}

// --- Truecolor → 256-color ------------------------------------------------

/// The six channel values in the 6×6×6 xterm color cube.
const CUBE_VALUES: [u8; 6] = [0, 95, 135, 175, 215, 255];

fn to_indexed(color: Color) -> Color {
    match color {
        Color::Rgb(r, g, b) => {
            let ri = nearest_cube_index(r);
            let gi = nearest_cube_index(g);
            let bi = nearest_cube_index(b);
            let cube_idx = 16 + 36 * ri + 6 * gi + bi;

            // Check if a grayscale ramp entry is closer.
            let cube_r = CUBE_VALUES[ri as usize];
            let cube_g = CUBE_VALUES[gi as usize];
            let cube_b = CUBE_VALUES[bi as usize];
            let cube_dist = color_dist(r, g, b, cube_r, cube_g, cube_b);

            let gray_idx = nearest_gray_index(r, g, b);
            let gray_val = 8 + 10 * gray_idx;
            let gray_dist = color_dist(r, g, b, gray_val, gray_val, gray_val);

            if gray_dist < cube_dist {
                Color::Indexed(232 + gray_idx)
            } else {
                Color::Indexed(cube_idx)
            }
        }
        other => other,
    }
}

fn nearest_cube_index(val: u8) -> u8 {
    CUBE_VALUES
        .iter()
        .enumerate()
        .min_by_key(|(_, cv)| (val as i16 - **cv as i16).unsigned_abs())
        .map(|(i, _)| i as u8)
        .unwrap_or(0)
}

fn nearest_gray_index(r: u8, g: u8, b: u8) -> u8 {
    // Grayscale ramp: indices 232-255, values 8, 18, 28, ... 238
    let lum = (r as u16 + g as u16 + b as u16) / 3;
    ((lum.saturating_sub(4)) / 10).min(23) as u8
}

// --- Truecolor → 16-color -------------------------------------------------

/// The 16 ANSI colors in approximate sRGB.
const ANSI16: [(u8, u8, u8); 16] = [
    (0, 0, 0),       // Black
    (170, 0, 0),     // Red
    (0, 170, 0),     // Green
    (170, 170, 0),   // Yellow
    (0, 0, 170),     // Blue
    (170, 0, 170),   // Magenta
    (0, 170, 170),   // Cyan
    (170, 170, 170), // Gray (ANSI "white")
    (85, 85, 85),    // DarkGray (bright black)
    (255, 85, 85),   // LightRed
    (85, 255, 85),   // LightGreen
    (255, 255, 85),  // LightYellow
    (85, 85, 255),   // LightBlue
    (255, 85, 255),  // LightMagenta
    (85, 255, 255),  // LightCyan
    (255, 255, 255), // White (bright white)
];

const ANSI16_COLORS: [Color; 16] = [
    Color::Black,
    Color::Red,
    Color::Green,
    Color::Yellow,
    Color::Blue,
    Color::Magenta,
    Color::Cyan,
    Color::Gray,
    Color::DarkGray,
    Color::LightRed,
    Color::LightGreen,
    Color::LightYellow,
    Color::LightBlue,
    Color::LightMagenta,
    Color::LightCyan,
    Color::White,
];

fn to_ansi16(color: Color) -> Color {
    match color {
        Color::Rgb(r, g, b) => nearest_ansi(r, g, b, 16),
        Color::Indexed(i) if i >= 16 => {
            let (r, g, b) = indexed_to_rgb(i);
            nearest_ansi(r, g, b, 16)
        }
        other => other,
    }
}

// --- Truecolor → 8-color --------------------------------------------------

fn to_ansi8(color: Color) -> Color {
    match color {
        Color::Rgb(r, g, b) => nearest_ansi(r, g, b, 8),
        Color::Indexed(i) if i >= 8 => {
            let (r, g, b) = indexed_to_rgb(i);
            nearest_ansi(r, g, b, 8)
        }
        // Collapse bright named variants to dim
        Color::DarkGray => Color::Black,
        Color::LightRed => Color::Red,
        Color::LightGreen => Color::Green,
        Color::LightYellow => Color::Yellow,
        Color::LightBlue => Color::Blue,
        Color::LightMagenta => Color::Magenta,
        Color::LightCyan => Color::Cyan,
        Color::White => Color::Gray,
        other => other,
    }
}

// --- Monochrome ------------------------------------------------------------

fn to_mono(_: Color) -> Color {
    Color::Reset
}

// --- Shared helpers --------------------------------------------------------

fn nearest_ansi(r: u8, g: u8, b: u8, count: usize) -> Color {
    ANSI16[..count]
        .iter()
        .zip(ANSI16_COLORS[..count].iter())
        .min_by_key(|((ar, ag, ab), _)| color_dist(r, g, b, *ar, *ag, *ab))
        .map(|(_, c)| *c)
        .unwrap_or(Color::Reset)
}

fn indexed_to_rgb(i: u8) -> (u8, u8, u8) {
    match i {
        0..=15 => ANSI16[i as usize],
        16..=231 => {
            let idx = i - 16;
            let ri = idx / 36;
            let gi = (idx % 36) / 6;
            let bi = idx % 6;
            (CUBE_VALUES[ri as usize], CUBE_VALUES[gi as usize], CUBE_VALUES[bi as usize])
        }
        232..=255 => {
            let v = 8 + 10 * (i - 232);
            (v, v, v)
        }
    }
}

/// Weighted squared Euclidean distance in sRGB. Cheap perceptual approximation.
fn color_dist(r1: u8, g1: u8, b1: u8, r2: u8, g2: u8, b2: u8) -> u32 {
    let dr = r1 as i32 - r2 as i32;
    let dg = g1 as i32 - g2 as i32;
    let db = b1 as i32 - b2 as i32;
    // Human vision is most sensitive to green, least to blue.
    (2 * dr * dr + 4 * dg * dg + 3 * db * db) as u32
}
