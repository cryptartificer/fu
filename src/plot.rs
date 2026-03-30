use crate::canvas::BrailleCanvas;
use crate::cli::Sides;
use crate::color::ColorMode;
use crate::data::{BarData, DataSet};

pub struct PlotOptions<'a> {
    pub width: usize,
    pub height: usize,
    pub title: Option<&'a str>,
    pub xlabel: Option<&'a str>,
    pub ylabel: Option<&'a str>,
    pub color_mode: &'a ColorMode,
    pub grid: bool,
    pub xlim: Option<(f64, f64)>,
    pub ylim: Option<(f64, f64)>,
    pub margin: Sides,
    pub padding: Sides,
}

struct MappedData {
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    series_points: Vec<Vec<(usize, usize)>>,
}

/// Map data points to pixel coordinates on the canvas.
fn map_points(
    data: &DataSet,
    canvas: &BrailleCanvas,
    xlim: Option<(f64, f64)>,
    ylim: Option<(f64, f64)>,
) -> MappedData {
    let (x_min, x_max) = xlim.unwrap_or_else(|| nice_bounds_for(data.x_range()));
    let (y_min, y_max) = ylim.unwrap_or_else(|| nice_bounds_for(data.y_range()));

    let pw = canvas.pixel_width().saturating_sub(1).max(1) as f64;
    let ph = canvas.pixel_height().saturating_sub(1).max(1) as f64;
    let x_span = x_max - x_min;
    let y_span = y_max - y_min;

    let series_points: Vec<Vec<(usize, usize)>> = data
        .series
        .iter()
        .map(|series| {
            data.x
                .iter()
                .zip(series.iter())
                .map(|(&x, &y)| {
                    let px = if x_span > 0.0 {
                        ((x - x_min) / x_span * pw).round() as usize
                    } else {
                        (pw / 2.0) as usize
                    };
                    let py = if y_span > 0.0 {
                        ((y_max - y) / y_span * ph).round() as usize
                    } else {
                        (ph / 2.0) as usize
                    };
                    (px, py)
                })
                .collect()
        })
        .collect();

    MappedData {
        x_min,
        x_max,
        y_min,
        y_max,
        series_points,
    }
}

/// Round a positive value to a "nice" number (1, 2, 5, 10, 20, 50, ...).
fn nice_num(x: f64, round: bool) -> f64 {
    if x == 0.0 {
        return 0.0;
    }
    let exp = x.abs().log10().floor();
    let frac = x.abs() / 10f64.powf(exp);
    let nice = if round {
        if frac < 1.5 {
            1.0
        } else if frac < 3.0 {
            2.0
        } else if frac < 7.0 {
            5.0
        } else {
            10.0
        }
    } else if frac <= 1.0 {
        1.0
    } else if frac <= 2.0 {
        2.0
    } else if frac <= 5.0 {
        5.0
    } else {
        10.0
    };
    nice * 10f64.powf(exp)
}

/// Expand a data range to the tightest nice round bounds.
/// Uses a fine step (range/16) so the padding is minimal (~5% not ~15%).
fn nice_bounds_calc(lo: f64, hi: f64) -> (f64, f64) {
    if (hi - lo).abs() < f64::EPSILON {
        return (lo - 1.0, hi + 1.0);
    }
    let d = nice_num((hi - lo) / 16.0, true);
    if d == 0.0 {
        return (lo, hi);
    }
    let nice_lo = (lo / d).floor() * d;
    let nice_hi = (hi / d).ceil() * d;
    (nice_lo, nice_hi)
}

/// Apply nice bounds to a (min, max) tuple from data range.
fn nice_bounds_for(range: (f64, f64)) -> (f64, f64) {
    nice_bounds_calc(range.0, range.1)
}

/// Draw a continuous thin horizontal line at y=0 when the range crosses zero.
/// Sets every pixel to produce ⠤⠤⠤ style (matching uplot).
fn draw_zero_line(canvas: &mut BrailleCanvas, y_min: f64, y_max: f64) {
    if y_min >= 0.0 || y_max <= 0.0 {
        return;
    }
    let pw = canvas.pixel_width();
    let ph = canvas.pixel_height().saturating_sub(1).max(1) as f64;
    let y_span = y_max - y_min;
    if y_span <= 0.0 {
        return;
    }
    let py = ((y_max / y_span) * ph).round() as usize;
    for px in 0..pw {
        canvas.set(px, py);
    }
}

