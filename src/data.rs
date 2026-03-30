use std::fs;
use std::io::{self, BufRead};

use crate::cli::Options;

#[derive(Debug)]
pub struct DataSet {
    pub headers: Option<Vec<String>>,
    pub x: Vec<f64>,
    pub series: Vec<Vec<f64>>,
}

#[derive(Debug)]
pub struct BarData {
    pub labels: Vec<String>,
    pub values: Vec<f64>,
}

impl DataSet {
    pub fn x_range(&self) -> (f64, f64) {
        let min = self.x.iter().cloned().reduce(f64::min).unwrap_or(0.0);
        let max = self.x.iter().cloned().reduce(f64::max).unwrap_or(1.0);
        if (max - min).abs() < f64::EPSILON {
            (min - 1.0, max + 1.0)
        } else {
            (min, max)
        }
    }

    pub fn y_range(&self) -> (f64, f64) {
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        for s in &self.series {
            for &v in s {
                if v < min {
                    min = v;
                }
                if v > max {
                    max = v;
                }
            }
        }
        if min == f64::INFINITY {
            return (0.0, 1.0);
        }
        if (max - min).abs() < f64::EPSILON {
            (min - 1.0, max + 1.0)
        } else {
            (min, max)
        }
    }
}

pub fn read_input(opts: &Options) -> Result<DataSet, String> {
    let lines = read_lines(opts)?;
    let lines = if opts.transpose {
        transpose_lines(&lines, opts.delimiter)
    } else {
        lines
    };
    parse_lines(&lines, opts.delimiter, opts.has_headers)
}

fn transpose_lines(lines: &[String], delimiter: char) -> Vec<String> {
    let grid: Vec<Vec<&str>> = lines
        .iter()
        .filter(|l| !l.is_empty())
        .map(|l| l.split(delimiter).map(|s| s.trim()).collect())
        .collect();

    if grid.is_empty() {
        return Vec::new();
    }

    let max_cols = grid.iter().map(|r| r.len()).max().unwrap_or(0);
    let mut transposed = Vec::with_capacity(max_cols);
    let sep = delimiter.to_string();

    for col in 0..max_cols {
        let row: Vec<&str> = grid
            .iter()
            .map(|r| if col < r.len() { r[col] } else { "" })
            .collect();
        transposed.push(row.join(&sep));
    }

    transposed
}

pub fn read_bar_input(opts: &Options) -> Result<BarData, String> {
    let lines = read_lines(opts)?;
    parse_bar_lines(&lines, opts.delimiter, opts.has_headers)
}

fn parse_bar_lines(
    lines: &[String],
    delimiter: char,
    has_headers: bool,
) -> Result<BarData, String> {
    let mut iter = lines.iter().filter(|l| !l.is_empty());

    if has_headers {
        iter.next().ok_or("no data (expected header row)")?;
    }

    let mut labels = Vec::new();
    let mut values = Vec::new();

    for line in iter {
        let parts: Vec<&str> = line.split(delimiter).map(|s| s.trim()).collect();
        match parts.len() {
            1 => {
                // Single column: try as value, label is row index
                let v: f64 = parts[0]
                    .parse()
                    .map_err(|_| format!("cannot parse as number: {:?}", parts[0]))?;
                labels.push((values.len() + 1).to_string());
                values.push(v);
            }
            _ => {
                // First column is label, second is value
                let v: f64 = parts[1]
                    .parse()
                    .map_err(|_| format!("cannot parse as number: {:?}", parts[1]))?;
                labels.push(parts[0].to_string());
                values.push(v);
            }
        }
    }

    if values.is_empty() {
        return Err("no data".to_string());
    }

    Ok(BarData { labels, values })
}

pub fn read_hist_input(opts: &Options) -> Result<Vec<f64>, String> {
    let lines = read_lines(opts)?;
    parse_hist_lines(&lines, opts.delimiter, opts.has_headers)
}

fn parse_hist_lines(
    lines: &[String],
    delimiter: char,
    has_headers: bool,
) -> Result<Vec<f64>, String> {
    let mut iter = lines.iter().filter(|l| !l.is_empty());

    if has_headers {
        iter.next().ok_or("no data (expected header row)")?;
    }

    let mut values = Vec::new();
    for line in iter {
        let field = line.split(delimiter).next().unwrap_or("").trim();
        let v: f64 = field
            .parse()
            .map_err(|_| format!("cannot parse as number: {field:?}"))?;
        values.push(v);
    }

    if values.is_empty() {
        return Err("no data".to_string());
    }

    Ok(values)
}

