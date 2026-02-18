use crate::color::{self, Color};

const BRAILLE_BASE: u32 = 0x2800;

// Braille dot bit positions for (row, col) within a 4×2 cell.
// Rows 0-2 map to dots 1-3 (left) and 4-6 (right).
// Row 3 maps to dots 7 (left) and 8 (right).
const BRAILLE_DOTS: [[u8; 2]; 4] = [[0x01, 0x08], [0x02, 0x10], [0x04, 0x20], [0x40, 0x80]];

pub struct BrailleCanvas {
    chars_wide: usize,
    chars_tall: usize,
    cells: Vec<u8>,
    /// Per-cell color index into an external palette. None = no color assigned.
    colors: Vec<Option<usize>>,
}

impl BrailleCanvas {
    pub fn new(chars_wide: usize, chars_tall: usize) -> Self {
        let n = chars_wide * chars_tall;
        Self {
            chars_wide,
            chars_tall,
            cells: vec![0u8; n],
            colors: vec![None; n],
        }
    }

    pub fn pixel_width(&self) -> usize {
        self.chars_wide * 2
    }

    pub fn pixel_height(&self) -> usize {
        self.chars_tall * 4
    }

    pub fn chars_wide(&self) -> usize {
        self.chars_wide
    }

    pub fn chars_tall(&self) -> usize {
        self.chars_tall
    }

    pub fn set(&mut self, px: usize, py: usize) {
        self.set_colored(px, py, None);
    }

    pub fn set_colored(&mut self, px: usize, py: usize, color_idx: Option<usize>) {
        let cx = px / 2;
        let cy = py / 4;
        if cx >= self.chars_wide || cy >= self.chars_tall {
            return;
        }
        let dot_col = px % 2;
        let dot_row = py % 4;
        let idx = cy * self.chars_wide + cx;
        self.cells[idx] |= BRAILLE_DOTS[dot_row][dot_col];
        if color_idx.is_some() {
            self.colors[idx] = color_idx;
        }
    }

    pub fn row_chars(&self, row: usize) -> String {
        let start = row * self.chars_wide;
        let end = start + self.chars_wide;
        self.cells[start..end]
            .iter()
            .map(|&bits| char::from_u32(BRAILLE_BASE + u32::from(bits)).unwrap_or(' '))
            .collect()
    }

    /// Render a row with ANSI color escapes. `palette` maps color indices to Colors.
    pub fn row_chars_colored(&self, row: usize, palette: &[Color]) -> String {
        let start = row * self.chars_wide;
        let end = start + self.chars_wide;
        let mut out = String::new();
        let mut current_color: Option<usize> = None;

        for i in start..end {
            let bits = self.cells[i];
            let ch = char::from_u32(BRAILLE_BASE + u32::from(bits)).unwrap_or(' ');
            let cell_color = self.colors[i];

            if cell_color != current_color {
                if current_color.is_some() {
                    out.push_str(color::RESET);
                }
                if let Some(ci) = cell_color
                    && ci < palette.len()
                {
                    out.push_str(&palette[ci].fg_code());
                }
                current_color = cell_color;
            }
            out.push(ch);
        }

        if current_color.is_some() {
            out.push_str(color::RESET);
        }

        out
    }

    pub fn line(&mut self, x0: usize, y0: usize, x1: usize, y1: usize) {
        self.line_colored(x0, y0, x1, y1, None);
    }

    pub fn line_colored(
        &mut self,
        x0: usize,
        y0: usize,
        x1: usize,
        y1: usize,
        color_idx: Option<usize>,
    ) {
        let dx = (x1 as isize - x0 as isize).abs();
        let dy = -(y1 as isize - y0 as isize).abs();
        let sx: isize = if x0 < x1 { 1 } else { -1 };
        let sy: isize = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let mut x = x0 as isize;
        let mut y = y0 as isize;

        loop {
            if x >= 0 && y >= 0 {
                self.set_colored(x as usize, y as usize, color_idx);
            }
            if x == x1 as isize && y == y1 as isize {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                if x == x1 as isize {
                    break;
                }
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                if y == y1 as isize {
                    break;
                }
                err += dx;
                y += sy;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_canvas_is_blank() {
        let c = BrailleCanvas::new(5, 3);
        assert_eq!(c.pixel_width(), 10);
        assert_eq!(c.pixel_height(), 12);
        for row in 0..3 {
            let s = c.row_chars(row);
            assert_eq!(s.chars().count(), 5);
            for ch in s.chars() {
                assert_eq!(ch, '\u{2800}');
            }
        }
    }

    #[test]
    fn set_top_left_pixel() {
        let mut c = BrailleCanvas::new(2, 2);
        c.set(0, 0);
        let row = c.row_chars(0);
        let first = row.chars().next().unwrap();
        assert_eq!(first, '\u{2801}');
    }

    #[test]
    fn set_bottom_right_of_cell() {
        let mut c = BrailleCanvas::new(1, 1);
        c.set(1, 3);
        let ch = c.row_chars(0).chars().next().unwrap();
        assert_eq!(ch, '\u{2880}');
    }

    #[test]
    fn set_multiple_dots_in_cell() {
        let mut c = BrailleCanvas::new(1, 1);
        c.set(0, 0); // 0x01
        c.set(1, 0); // 0x08
        c.set(0, 3); // 0x40
        c.set(1, 3); // 0x80
        let ch = c.row_chars(0).chars().next().unwrap();
        assert_eq!(ch as u32, BRAILLE_BASE + 0x01 + 0x08 + 0x40 + 0x80);
    }

    #[test]
    fn out_of_bounds_ignored() {
        let mut c = BrailleCanvas::new(1, 1);
        c.set(2, 0); // out of bounds x
        c.set(0, 4); // out of bounds y
        c.set(100, 100);
        let ch = c.row_chars(0).chars().next().unwrap();
        assert_eq!(ch, '\u{2800}');
    }

    #[test]
    fn horizontal_line() {
        let mut c = BrailleCanvas::new(5, 1);
        c.line(0, 2, 9, 2);
        let row = c.row_chars(0);
        for ch in row.chars() {
            assert_ne!(
                ch, '\u{2800}',
                "all cells along horizontal line should have dots"
            );
        }
    }

    #[test]
    fn vertical_line() {
        let mut c = BrailleCanvas::new(1, 3);
        c.line(0, 0, 0, 11);
        for row in 0..3 {
            let ch = c.row_chars(row).chars().next().unwrap();
            assert_ne!(ch, '\u{2800}');
        }
    }

    #[test]
    fn diagonal_line() {
        let mut c = BrailleCanvas::new(3, 3);
        c.line(0, 0, 5, 11);
        let mut any_set = false;
        for row in 0..3 {
            for ch in c.row_chars(row).chars() {
                if ch != '\u{2800}' {
                    any_set = true;
                }
            }
        }
        assert!(any_set);
    }
}