/// Draw dotted horizontal grid lines at regular y intervals.
fn draw_grid(canvas: &mut BrailleCanvas, n_lines: usize) {
    let pw = canvas.pixel_width();
    let ph = canvas.pixel_height();
    if n_lines == 0 || ph == 0 {
        return;
    }
    for i in 1..n_lines {
        let py = (i as f64 / n_lines as f64 * ph as f64).round() as usize;
        if py >= ph {
            continue;
        }
        // Dotted: every 4th pixel
        let mut px = 0;
        while px < pw {
            canvas.set(px, py);
            px += 4;
        }
    }
}

pub fn render_lineplot(data: &DataSet, opts: &PlotOptions) -> String {
    let mut canvas = BrailleCanvas::new(opts.width, opts.height);
    let m = map_points(data, &canvas, opts.xlim, opts.ylim);

    if opts.grid {
        draw_grid(&mut canvas, 4);
    }
    draw_zero_line(&mut canvas, m.y_min, m.y_max);

    for (si, points) in m.series_points.iter().enumerate() {
        let ci = opts.color_mode.series_color_idx(si);
        for pair in points.windows(2) {
            canvas.line_colored(pair[0].0, pair[0].1, pair[1].0, pair[1].1, ci);
        }
    }

    let legend = series_labels(data);
    render_frame(
        &canvas,
        m.x_min,
        m.x_max,
        m.y_min,
        m.y_max,
        opts.title,
        opts.xlabel,
        opts.ylabel,
        opts.color_mode,
        &legend,
        &opts.margin,
        &opts.padding,
    )
}

pub fn render_scatter(data: &DataSet, opts: &PlotOptions) -> String {
    let mut canvas = BrailleCanvas::new(opts.width, opts.height);
    let m = map_points(data, &canvas, opts.xlim, opts.ylim);

    if opts.grid {
        draw_grid(&mut canvas, 4);
    }
    draw_zero_line(&mut canvas, m.y_min, m.y_max);

    for (si, points) in m.series_points.iter().enumerate() {
        let ci = opts.color_mode.series_color_idx(si);
        for &(px, py) in points {
            canvas.set_colored(px, py, ci);
        }
    }

    let legend = series_labels(data);
    render_frame(
        &canvas,
        m.x_min,
        m.x_max,
        m.y_min,
        m.y_max,
        opts.title,
        opts.xlabel,
        opts.ylabel,
        opts.color_mode,
        &legend,
        &opts.margin,
        &opts.padding,
    )
}

/// Extract series labels from dataset headers (y-column names).
fn series_labels(data: &DataSet) -> Vec<String> {
    match &data.headers {
        Some(h) if h.len() > 1 => h[1..].to_vec(),
        _ => Vec::new(),
    }
}

