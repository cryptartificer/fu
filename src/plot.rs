use crate::canvas::BrailleCanvas;
use crate::data::{BarData, DataSet};

pub fn render_lineplot(
    data: &DataSet,
    width: usize,
    height: usize,
    title: Option<&str>,
    xlabel: Option<&str>,
    ylabel: Option<&str>,
) -> String {
    let mut canvas = BrailleCanvas::new(width, height);
    let (x_min, x_max) = data.x_range();
    let (y_min, y_max) = data.y_range();

    let pw = canvas.pixel_width().saturating_sub(1).max(1) as f64;
    let ph = canvas.pixel_height().saturating_sub(1).max(1) as f64;
    let x_span = x_max - x_min;
    let y_span = y_max - y_min;

    for series in &data.series {
        let points: Vec<(usize, usize)> = data
            .x
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
            .collect();

        for pair in points.windows(2) {
            canvas.line(pair[0].0, pair[0].1, pair[1].0, pair[1].1);
        }
    }

    render_frame(&canvas, x_min, x_max, y_min, y_max, title, xlabel, ylabel)
}

fn format_val(v: f64) -> String {
    if v == v.trunc() && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        let s = format!("{:.2}", v);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

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
) -> String {
    let y_min_label = format_val(y_min);
    let y_max_label = format_val(y_max);
    let y_label_width = y_min_label.len().max(y_max_label.len());

    let cw = canvas.chars_wide();
    let ch = canvas.chars_tall();
    let margin = y_label_width + 1; // right-aligned label + 1 space

    let mut out = String::new();

    // Title
    if let Some(t) = title {
        let total_width = margin + 1 + cw + 1;
        let pad = total_width.saturating_sub(t.len()) / 2;
        out.push_str(&" ".repeat(pad));
        out.push_str(t);
        out.push('\n');
    }

    // Top border
    out.push_str(&" ".repeat(margin));
    out.push('┌');
    for _ in 0..cw {
        out.push('─');
    }
    out.push_str("┐ \n");

    // Canvas rows
    let mid_row = ch / 2;
    for row in 0..ch {
        let label = if row == 0 {
            format!("{:>w$}", y_max_label, w = y_label_width)
        } else if row == ch - 1 {
            format!("{:>w$}", y_min_label, w = y_label_width)
        } else if row == mid_row {
            if let Some(yl) = ylabel {
                if yl.len() <= y_label_width {
                    format!("{:>w$}", yl, w = y_label_width)
                } else {
                    " ".repeat(y_label_width)
                }
            } else {
                " ".repeat(y_label_width)
            }
        } else {
            " ".repeat(y_label_width)
        };

        out.push_str(&label);
        out.push_str(" │");
        out.push_str(&canvas.row_chars(row));
        out.push_str("│ \n");
    }

    // Bottom border
    out.push_str(&" ".repeat(margin));
    out.push('└');
    for _ in 0..cw {
        out.push('─');
    }
    out.push_str("┘ \n");

    // X-axis labels
    let x_min_label = format_val(x_min);
    let x_max_label = format_val(x_max);
    let x_label_area = cw + 2; // includes the two border corners

    let mut x_line = " ".repeat(margin);
    if x_min_label.len() + x_max_label.len() < x_label_area {
        let gap = x_label_area - x_min_label.len() - x_max_label.len();
        x_line.push_str(&x_min_label);
        x_line.push_str(&" ".repeat(gap));
        x_line.push_str(&x_max_label);
    } else {
        x_line.push_str(&x_min_label);
    }
    out.push_str(&x_line);
    out.push('\n');

    // xlabel
    if let Some(xl) = xlabel {
        let total_width = margin + 1 + cw + 1;
        let pad = total_width.saturating_sub(xl.len()) / 2;
        out.push_str(&" ".repeat(pad));
        out.push_str(xl);
        out.push('\n');
    }

    out
}

pub fn render_barplot(data: &BarData, width: usize, title: Option<&str>) -> String {
    let max_val = data
        .values
        .iter()
        .cloned()
        .reduce(f64::max)
        .unwrap_or(1.0)
        .max(f64::MIN_POSITIVE);

    let label_width = data.labels.iter().map(|l| l.len()).max().unwrap_or(0);
    let val_labels: Vec<String> = data.values.iter().map(|&v| format_val(v)).collect();
    let max_val_label_len = val_labels.iter().map(|l| l.len()).max().unwrap_or(0);

    // Bar area = total width - label - " ┤" - " " - val_label
    let bar_area = width
        .saturating_sub(label_width + 3 + max_val_label_len)
        .max(4);

    let mut out = String::new();

    // Title
    if let Some(t) = title {
        let total = label_width + 2 + bar_area + 1 + max_val_label_len;
        let pad = total.saturating_sub(t.len()) / 2;
        out.push_str(&" ".repeat(pad));
        out.push_str(t);
        out.push('\n');
    }

    // Top border
    out.push_str(&" ".repeat(label_width + 2));
    out.push('┌');
    out.push_str(&" ".repeat(bar_area));
    out.push_str("┐\n");

    // Bars
    for (i, (label, &val)) in data.labels.iter().zip(data.values.iter()).enumerate() {
        // Right-aligned label
        out.push_str(&format!("{:>w$}", label, w = label_width));
        out.push_str(" ┤");

        let bar_len = if max_val > 0.0 {
            ((val / max_val) * bar_area as f64).round() as usize
        } else {
            0
        };

        for _ in 0..bar_len {
            out.push('█');
        }

        let gap = bar_area.saturating_sub(bar_len);
        out.push_str(&" ".repeat(gap));
        out.push_str("┤ ");
        out.push_str(&val_labels[i]);
        out.push('\n');
    }

    // Bottom border
    out.push_str(&" ".repeat(label_width + 2));
    out.push('└');
    out.push_str(&" ".repeat(bar_area));
    out.push_str("┘\n");

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{BarData, DataSet};

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
        let output = render_lineplot(&data, 20, 8, Some("test"), None, None);
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
        let output = render_lineplot(&data, 10, 5, None, None, None);
        assert!(output.contains("┌"));
        assert!(!output.contains("test"));
    }

    #[test]
    fn render_basic_barplot() {
        let data = BarData {
            labels: vec!["cat".into(), "dog".into(), "parrot".into()],
            values: vec![30.0, 45.0, 12.0],
        };
        let output = render_barplot(&data, 40, Some("Animals"));
        assert!(output.contains("Animals"));
        assert!(output.contains("cat"));
        assert!(output.contains("dog"));
        assert!(output.contains("parrot"));
        assert!(output.contains("45"));
        assert!(output.contains("█"));
        assert!(output.contains("┤"));
    }

    #[test]
    fn barplot_max_bar_is_full() {
        let data = BarData {
            labels: vec!["a".into(), "b".into()],
            values: vec![100.0, 50.0],
        };
        let output = render_barplot(&data, 30, None);
        // "a" bar should be longer than "b" bar
        let lines: Vec<&str> = output.lines().collect();
        let a_blocks = lines[1].matches('█').count();
        let b_blocks = lines[2].matches('█').count();
        assert!(a_blocks > b_blocks);
    }

    #[test]
    fn render_contains_data_labels() {
        let data = DataSet {
            headers: None,
            x: vec![0.0, 100.0],
            series: vec![vec![0.0, 50.0]],
        };
        let output = render_lineplot(&data, 20, 5, None, None, None);
        assert!(output.contains("0"), "should contain x_min label");
        assert!(output.contains("100"), "should contain x_max label");
        assert!(output.contains("50"), "should contain y_max label");
    }
}