pub fn bin_values(values: &[f64], nbins: usize) -> BarData {
    let raw_min = values.iter().cloned().reduce(f64::min).unwrap();
    let raw_max = values.iter().cloned().reduce(f64::max).unwrap();
    let range = raw_max - raw_min;

    if range <= 0.0 {
        let edges = vec![(format_compact(raw_min), format_compact(raw_min + 1.0))];
        let labels = format_bin_labels(&edges);
        let values = vec![values.len() as f64];
        return BarData { labels, values };
    }

    // Pick the nicest bin width that best preserves the requested count.
    let bin_width = okay_bin_width(range / nbins as f64);
    let nice_lo = (raw_min / bin_width).floor() * bin_width;
    let nice_hi = (raw_max / bin_width).ceil() * bin_width;
    let actual_bins = ((nice_hi - nice_lo) / bin_width).round() as usize;

    let mut counts = vec![0u64; actual_bins];
    for &v in values {
        let mut idx = ((v - nice_lo) / bin_width) as usize;
        if idx >= actual_bins {
            idx = actual_bins - 1;
        }
        counts[idx] += 1;
    }

    let edges: Vec<(String, String)> = (0..actual_bins)
        .map(|i| {
            let lo = nice_lo + i as f64 * bin_width;
            let hi = lo + bin_width;
            (format_compact(lo), format_compact(hi))
        })
        .collect();

    let labels = format_bin_labels(&edges);
    let values: Vec<f64> = counts.iter().map(|&c| c as f64).collect();

    BarData { labels, values }
}

pub fn filter_values(mut values: Vec<f64>, gt: Option<f64>, lt: Option<f64>) -> Vec<f64> {
    if let Some(lo) = gt {
        values.retain(|&v| v > lo);
    }
    if let Some(hi) = lt {
        values.retain(|&v| v < hi);
    }
    values
}

pub fn bin_values_log(values: &[f64], nbins: usize) -> Result<BarData, String> {
    for &v in values {
        if v <= 0.0 {
            return Err(format!("--log requires all values > 0 (found {v})"));
        }
    }

    let log_vals: Vec<f64> = values.iter().map(|v| v.log10()).collect();
    let raw_min = log_vals.iter().cloned().reduce(f64::min).unwrap();
    let raw_max = log_vals.iter().cloned().reduce(f64::max).unwrap();
    let range = raw_max - raw_min;

    if range <= 0.0 {
        let v = values[0];
        let edges = vec![(format_compact(v), format_compact(v * 10.0))];
        let labels = format_bin_labels(&edges);
        let values = vec![values.len() as f64];
        return Ok(BarData { labels, values });
    }

    let bin_width = okay_bin_width(range / nbins as f64);
    let nice_lo = (raw_min / bin_width).floor() * bin_width;
    let nice_hi = (raw_max / bin_width).ceil() * bin_width;
    let actual_bins = ((nice_hi - nice_lo) / bin_width).round() as usize;

    let mut counts = vec![0u64; actual_bins];
    for &lv in &log_vals {
        let mut idx = ((lv - nice_lo) / bin_width) as usize;
        if idx >= actual_bins {
            idx = actual_bins - 1;
        }
        counts[idx] += 1;
    }

    let edges: Vec<(String, String)> = (0..actual_bins)
        .map(|i| {
            let lo = 10f64.powf(nice_lo + i as f64 * bin_width);
            let hi = 10f64.powf(nice_lo + (i + 1) as f64 * bin_width);
            (format_compact(lo), format_compact(hi))
        })
        .collect();

    let labels = format_bin_labels(&edges);
    let values: Vec<f64> = counts.iter().map(|&c| c as f64).collect();

    Ok(BarData { labels, values })
}

/// Format bin labels with right-justified numbers inside brackets (matching uplot).
fn format_bin_labels(edges: &[(String, String)]) -> Vec<String> {
    let max_w = edges
        .iter()
        .flat_map(|(lo, hi)| [lo.len(), hi.len()])
        .max()
        .unwrap_or(0);
    edges
        .iter()
        .map(|(lo, hi)| format!("[{:>w$}, {:>w$})", lo, hi, w = max_w))
        .collect()
}