fn format_val(v: f64) -> String {
    let v = if v.abs() < 1e-10 { 0.0 } else { v };
    if v == v.trunc() && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        let s = format!("{:.2}", v);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

/// Generate evenly-spaced nice-valued ticks within [lo, hi], mapped to grid
/// positions. Returns (value, grid_position) pairs. Only multiples of the
/// computed step are emitted — no forced bounds — so spacing is always even.
fn axis_ticks(lo: f64, hi: f64, n_cells: usize, n_target: usize) -> Vec<(f64, usize)> {
    let range = hi - lo;
    let last = n_cells.saturating_sub(1).max(1);
    if range <= 0.0 || n_target < 1 {
        return vec![(lo, 0)];
    }
    let step = nice_num(range / n_target as f64, true);
    if step <= 0.0 {
        return vec![(lo, 0)];
    }
    let n1 = last as f64;
    let mut ticks = Vec::new();

    let first = (lo / step).ceil() * step;
    let last_tick = (hi / step).floor() * step;

    let mut v = first;
    while v <= last_tick + step * 0.001 {
        let pos = ((v - lo) / range * n1).round() as usize;
        ticks.push((v, pos));
        v += step;
    }

    if ticks.is_empty() {
        ticks.push((lo, 0));
    }
    ticks
}

/// ANSI dark gray for borders/labels (matches uplot \033[90m).
const DIM: &str = "\x1b[90m";
const DIM_RESET: &str = "\x1b[39m";

#[allow(clippy::too_many_arguments)]
fn render_frame(
    canvas: &BrailleCanvas,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    title: Option<&str>,
    xlabel: Option<&str>,
    ylabel: Option<&str>,
    color_mode: &ColorMode,
    legend: &[String],
    margin: &Sides,
    padding: &Sides,
) -> String {
    let cw = canvas.chars_wide();
    let ch = canvas.chars_tall();
    let use_color = color_mode.is_enabled();
    let palette = color_mode.palette();

    // Y-axis: sparse nice ticks (match uplot: just bounds + key values like zero)
    let n_y_target = (ch / 6).clamp(2, 5);
    let y_ticks = axis_ticks(y_min, y_max, ch, n_y_target);
    let y_label_width = y_ticks
        .iter()
        .map(|&(v, _)| format_val(v).len())
        .max()
        .unwrap_or(2);

    // Gutter = margin.left + max_label_width + 1 (space before │)
    let label_field = margin.left + y_label_width;
    let gutter = label_field + 1;
    let inner_width = padding.left + cw + padding.right;

    // Right-side legend: build legend entries for rows to the right of │
    let has_legend = use_color && !legend.is_empty();
    let legend_width = if has_legend {
        legend.iter().map(|l| l.len()).max().unwrap_or(0) + 1
    } else {
        0
    };
    let right_trail = legend_width.max(1) + margin.right;

    let left_pad_str = " ".repeat(padding.left);
    let right_pad_str = " ".repeat(padding.right);

    let mut out = String::new();

    // margin.top
    for _ in 0..margin.top {
        out.push('\n');
    }

    // Title
    if let Some(t) = title {
        let total_width = gutter + 1 + inner_width + 1 + right_trail;
        let pad = total_width.saturating_sub(t.len()) / 2;
        out.push_str(&" ".repeat(pad));
        out.push_str(t);
        out.push('\n');
    }

    // Helper: emit an empty row inside the border (for padding.top / padding.bottom)
    let emit_empty_inner_row = |out: &mut String| {
        out.push_str(&" ".repeat(gutter));
        if use_color {
            out.push_str(DIM);
        }
        out.push('│');
        if use_color {
            out.push_str(DIM_RESET);
        }
        out.push_str(&" ".repeat(inner_width));
        if use_color {
            out.push_str(DIM);
        }
        out.push('│');
        if use_color {
            out.push_str(DIM_RESET);
        }
        out.push_str(&" ".repeat(right_trail));
        out.push('\n');
    };

    // Top border
    out.push_str(&" ".repeat(gutter));
    if use_color {
        out.push_str(DIM);
    }
    out.push('┌');
    out.push_str(&"─".repeat(inner_width));
    out.push('┐');
    if use_color {
        out.push_str(DIM_RESET);
    }
    out.push_str(&" ".repeat(right_trail));
    out.push('\n');

    // padding.top — empty bordered rows
    for _ in 0..padding.top {
        emit_empty_inner_row(&mut out);
    }

    // Canvas rows
    let mid_row = ch / 2;
    for row in 0..ch {
        let last_y = ch.saturating_sub(1).max(1);
        let tick_label = y_ticks
            .iter()
            .find(|&&(_, pos)| last_y - pos == row)
            .map(|&(v, _)| format_val(v));

        let label = if let Some(ref tl) = tick_label {
            format!("{:>w$}", tl, w = label_field)
        } else if row == mid_row && tick_label.is_none() {
            if let Some(yl) = ylabel {
                if yl.len() <= label_field {
                    format!("{:>w$}", yl, w = label_field)
                } else {
                    " ".repeat(label_field)
                }
            } else {
                " ".repeat(label_field)
            }
        } else {
            " ".repeat(label_field)
        };

        if use_color {
            out.push_str(DIM);
            out.push_str(&label);
            out.push_str(" │");
            out.push_str(DIM_RESET);
        } else {
            out.push_str(&label);
            out.push_str(" │");
        }
        out.push_str(&left_pad_str);
        if use_color {
            out.push_str(&canvas.row_chars_colored(row, &palette));
        } else {
            out.push_str(&canvas.row_chars(row));
        }
        out.push_str(&right_pad_str);
        if use_color {
            out.push_str(DIM);
        }
        out.push('│');
        if use_color {
            out.push_str(DIM_RESET);
        }

        // Right-side legend label (uplot style) + margin.right
        if has_legend && row < legend.len() {
            out.push(' ');
            if let Some(ci) = color_mode.series_color_idx(row)
                && ci < palette.len()
            {
                out.push_str(&palette[ci].fg_code());
            }
            out.push_str(&legend[row]);
            if color_mode.series_color_idx(row).is_some() {
                out.push_str(crate::color::RESET);
            }
            let pad_needed = right_trail.saturating_sub(1 + legend[row].len());
            if pad_needed > 0 {
                out.push_str(&" ".repeat(pad_needed));
            }
        } else {
            out.push_str(&" ".repeat(right_trail));
        }
        out.push('\n');
    }

    // padding.bottom — empty bordered rows
    for _ in 0..padding.bottom {
        emit_empty_inner_row(&mut out);
    }

    // Bottom border
    out.push_str(&" ".repeat(gutter));
    if use_color {
        out.push_str(DIM);
    }
    out.push('└');
    out.push_str(&"─".repeat(inner_width));
    out.push('┘');
    if use_color {
        out.push_str(DIM_RESET);
    }
    out.push_str(&" ".repeat(right_trail));
    out.push('\n');

    // X-axis: just min and max at the edges (like uplot)
    let x_area = inner_width + 2;
    let x_total = gutter + x_area;
    let mut x_buf = vec![b' '; x_total];

    let lo_label = format_val(x_min);
    let hi_label = format_val(x_max);
    let lo_end = (gutter + lo_label.len()).min(x_total);
    x_buf[gutter..lo_end].copy_from_slice(&lo_label.as_bytes()[..lo_end - gutter]);
    let hi_start = x_total.saturating_sub(hi_label.len());
    x_buf[hi_start..x_total].copy_from_slice(hi_label.as_bytes());
    let x_line_str: String = x_buf.iter().map(|&b| b as char).collect();
    let x_line_trimmed = x_line_str.trim_end();

    let mut x_line = String::new();
    if use_color {
        x_line.push_str(DIM);
    }
    x_line.push_str(x_line_trimmed);
    if use_color {
        x_line.push_str(DIM_RESET);
    }
    out.push_str(&x_line);
    out.push('\n');

    // xlabel
    if let Some(xl) = xlabel {
        let total_width = gutter + 1 + inner_width + 1;
        let pad = total_width.saturating_sub(xl.len()) / 2;
        if use_color {
            out.push_str(crate::color::RESET);
        }
        out.push_str(&" ".repeat(pad));
        out.push_str(xl);
        out.push('\n');
    }

    // margin.bottom
    for _ in 0..margin.bottom {
        out.push('\n');
    }

    out
}

pub fn render_barplot(
    data: &BarData,
    width: usize,
    title: Option<&str>,
    color_mode: &ColorMode,
    margin: &Sides,
    padding: &Sides,
    symbol: char,
) -> String {
    let max_val = data
        .values
        .iter()
        .cloned()
        .reduce(f64::max)
        .unwrap_or(1.0)
        .max(f64::MIN_POSITIVE);

    let use_color = color_mode.is_enabled();
    let palette = color_mode.palette();
    let max_label_len = data.labels.iter().map(|l| l.len()).max().unwrap_or(0);
    let label_field = margin.left + max_label_len;
    let gutter = label_field + 2; // label + " ┤"
    let val_labels: Vec<String> = data.values.iter().map(|&v| format_val(v)).collect();
    let max_val_label_len = val_labels.iter().map(|l| l.len()).max().unwrap_or(0);

    let bar_area = width
        .saturating_sub(1 + max_val_label_len + padding.left + padding.right)
        .max(4);
    let inner_width = padding.left + bar_area + 1 + max_val_label_len + padding.right;

    let right_trail = margin.right.max(1);

    let mut out = String::new();

    // margin.top
    for _ in 0..margin.top {
        out.push('\n');
    }

    // Title
    if let Some(t) = title {
        let total = gutter + 1 + inner_width + 1 + right_trail;
        let pad = total.saturating_sub(t.len()) / 2;
        out.push_str(&" ".repeat(pad));
        out.push_str(t);
        out.push('\n');
    }

    // Helper: emit an empty row inside the border (for padding.top / padding.bottom)
    let emit_empty_inner_row = |out: &mut String| {
        out.push_str(&" ".repeat(gutter - 1));
        if use_color {
            out.push_str(DIM);
        }
        out.push('│');
        if use_color {
            out.push_str(DIM_RESET);
        }
        out.push_str(&" ".repeat(inner_width + 1));
        if use_color {
            out.push_str(DIM);
        }
        out.push('│');
        if use_color {
            out.push_str(DIM_RESET);
        }
        out.push_str(&" ".repeat(right_trail));
        out.push('\n');
    };

    // Top border (align ┌ with ┤ on data rows)
    out.push_str(&" ".repeat(gutter - 1));
    if use_color {
        out.push_str(DIM);
    }
    out.push('┌');
    out.push_str(&" ".repeat(inner_width + 1));
    out.push('┐');
    if use_color {
        out.push_str(DIM_RESET);
    }
    out.push_str(&" ".repeat(right_trail));
    out.push('\n');

    // padding.top
    for _ in 0..padding.top {
        emit_empty_inner_row(&mut out);
    }

    // Bars
    let bar_color = if let Some(ci) = color_mode.series_color_idx(0) {
        palette.get(ci).map(|c| c.fg_code())
    } else {
        None
    };
    let left_pad_str = " ".repeat(padding.left);

    for (i, (label, &val)) in data.labels.iter().zip(data.values.iter()).enumerate() {
        if use_color {
            out.push_str(crate::color::RESET);
        }
        out.push_str(&format!("{:>w$}", label, w = label_field));
        if use_color {
            out.push_str(DIM);
        }
        out.push_str(" ┤");
        if use_color {
            out.push_str(DIM_RESET);
        }
        out.push_str(&left_pad_str);

        let bar_len = if max_val > 0.0 {
            ((val / max_val) * bar_area as f64).round() as usize
        } else {
            0
        };

        if let Some(ref fc) = bar_color {
            out.push_str(fc);
        }
        out.push_str(&symbol.to_string().repeat(bar_len));
        if bar_color.is_some() {
            out.push_str(DIM_RESET);
        }

        if use_color {
            out.push_str(crate::color::RESET);
        }
        out.push(' ');
        out.push_str(&val_labels[i]);

        let used = padding.left + bar_len + 1 + val_labels[i].len() + padding.right;
        if used < inner_width {
            out.push_str(&" ".repeat(inner_width - used));
        }
        if use_color {
            out.push_str(DIM);
            out.push_str(&" ".repeat(right_trail));
            out.push_str(DIM_RESET);
        }
        out.push('\n');
    }

    // padding.bottom
    for _ in 0..padding.bottom {
        emit_empty_inner_row(&mut out);
    }

    // Bottom border (align └ with ┤ on data rows)
    out.push_str(&" ".repeat(gutter - 1));
    if use_color {
        out.push_str(DIM);
    }
    out.push('└');
    out.push_str(&" ".repeat(inner_width + 1));
    out.push('┘');
    if use_color {
        out.push_str(DIM_RESET);
    }
    out.push_str(&" ".repeat(right_trail));
    out.push('\n');

    // margin.bottom
    for _ in 0..margin.bottom {
        out.push('\n');
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::Sides;
    use crate::color::{Color, ColorMode};
    use crate::data::{BarData, DataSet};

    const OFF: ColorMode = ColorMode::Off;

    fn opts(w: usize, h: usize) -> PlotOptions<'static> {
        PlotOptions {
            width: w,
            height: h,
            title: None,
            xlabel: None,
            ylabel: None,
            color_mode: &OFF,
            grid: false,
            xlim: None,
            ylim: None,
            margin: Sides::all(0),
            padding: Sides::all(0),
        }
    }

    #[test]
    fn format_val_integer() {
        assert_eq!(format_val(10.0), "10");
        assert_eq!(format_val(-5.0), "-5");
        assert_eq!(format_val(0.0), "0");
    }

    #[test]
    fn format_val_float() {
        assert_eq!(format_val(3.14), "3.14");
        assert_eq!(format_val(2.50), "2.5");
        assert_eq!(format_val(1.10), "1.1");
    }

    #[test]
    fn render_basic_lineplot() {
        let data = DataSet {
            headers: None,
            x: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            series: vec![vec![10.0, 20.0, 15.0, 30.0, 25.0]],
        };
        let mut o = opts(20, 8);
        o.title = Some("test");
        let output = render_lineplot(&data, &o);
        assert!(output.contains("test"));
        assert!(output.contains("┌"));
        assert!(output.contains("┘"));
        assert!(output.contains("│"));
        assert!(output.contains("10"));
        assert!(output.contains("30"));
    }

    #[test]
    fn render_without_title() {
        let data = DataSet {
            headers: None,
            x: vec![1.0, 2.0],
            series: vec![vec![5.0, 10.0]],
        };
        let output = render_lineplot(&data, &opts(10, 5));
        assert!(output.contains("┌"));
        assert!(!output.contains("test"));
    }

    #[test]
    fn render_basic_barplot() {
        let data = BarData {
            labels: vec!["cat".into(), "dog".into(), "parrot".into()],
            values: vec![30.0, 45.0, 12.0],
        };
        let output = render_barplot(
            &data,
            40,
            Some("Animals"),
            &OFF,
            &Sides::all(0),
            &Sides::all(0),
            '■',
        );
        assert!(output.contains("Animals"));
        assert!(output.contains("cat"));
        assert!(output.contains("dog"));
        assert!(output.contains("parrot"));
        assert!(output.contains("45"));
        assert!(output.contains('■'));
        assert!(output.contains("┤"));
    }

    #[test]
    fn barplot_max_bar_is_full() {
        let data = BarData {
            labels: vec!["a".into(), "b".into()],
            values: vec![100.0, 50.0],
        };
        let output = render_barplot(&data, 30, None, &OFF, &Sides::all(0), &Sides::all(0), '■');
        let lines: Vec<&str> = output.lines().collect();
        let a_blocks = lines[1].matches('■').count();
        let b_blocks = lines[2].matches('■').count();
        assert!(a_blocks > b_blocks);
    }

    #[test]
    fn render_basic_scatter() {
        let data = DataSet {
            headers: None,
            x: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            series: vec![vec![10.0, 20.0, 15.0, 30.0, 25.0]],
        };
        let mut o = opts(20, 8);
        o.title = Some("dots");
        let output = render_scatter(&data, &o);
        assert!(output.contains("dots"));
        assert!(output.contains("┌"));
        assert!(output.contains("┘"));
        assert!(output.contains("10"));
        assert!(output.contains("30"));
    }

    #[test]
    fn scatter_fewer_dots_than_lineplot() {
        let data = DataSet {
            headers: None,
            x: vec![1.0, 5.0, 10.0],
            series: vec![vec![0.0, 50.0, 100.0]],
        };
        let o = opts(20, 8);
        let line_out = render_lineplot(&data, &o);
        let scat_out = render_scatter(&data, &o);
        let line_braille: usize = line_out
            .chars()
            .filter(|&c| c as u32 > 0x2800 && c as u32 <= 0x28FF)
            .count();
        let scat_braille: usize = scat_out
            .chars()
            .filter(|&c| c as u32 > 0x2800 && c as u32 <= 0x28FF)
            .count();
        assert!(
            scat_braille <= line_braille,
            "scatter should set fewer dots than line"
        );
    }

    #[test]
    fn render_contains_data_labels() {
        let data = DataSet {
            headers: None,
            x: vec![0.0, 100.0],
            series: vec![vec![0.0, 50.0]],
        };
        let output = render_lineplot(&data, &opts(20, 5));
        assert!(output.contains("0"), "should contain x_min label");
        assert!(output.contains("100"), "should contain x_max label");
        // Y-axis uses nice ticks (step=20 for range 0-50): 0, 20, 40
        assert!(output.contains("20"), "should contain y nice tick");
    }

    #[test]
    fn render_lineplot_with_ylim() {
        let data = DataSet {
            headers: None,
            x: vec![1.0, 2.0, 3.0],
            series: vec![vec![10.0, 20.0, 15.0]],
        };
        let mut o = opts(20, 8);
        o.ylim = Some((0.0, 50.0));
        let output = render_lineplot(&data, &o);
        assert!(output.contains("0"), "should show ylim min");
        // Ticks are nice multiples (step=20): 0, 20, 40
        assert!(output.contains("20"), "should show nice tick within ylim");
    }

    #[test]
    fn render_lineplot_with_xlim() {
        let data = DataSet {
            headers: None,
            x: vec![1.0, 2.0, 3.0],
            series: vec![vec![10.0, 20.0, 15.0]],
        };
        let mut o = opts(20, 8);
        o.xlim = Some((0.0, 10.0));
        let output = render_lineplot(&data, &o);
        assert!(output.contains("10"), "should show xlim max");
    }

    #[test]
    fn render_lineplot_with_grid() {
        let data = DataSet {
            headers: None,
            x: vec![1.0, 2.0, 3.0],
            series: vec![vec![0.0, 0.0, 0.0]],
        };
        let no_grid = render_lineplot(&data, &opts(20, 8));
        let mut o = opts(20, 8);
        o.grid = true;
        let with_grid = render_lineplot(&data, &o);
        let no_grid_dots: usize = no_grid
            .chars()
            .filter(|&c| c as u32 > 0x2800 && c as u32 <= 0x28FF)
            .count();
        let grid_dots: usize = with_grid
            .chars()
            .filter(|&c| c as u32 > 0x2800 && c as u32 <= 0x28FF)
            .count();
        assert!(
            grid_dots > no_grid_dots,
            "grid should add extra dots to the canvas"
        );
    }

    #[test]
    fn render_lineplot_with_color() {
        let data = DataSet {
            headers: None,
            x: vec![1.0, 2.0, 3.0],
            series: vec![vec![10.0, 20.0, 15.0]],
        };
        let cm = ColorMode::Single(Color::Named(1));
        let mut o = opts(20, 8);
        o.color_mode = &cm;
        let output = render_lineplot(&data, &o);
        assert!(output.contains("\x1b[31m"), "should contain red ANSI code");
        assert!(output.contains("\x1b[0m"), "should contain reset code");
    }

    #[test]
    fn render_lineplot_multi_series_auto_color() {
        let data = DataSet {
            headers: None,
            x: vec![1.0, 2.0, 3.0],
            series: vec![vec![10.0, 20.0, 15.0], vec![5.0, 15.0, 25.0]],
        };
        let cm = ColorMode::Auto(vec![Color::Named(1), Color::Named(2)]);
        let mut o = opts(20, 8);
        o.color_mode = &cm;
        let output = render_lineplot(&data, &o);
        assert!(output.contains("\x1b[31m"), "should contain red");
        assert!(output.contains("\x1b[32m"), "should contain green");
    }

    #[test]
    fn render_legend_from_headers() {
        let data = DataSet {
            headers: Some(vec!["x".into(), "temp".into(), "pressure".into()]),
            x: vec![1.0, 2.0, 3.0],
            series: vec![vec![10.0, 20.0, 15.0], vec![5.0, 15.0, 25.0]],
        };
        let cm = ColorMode::Auto(vec![Color::Named(1), Color::Named(2)]);
        let mut o = opts(30, 8);
        o.color_mode = &cm;
        let output = render_lineplot(&data, &o);
        assert!(
            output.contains("temp"),
            "legend should contain series name 'temp'"
        );
        assert!(
            output.contains("pressure"),
            "legend should contain series name 'pressure'"
        );
    }

    #[test]
    fn no_legend_without_headers() {
        let data = DataSet {
            headers: None,
            x: vec![1.0, 2.0, 3.0],
            series: vec![vec![10.0, 20.0, 15.0], vec![5.0, 15.0, 25.0]],
        };
        let output = render_lineplot(&data, &opts(30, 8));
        assert!(!output.contains("temp"), "no legend without headers");
    }
}
