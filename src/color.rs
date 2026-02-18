/// ANSI color support: 16 named colors + 256 indexed.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Named(u8),   // 0-15: standard + bright
    Indexed(u8), // 0-255
}

impl Color {
    pub fn fg_code(self) -> String {
        match self {
            Color::Named(n) if n < 8 => format!("\x1b[{}m", 30 + n),
            Color::Named(n) => format!("\x1b[{}m", 90 + n - 8),
            Color::Indexed(n) => format!("\x1b[38;5;{}m", n),
        }
    }
}

pub const RESET: &str = "\x1b[0m";

/// Default palette for auto-assigning series colors.
/// Green, cyan, magenta, yellow, red, blue — high-contrast on dark terminals.
pub const PALETTE: &[Color] = &[
    Color::Named(2),  // green
    Color::Named(6),  // cyan
    Color::Named(5),  // magenta
    Color::Named(3),  // yellow
    Color::Named(1),  // red
    Color::Named(4),  // blue
    Color::Named(10), // bright green
    Color::Named(14), // bright cyan
    Color::Named(13), // bright magenta
    Color::Named(11), // bright yellow
];

/// Parse a color string: named color or numeric index.
pub fn parse_color(s: &str) -> Result<Color, String> {
    if let Ok(n) = s.parse::<u16>() {
        if n > 255 {
            return Err(format!("color index out of range: {n} (0-255)"));
        }
        return Ok(Color::Indexed(n as u8));
    }

    match s.to_lowercase().as_str() {
        "black" => Ok(Color::Named(0)),
        "red" => Ok(Color::Named(1)),
        "green" => Ok(Color::Named(2)),
        "yellow" => Ok(Color::Named(3)),
        "blue" => Ok(Color::Named(4)),
        "magenta" => Ok(Color::Named(5)),
        "cyan" => Ok(Color::Named(6)),
        "white" => Ok(Color::Named(7)),
        "bright_black" | "bright black" => Ok(Color::Named(8)),
        "bright_red" | "bright red" => Ok(Color::Named(9)),
        "bright_green" | "bright green" => Ok(Color::Named(10)),
        "bright_yellow" | "bright yellow" => Ok(Color::Named(11)),
        "bright_blue" | "bright blue" => Ok(Color::Named(12)),
        "bright_magenta" | "bright magenta" => Ok(Color::Named(13)),
        "bright_cyan" | "bright cyan" => Ok(Color::Named(14)),
        "bright_white" | "bright white" => Ok(Color::Named(15)),
        _ => Err(format!("unknown color: {s:?}")),
    }
}

/// Resolved color configuration for rendering.
#[derive(Debug, Clone)]
pub enum ColorMode {
    /// No color output.
    Off,
    /// Single explicit color for all series.
    Single(Color),
    /// Auto-assign from palette. Vec is the resolved palette.
    Auto(Vec<Color>),
}

impl ColorMode {
    /// Build a palette slice for the canvas. Returns empty slice for Off.
    pub fn palette(&self) -> Vec<Color> {
        match self {
            ColorMode::Off => Vec::new(),
            ColorMode::Single(c) => vec![*c],
            ColorMode::Auto(p) => p.clone(),
        }
    }

    pub fn is_enabled(&self) -> bool {
        !matches!(self, ColorMode::Off)
    }

    /// Map series index to color index for the canvas.
    /// For Single mode, everything maps to 0.
    /// For Auto mode, wraps around the palette.
    pub fn series_color_idx(&self, series_idx: usize) -> Option<usize> {
        match self {
            ColorMode::Off => None,
            ColorMode::Single(_) => Some(0),
            ColorMode::Auto(p) => {
                if p.is_empty() {
                    None
                } else {
                    Some(series_idx % p.len())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_named_colors() {
        assert_eq!(parse_color("red").unwrap(), Color::Named(1));
        assert_eq!(parse_color("Green").unwrap(), Color::Named(2));
        assert_eq!(parse_color("CYAN").unwrap(), Color::Named(6));
        assert_eq!(parse_color("bright_red").unwrap(), Color::Named(9));
    }

    #[test]
    fn parse_indexed_colors() {
        assert_eq!(parse_color("0").unwrap(), Color::Indexed(0));
        assert_eq!(parse_color("196").unwrap(), Color::Indexed(196));
        assert_eq!(parse_color("255").unwrap(), Color::Indexed(255));
    }

    #[test]
    fn parse_out_of_range() {
        assert!(parse_color("256").is_err());
        assert!(parse_color("999").is_err());
    }

    #[test]
    fn parse_unknown() {
        assert!(parse_color("rainbow").is_err());
    }

    #[test]
    fn fg_code_named() {
        assert_eq!(Color::Named(1).fg_code(), "\x1b[31m");
        assert_eq!(Color::Named(0).fg_code(), "\x1b[30m");
        assert_eq!(Color::Named(9).fg_code(), "\x1b[91m");
    }

    #[test]
    fn fg_code_indexed() {
        assert_eq!(Color::Indexed(196).fg_code(), "\x1b[38;5;196m");
    }

    #[test]
    fn palette_has_entries() {
        assert!(PALETTE.len() >= 6);
    }
}