/// Round a bin width to a reasonably round number.
/// Denser than classic 1/2/5: snaps to {1, 2, 2.5, 3, 4, 5, 8} × 10^k
/// so the resulting bin count stays close to what was requested.
fn okay_bin_width(raw: f64) -> f64 {
    if raw <= 0.0 {
        return 1.0;
    }
    let exp = raw.log10().floor();
    let frac = raw / 10f64.powf(exp);
    let nice = if frac < 1.5 {
        1.0
    } else if frac < 2.25 {
        2.0
    } else if frac < 2.75 {
        2.5
    } else if frac < 3.5 {
        3.0
    } else if frac < 4.5 {
        4.0
    } else if frac < 6.5 {
        5.0
    } else if frac < 9.0 {
        8.0
    } else {
        10.0
    };
    nice * 10f64.powf(exp)
}

fn format_compact(v: f64) -> String {
    let v = if v.abs() < 1e-10 { 0.0 } else { v };
    // Always one decimal place for consistent label width (matches uplot)
    format!("{:.1}", v)
}

pub fn read_count_input(opts: &Options) -> Result<BarData, String> {
    let lines = read_lines(opts)?;
    parse_count_lines(&lines, opts.delimiter, opts.has_headers)
}

fn parse_count_lines(
    lines: &[String],
    delimiter: char,
    has_headers: bool,
) -> Result<BarData, String> {
    let mut iter = lines.iter().filter(|l| !l.is_empty());

    if has_headers {
        iter.next().ok_or("no data (expected header row)")?;
    }

    let mut counts: Vec<(String, u64)> = Vec::new();
    for line in iter {
        let field = line
            .split(delimiter)
            .next()
            .unwrap_or("")
            .trim()
            .to_string();
        if let Some(entry) = counts.iter_mut().find(|(k, _)| k == &field) {
            entry.1 += 1;
        } else {
            counts.push((field, 1));
        }
    }

    if counts.is_empty() {
        return Err("no data".to_string());
    }

    // Sort by count descending, then alphabetically for ties
    counts.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    let labels = counts.iter().map(|(k, _)| k.clone()).collect();
    let values = counts.iter().map(|(_, v)| *v as f64).collect();

    Ok(BarData { labels, values })
}

fn read_lines(opts: &Options) -> Result<Vec<String>, String> {
    if opts.files.is_empty() {
        let stdin = io::stdin();
        let lines: Vec<String> = stdin
            .lock()
            .lines()
            .map(|l| l.map_err(|e| e.to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(lines)
    } else {
        let mut all_lines = Vec::new();
        for path in &opts.files {
            let content = fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
            all_lines.extend(content.lines().map(String::from));
        }
        Ok(all_lines)
    }
}

fn parse_lines(lines: &[String], delimiter: char, has_headers: bool) -> Result<DataSet, String> {
    let mut iter = lines.iter().filter(|l| !l.is_empty());

    let headers = if has_headers {
        let header_line = iter.next().ok_or("no data (expected header row)")?;
        Some(
            header_line
                .split(delimiter)
                .map(|s| s.trim().to_string())
                .collect(),
        )
    } else {
        None
    };

    let mut rows: Vec<Vec<f64>> = Vec::new();
    for line in iter {
        let fields: Vec<f64> = line
            .split(delimiter)
            .map(|s| {
                s.trim()
                    .parse::<f64>()
                    .map_err(|_| format!("cannot parse as number: {s:?}"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        if !fields.is_empty() {
            rows.push(fields);
        }
    }

    if rows.is_empty() {
        return Err("no data".to_string());
    }

    let ncols = rows[0].len();

    if ncols == 1 {
        let x: Vec<f64> = (1..=rows.len()).map(|i| i as f64).collect();
        let y: Vec<f64> = rows.iter().map(|r| r[0]).collect();
        Ok(DataSet {
            headers,
            x,
            series: vec![y],
        })
    } else {
        let x: Vec<f64> = rows.iter().map(|r| r[0]).collect();
        let mut series = Vec::with_capacity(ncols - 1);
        for col in 1..ncols {
            let y: Vec<f64> = rows
                .iter()
                .map(|r| if col < r.len() { r[col] } else { 0.0 })
                .collect();
            series.push(y);
        }
        Ok(DataSet { headers, x, series })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_two_columns() {
        let lines: Vec<String> = vec!["1\t10", "2\t20", "3\t15"]
            .into_iter()
            .map(String::from)
            .collect();
        let ds = parse_lines(&lines, '\t', false).unwrap();
        assert_eq!(ds.x, vec![1.0, 2.0, 3.0]);
        assert_eq!(ds.series, vec![vec![10.0, 20.0, 15.0]]);
        assert!(ds.headers.is_none());
    }

    #[test]
    fn parse_single_column() {
        let lines: Vec<String> = vec!["10", "20", "15"]
            .into_iter()
            .map(String::from)
            .collect();
        let ds = parse_lines(&lines, '\t', false).unwrap();
        assert_eq!(ds.x, vec![1.0, 2.0, 3.0]);
        assert_eq!(ds.series, vec![vec![10.0, 20.0, 15.0]]);
    }

    #[test]
    fn parse_with_headers() {
        let lines: Vec<String> = vec!["x\ty", "1\t10", "2\t20"]
            .into_iter()
            .map(String::from)
            .collect();
        let ds = parse_lines(&lines, '\t', true).unwrap();
        assert_eq!(ds.headers.as_ref().unwrap(), &vec!["x", "y"]);
        assert_eq!(ds.x, vec![1.0, 2.0]);
        assert_eq!(ds.series, vec![vec![10.0, 20.0]]);
    }

    #[test]
    fn parse_csv_delimiter() {
        let lines: Vec<String> = vec!["1,10", "2,20"].into_iter().map(String::from).collect();
        let ds = parse_lines(&lines, ',', false).unwrap();
        assert_eq!(ds.x, vec![1.0, 2.0]);
        assert_eq!(ds.series, vec![vec![10.0, 20.0]]);
    }

    #[test]
    fn parse_multi_series() {
        let lines: Vec<String> = vec!["0\t1\t2", "1\t3\t4"]
            .into_iter()
            .map(String::from)
            .collect();
        let ds = parse_lines(&lines, '\t', false).unwrap();
        assert_eq!(ds.x, vec![0.0, 1.0]);
        assert_eq!(ds.series.len(), 2);
        assert_eq!(ds.series[0], vec![1.0, 3.0]);
        assert_eq!(ds.series[1], vec![2.0, 4.0]);
    }

    #[test]
    fn parse_empty_is_err() {
        let lines: Vec<String> = vec![];
        assert!(parse_lines(&lines, '\t', false).is_err());
    }

    #[test]
    fn x_range_single_value() {
        let ds = DataSet {
            headers: None,
            x: vec![5.0],
            series: vec![vec![10.0]],
        };
        let (min, max) = ds.x_range();
        assert!(min < max);
    }

    #[test]
    fn y_range_across_series() {
        let ds = DataSet {
            headers: None,
            x: vec![1.0, 2.0],
            series: vec![vec![5.0, 10.0], vec![3.0, 20.0]],
        };
        assert_eq!(ds.y_range(), (3.0, 20.0));
    }

    #[test]
    fn parse_bar_two_columns() {
        let lines: Vec<String> = vec!["cat\t30", "dog\t45", "parrot\t12"]
            .into_iter()
            .map(String::from)
            .collect();
        let bd = parse_bar_lines(&lines, '\t', false).unwrap();
        assert_eq!(bd.labels, vec!["cat", "dog", "parrot"]);
        assert_eq!(bd.values, vec![30.0, 45.0, 12.0]);
    }

    #[test]
    fn parse_bar_single_column() {
        let lines: Vec<String> = vec!["10", "20", "15"]
            .into_iter()
            .map(String::from)
            .collect();
        let bd = parse_bar_lines(&lines, '\t', false).unwrap();
        assert_eq!(bd.labels, vec!["1", "2", "3"]);
        assert_eq!(bd.values, vec![10.0, 20.0, 15.0]);
    }

    #[test]
    fn parse_bar_with_headers() {
        let lines: Vec<String> = vec!["name\tval", "a\t5", "b\t10"]
            .into_iter()
            .map(String::from)
            .collect();
        let bd = parse_bar_lines(&lines, '\t', true).unwrap();
        assert_eq!(bd.labels, vec!["a", "b"]);
        assert_eq!(bd.values, vec![5.0, 10.0]);
    }

    #[test]
    fn parse_hist_values() {
        let lines: Vec<String> = vec!["1.5", "2.3", "3.1", "1.8"]
            .into_iter()
            .map(String::from)
            .collect();
        let vals = parse_hist_lines(&lines, '\t', false).unwrap();
        assert_eq!(vals.len(), 4);
        assert!((vals[0] - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn bin_values_basic() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let bd = bin_values(&values, 5);
        assert_eq!(bd.labels.len(), 5);
        assert_eq!(bd.values.len(), 5);
        let total: f64 = bd.values.iter().sum();
        assert_eq!(total, 10.0);
    }

    #[test]
    fn count_occurrences() {
        let lines: Vec<String> = vec!["apple", "banana", "apple", "apple", "banana", "cherry"]
            .into_iter()
            .map(String::from)
            .collect();
        let bd = parse_count_lines(&lines, '\t', false).unwrap();
        assert_eq!(bd.labels[0], "apple");
        assert_eq!(bd.values[0], 3.0);
        assert_eq!(bd.labels[1], "banana");
        assert_eq!(bd.values[1], 2.0);
        assert_eq!(bd.labels[2], "cherry");
        assert_eq!(bd.values[2], 1.0);
    }

    #[test]
    fn count_tiebreak_alphabetical() {
        let lines: Vec<String> = vec!["cat", "dog", "ant", "dog", "cat", "ant"]
            .into_iter()
            .map(String::from)
            .collect();
        let bd = parse_count_lines(&lines, '\t', false).unwrap();
        // All have count 2 — should be alphabetical
        assert_eq!(bd.labels, vec!["ant", "cat", "dog"]);
        assert!(bd.values.iter().all(|&v| v == 2.0));
    }

    #[test]
    fn transpose_basic() {
        let lines: Vec<String> = vec!["1\t2\t3", "4\t5\t6"]
            .into_iter()
            .map(String::from)
            .collect();
        let t = transpose_lines(&lines, '\t');
        assert_eq!(t.len(), 3);
        assert_eq!(t[0], "1\t4");
        assert_eq!(t[1], "2\t5");
        assert_eq!(t[2], "3\t6");
    }

    #[test]
    fn skip_empty_lines() {
        let lines: Vec<String> = vec!["1\t10", "", "2\t20", ""]
            .into_iter()
            .map(String::from)
            .collect();
        let ds = parse_lines(&lines, '\t', false).unwrap();
        assert_eq!(ds.x, vec![1.0, 2.0]);
    }

    #[test]
    fn filter_values_gt() {
        let vals = vec![1.0, 5.0, 10.0, 15.0, 20.0];
        let out = filter_values(vals, Some(5.0), None);
        assert_eq!(out, vec![10.0, 15.0, 20.0]);
    }

    #[test]
    fn filter_values_lt() {
        let vals = vec![1.0, 5.0, 10.0, 15.0, 20.0];
        let out = filter_values(vals, None, Some(15.0));
        assert_eq!(out, vec![1.0, 5.0, 10.0]);
    }

    #[test]
    fn filter_values_both() {
        let vals = vec![1.0, 5.0, 10.0, 15.0, 20.0];
        let out = filter_values(vals, Some(1.0), Some(20.0));
        assert_eq!(out, vec![5.0, 10.0, 15.0]);
    }

    #[test]
    fn filter_values_none() {
        let vals = vec![1.0, 2.0, 3.0];
        let out = filter_values(vals, None, None);
        assert_eq!(out, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn filter_values_all_excluded() {
        let vals = vec![1.0, 2.0, 3.0];
        let out = filter_values(vals, Some(10.0), None);
        assert!(out.is_empty());
    }

    #[test]
    fn bin_values_log_basic() {
        let vals: Vec<f64> = (0..300).map(|i| 10f64.powf(i as f64 / 100.0)).collect();
        let bar = bin_values_log(&vals, 6).unwrap();
        let total: f64 = bar.values.iter().sum();
        assert_eq!(total, 300.0);
        assert!(bar.labels[0].starts_with('['));
        assert!(bar.labels[0].contains(','));
        assert!(bar.labels.len() >= 2);
    }

    #[test]
    fn bin_values_log_single_decade() {
        let vals = vec![1.0, 2.0, 3.0, 5.0, 7.0, 9.0];
        let bar = bin_values_log(&vals, 4).unwrap();
        let total: f64 = bar.values.iter().sum();
        assert_eq!(total, 6.0);
    }

    #[test]
    fn bin_values_log_negative_errors() {
        let vals = vec![1.0, -5.0, 10.0];
        let result = bin_values_log(&vals, 5);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("--log requires all values > 0")
        );
    }

    #[test]
    fn bin_values_log_zero_errors() {
        let vals = vec![0.0, 1.0, 10.0];
        let result = bin_values_log(&vals, 5);
        assert!(result.is_err());
    }

    #[test]
    fn bin_values_log_all_same() {
        let vals = vec![100.0, 100.0, 100.0];
        let bar = bin_values_log(&vals, 5).unwrap();
        let total: f64 = bar.values.iter().sum();
        assert_eq!(total, 3.0);
    }
}
